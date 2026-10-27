#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, RunEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use url::Url;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    #[default]
    Idle,
    Starting,
    Local,
    Remote,
    Stopping,
}

#[derive(Default)]
struct Session {
    mode: Mode,
    generation: u64,
    child: Option<Child>,
}

#[derive(Clone, Default)]
struct DesktopState {
    session: Arc<Mutex<Session>>,
    exiting: Arc<AtomicBool>,
    shutdown_complete: Arc<AtomicBool>,
    cleanup: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Serialize)]
struct LaunchInfo {
    mode: &'static str,
    url: String,
    data_dir: Option<String>,
    setup_proof: Option<String>,
}

fn bundled_launcher(window: &WebviewWindow) -> Result<(), String> {
    let location = window
        .url()
        .map_err(|_| "Could not verify launcher origin")?;
    let app_origin = location.scheme() == "tauri" && location.host_str() == Some("localhost")
        || matches!(location.scheme(), "http" | "https")
            && location.host_str() == Some("tauri.localhost");
    if window.label() != "launcher" || !app_origin {
        return Err("Only the bundled launcher can manage the local process".into());
    }
    Ok(())
}

fn begin_session(state: &DesktopState) -> Result<u64, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "Launcher state unavailable")?;
    if session.mode != Mode::Idle {
        return Err("Disconnect the current workspace before opening another one".into());
    }
    session.generation = session.generation.wrapping_add(1);
    session.mode = Mode::Starting;
    Ok(session.generation)
}

fn open_workspace(app: &AppHandle, url: Url) -> Result<(), String> {
    let origin = url.origin();
    WebviewWindowBuilder::new(app, "workspace", WebviewUrl::External(url))
        .title("RstrBot")
        .inner_size(1280.0, 820.0)
        .min_inner_size(320.0, 480.0)
        .on_navigation(move |next| next.origin() == origin)
        .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
        .build()
        .map_err(|_| "The workspace window could not be opened".to_owned())?;
    Ok(())
}

fn stop_owned_process(mut child: Child) {
    if child.try_wait().ok().flatten().is_some() {
        return;
    }
    #[cfg(unix)]
    {
        use nix::{
            sys::signal::{Signal, kill},
            unistd::Pid,
        };
        if let Ok(pid) = i32::try_from(child.id()) {
            // The core handles SIGTERM and cancels its registered plugin process groups.
            let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
        }
        let deadline = Instant::now() + Duration::from_secs(12);
        while Instant::now() < deadline {
            if child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

async fn stop_session(state: DesktopState) {
    let _cleanup = state.cleanup.lock().await;
    let (child, generation) = {
        let Ok(mut session) = state.session.lock() else {
            return;
        };
        if session.mode == Mode::Idle {
            return;
        }
        session.generation = session.generation.wrapping_add(1);
        session.mode = Mode::Stopping;
        (session.child.take(), session.generation)
    };
    if let Some(child) = child {
        let _ = tauri::async_runtime::spawn_blocking(move || stop_owned_process(child)).await;
    }
    if let Ok(mut session) = state.session.lock() {
        if session.generation == generation {
            session.mode = Mode::Idle;
        }
    }
}

fn spawn_core(app: &AppHandle) -> Result<(Child, mpsc::Receiver<()>, Url, PathBuf), String> {
    let directory = directories::ProjectDirs::from("org", "RstrBot", "RstrBot")
        .ok_or("No application data directory is available")?
        .data_local_dir()
        .to_owned();
    if directory
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err("The application data directory cannot be a symlink".into());
    }
    fs::create_dir_all(&directory)
        .map_err(|_| "Could not create the application data directory")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .map_err(|_| "Could not protect the application data directory")?;
    }
    let executable = std::env::current_exe().map_err(|_| "Could not locate the application")?;
    let executable = executable
        .parent()
        .ok_or("Application executable directory is unavailable")?
        .join(format!("rstrbot{}", std::env::consts::EXE_SUFFIX));
    if !executable.is_file() {
        return Err(
            "Bundled rstrbot sidecar is missing; prepare desktop assets before building".into(),
        );
    }
    let web = app
        .path()
        .resource_dir()
        .map_err(|_| "Bundled resources are unavailable")?
        .join("web");
    if !web.join("index.html").is_file() {
        return Err("Bundled WebUI is missing; prepare desktop assets before building".into());
    }
    let reservation = TcpListener::bind("127.0.0.1:0").map_err(|_| "No local port is available")?;
    let address = reservation
        .local_addr()
        .map_err(|_| "Could not read the local port")?;
    let url = Url::parse(&format!("http://{address}")).map_err(|_| "Invalid local address")?;
    let marker = format!("RstrBot listening on http://{address}");
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let log_path = directory.join("desktop-core.log");
    if log_path
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err("The local process log cannot be a symlink".into());
    }
    let mut log = options
        .open(&log_path)
        .map_err(|_| "Could not open the local process log")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        log.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|_| "Could not protect the local process log")?;
    }
    let stderr = log
        .try_clone()
        .map_err(|_| "Could not attach the local process log")?;
    drop(reservation);
    let mut command = Command::new(executable);
    command
        .args(["--listen", &address.to_string(), "--data-dir"])
        .arg(&directory)
        .arg("--web-dir")
        .arg(web)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(stderr));
    // An inherited service configuration must not silently redirect local mode.
    for key in [
        "RSTRBOT_DATABASE_URL",
        "RSTRBOT_DATA_DIR",
        "RSTRBOT_LISTEN",
        "RSTRBOT_WEB_DIR",
        "RSTRBOT_SECURE_COOKIE",
        "RSTRBOT_VALKEY_URL",
        "RSTRBOT_ALLOW_PRIVATE_NETWORK",
    ] {
        command.env_remove(key);
    }
    let mut child = command
        .spawn()
        .map_err(|_| "The bundled backend could not start")?;
    let Some(stdout) = child.stdout.take() else {
        stop_owned_process(child);
        return Err("Could not observe the backend startup result".into());
    };
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut announced = false;
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if !announced && line.trim() == marker {
                let _ = sender.try_send(());
                announced = true;
            }
            // Keep draining even if the disk fills; closing the read side would
            // turn the core's next stdout write into an unrelated process failure.
            let _ = writeln!(log, "{line}");
        }
    });
    Ok((child, receiver, url, directory))
}

#[tauri::command]
async fn start_local(app: AppHandle, window: WebviewWindow) -> Result<LaunchInfo, String> {
    bundled_launcher(&window)?;
    if cfg!(windows) {
        return Err("This preview does not yet provide the Windows plugin transport. Connect to an existing server instead.".into());
    }
    let state = app.state::<DesktopState>().inner().clone();
    let generation = begin_session(&state)?;
    let (child, ready, url, directory) = match spawn_core(&app) {
        Ok(value) => value,
        Err(error) => {
            stop_session(state).await;
            return Err(error);
        }
    };
    let registered = if let Ok(mut session) = state.session.lock() {
        if session.generation == generation {
            session.child = Some(child);
            true
        } else {
            drop(session);
            stop_owned_process(child);
            false
        }
    } else {
        stop_owned_process(child);
        false
    };
    if !registered {
        return Err("Startup was cancelled or launcher state became unavailable".into());
    }
    let ready =
        tauri::async_runtime::spawn_blocking(move || ready.recv_timeout(Duration::from_secs(25)))
            .await;
    if !matches!(ready, Ok(Ok(()))) {
        stop_session(state).await;
        return Err("Backend startup failed or timed out. Read desktop-core.log in the RstrBot data directory. An existing local service may already own the directory.".into());
    }
    let exited = {
        let mut session = state
            .session
            .lock()
            .map_err(|_| "Launcher state unavailable")?;
        if session.generation != generation || session.mode != Mode::Starting {
            return Err("Startup was cancelled".into());
        }
        let exited = session
            .child
            .as_mut()
            .and_then(|child| child.try_wait().ok().flatten())
            .is_some();
        if !exited {
            session.mode = Mode::Local;
        }
        exited
    };
    if exited {
        stop_session(state).await;
        return Err("The backend exited during startup; inspect desktop-core.log".into());
    }
    if let Err(error) = open_workspace(&app, url.clone()) {
        stop_session(state).await;
        return Err(error);
    }
    // Read only the fixed bootstrap file from this launch's own data directory.
    // It is returned to the privileged bundled launcher, never to the web workspace.
    let proof = fs::read_to_string(directory.join("bootstrap.token"))
        .ok()
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()));
    Ok(LaunchInfo {
        mode: "local",
        url: url.to_string(),
        data_dir: Some(directory.display().to_string()),
        setup_proof: proof,
    })
}

fn remote_url(value: &str) -> Result<Url, String> {
    if value.len() > 2048 {
        return Err("Server address is too long".into());
    }
    let url = Url::parse(value).map_err(|_| "Enter an HTTPS server origin")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(
            "Use an HTTPS server origin without embedded credentials, path, query or fragment"
                .into(),
        );
    }
    Ok(url)
}

#[tauri::command]
async fn connect_remote(
    app: AppHandle,
    window: WebviewWindow,
    server: String,
) -> Result<LaunchInfo, String> {
    bundled_launcher(&window)?;
    let url = remote_url(&server)?;
    let state = app.state::<DesktopState>().inner().clone();
    let generation = begin_session(&state)?;
    if let Err(error) = open_workspace(&app, url.clone()) {
        stop_session(state).await;
        return Err(error);
    }
    let activated = if let Ok(mut session) = state.session.lock() {
        if session.generation == generation && session.mode == Mode::Starting {
            session.mode = Mode::Remote;
            true
        } else {
            false
        }
    } else {
        false
    };
    if !activated {
        if let Some(workspace) = app.get_webview_window("workspace") {
            let _ = workspace.close();
        }
        return Err("Connection was cancelled or launcher state became unavailable".into());
    }
    // No data directory, database, key or local backend is created in this path.
    Ok(LaunchInfo {
        mode: "remote",
        url: url.to_string(),
        data_dir: None,
        setup_proof: None,
    })
}

#[tauri::command]
async fn disconnect(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    bundled_launcher(&window)?;
    let state = app.state::<DesktopState>().inner().clone();
    stop_session(state).await;
    if let Some(workspace) = app.get_webview_window("workspace") {
        workspace
            .close()
            .map_err(|_| "Could not close the workspace")?;
    }
    Ok(())
}

fn main() {
    let app = tauri::Builder::default()
        .manage(DesktopState::default())
        .invoke_handler(tauri::generate_handler![
            start_local,
            connect_remote,
            disconnect
        ])
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(error) => {
            eprintln!("Desktop startup failed: {error}");
            std::process::exit(1);
        }
    };
    app.run(|app, event| {
        let state = app.state::<DesktopState>().inner().clone();
        match event {
            RunEvent::ExitRequested { api, .. } => {
                if state.shutdown_complete.load(Ordering::SeqCst) {
                    return;
                }
                api.prevent_exit();
                if !state.exiting.swap(true, Ordering::SeqCst) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        stop_session(state.clone()).await;
                        state.shutdown_complete.store(true, Ordering::SeqCst);
                        app.exit(0);
                    });
                }
            }
            RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::Destroyed,
                ..
            } if label == "workspace" && !state.exiting.load(Ordering::SeqCst) => {
                tauri::async_runtime::spawn(stop_session(state));
                if let Some(launcher) = app.get_webview_window("launcher") {
                    let _ = launcher.show();
                }
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::remote_url;
    #[test]
    fn remote_destinations_cannot_smuggle_credentials_or_local_protocols() {
        assert!(remote_url("https://bot.example.test").is_ok());
        for value in [
            "http://bot.example.test",
            "file:///tmp/database",
            "https://name:secret@bot.example.test",
            "https://bot.example.test?token=secret",
            "https://bot.example.test/path",
            "javascript:alert(1)",
        ] {
            assert!(remote_url(value).is_err(), "{value}");
        }
    }
}
