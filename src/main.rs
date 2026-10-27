use clap::{Parser, Subcommand};
use fs2::FileExt;
use rstrbot::{
    api, engine, schedule,
    state::{AppState, Config, atomic_private_write, private_dir},
};
use rstrbot_store::{Backup, Store};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

type CliResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Parser)]
#[command(
    name = "rstrbot",
    version,
    about = "A private workspace for conversations, tools and knowledge"
)]
struct Cli {
    /// Use a portable data directory instead of the operating system default.
    #[arg(long, global = true, env = "RSTRBOT_DATA_DIR")]
    data_dir: Option<PathBuf>,
    /// SQLite or PostgreSQL connection URL. Prefer the environment for secrets.
    #[arg(long, global = true, env = "RSTRBOT_DATABASE_URL")]
    database_url: Option<String>,
    #[arg(
        long,
        global = true,
        default_value = "127.0.0.1:8787",
        env = "RSTRBOT_LISTEN"
    )]
    listen: SocketAddr,
    #[arg(long, global = true, env = "RSTRBOT_VALKEY_URL")]
    valkey_url: Option<String>,
    /// Use only behind an explicitly configured HTTPS reverse proxy.
    #[arg(long, global = true, env = "RSTRBOT_SECURE_COOKIE")]
    secure_cookie: bool,
    /// Permit configured providers on private networks; this is an operator trust decision.
    #[arg(long, global = true, env = "RSTRBOT_ALLOW_PRIVATE_NETWORK")]
    allow_private_network: bool,
    /// Directory containing the built WebUI index.html.
    #[arg(long, global = true, env = "RSTRBOT_WEB_DIR")]
    web_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start the core and WebUI (the default command).
    Serve,
    /// Write a consistent, sensitive database snapshot to a new file.
    Backup {
        #[arg(long)]
        output: PathBuf,
    },
    /// Import a snapshot into an empty database and keep external effects paused.
    Restore {
        #[arg(long)]
        input: PathBuf,
    },
    /// Local operator recovery; password is read from RSTRBOT_RECOVERY_PASSWORD.
    RecoverPassword {
        #[arg(long)]
        username: String,
    },
    /// Verify database access and report initialization/recovery state.
    Check,
    /// Register a trusted local executable manifest; does not start its process.
    PluginInstall {
        #[arg(long)]
        manifest: PathBuf,
        /// Replace this installed package after reviewing its permissions.
        #[arg(long)]
        replace: bool,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rstrbot=info,tower_http=info".into()),
        )
        .with_target(false)
        .init();
    if let Err(error) = run(Cli::parse()).await {
        eprintln!("RstrBot: {error}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> CliResult<()> {
    let data_dir = match cli.data_dir {
        Some(path) => path,
        None => directories::ProjectDirs::from("org", "RstrBot", "RstrBot")
            .ok_or("specify --data-dir on this platform")?
            .data_local_dir()
            .to_owned(),
    };
    private_dir(&data_dir)?;
    let _instance_lock = acquire_lock(&data_dir)?;
    let config = Config::load(
        Some(data_dir),
        cli.database_url,
        cli.listen,
        cli.valkey_url,
        cli.secure_cookie,
        cli.allow_private_network,
    )?;
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve(config, cli.web_dir).await,
        Command::Backup { output } => {
            let store = Store::connect(&config.database_url).await?;
            if !store.initialized().await? {
                return Err("initialize this instance before taking a database backup".into());
            }
            let backup = store.export_backup_local().await?;
            write_new_private(&output, &serde_json::to_vec_pretty(&backup)?)?;
            println!("Database snapshot written to {}", output.display());
            println!("SHA-256: {}", backup.sha256);
            println!(
                "Also preserve secrets.key and external attachments/plugin state separately. Sessions are excluded."
            );
            store.close().await;
            Ok(())
        }
        Command::Restore { input } => {
            let metadata = fs::metadata(&input)?;
            if metadata.len() > 300 * 1024 * 1024 {
                return Err("snapshot exceeds the 300 MiB input limit".into());
            }
            let backup: Backup = serde_json::from_slice(&fs::read(&input)?)?;
            let store = Store::connect(&config.database_url).await?;
            store.import_backup(&backup).await?;
            println!("Database restored into an empty target. Recovery mode is active.");
            println!(
                "Restore the original secrets.key, review external receipts and plugin state, then explicitly leave recovery mode."
            );
            store.close().await;
            Ok(())
        }
        Command::RecoverPassword { username } => {
            let password = std::env::var("RSTRBOT_RECOVERY_PASSWORD").map_err(
                |_| "set RSTRBOT_RECOVERY_PASSWORD through a protected local environment",
            )?;
            let store = Store::connect(&config.database_url).await?;
            store.recover_password(&username, &password).await?;
            println!("Password changed; all sessions for this account were revoked.");
            store.close().await;
            Ok(())
        }
        Command::Check => {
            let store = Store::connect(&config.database_url).await?;
            store.health().await?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"database":store.backend(),"database_available":true,"initialized":store.initialized().await?,"recovery_mode":store.recovery_mode().await?,"valkey_configured":config.valkey_url.is_some(),"provider_connectivity":"not_checked"})
                )?
            );
            store.close().await;
            Ok(())
        }
        Command::PluginInstall { manifest, replace } => {
            install_plugin(&config.data_dir, &manifest, replace)
        }
    }
}

async fn serve(config: Config, web_dir: Option<PathBuf>) -> CliResult<()> {
    if !config.listen.ip().is_loopback() && !config.secure_cookie {
        return Err("non-loopback listeners require --secure-cookie and an HTTPS reverse proxy; the core HTTP socket does not provide TLS".into());
    }
    let web_dir = web_dir.unwrap_or_else(default_web_dir);
    if !web_dir.join("index.html").is_file() {
        return Err(format!(
            "WebUI not found at {}; run npm ci && npm run build in web/ or pass --web-dir",
            web_dir.display()
        )
        .into());
    }
    let address = config.listen;
    let data_dir = config.data_dir.clone();
    let state = AppState::new(config).await?;
    engine::recover(state.clone()).await?;
    rstrbot::delivery::recover(&state).await?;
    schedule::recover(&state).await?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let scheduler = tokio::spawn(schedule::run_scheduler(state.clone()));
    println!("RstrBot listening on http://{address}");
    if data_dir.join("bootstrap.token").exists() && !state.store().await.initialized().await? {
        println!(
            "Read the short-lived initialization proof from {}",
            data_dir.join("bootstrap.token").display()
        );
    }
    if !address.ip().is_loopback() {
        println!(
            "This upstream socket is HTTP. Expose only the configured HTTPS reverse proxy to clients."
        );
    }
    let signal_state = state.clone();
    let result = axum::serve(listener, api::router(state.clone(), web_dir))
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            signal_state.shutdown_runs().await;
        })
        .await;
    state.shutdown_runs().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), scheduler).await;
    state.store().await.close().await;
    result?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        if let Ok(mut terminate) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}

fn acquire_lock(data_dir: &Path) -> CliResult<File> {
    let path = data_dir.join("instance.lock");
    if path
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err("instance.lock cannot be a symlink".into());
    }
    let mut options = OpenOptions::new();
    options.create(true).read(true).write(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    file.try_lock_exclusive().map_err(|_| "another RstrBot process is using this data directory; stop it before local maintenance")?;
    Ok(file)
}

fn write_new_private(path: &Path, contents: &[u8]) -> CliResult<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

fn default_web_dir() -> PathBuf {
    if let Ok(executable) = std::env::current_exe()
        && let Some(parent) = executable.parent()
    {
        let adjacent = parent.join("web");
        if adjacent.join("index.html").is_file() {
            return adjacent;
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("web/dist")
}

fn install_plugin(data_dir: &Path, source: &Path, replace: bool) -> CliResult<()> {
    if fs::metadata(source)?.len() > 64 * 1024 {
        return Err("plugin manifest exceeds 64 KiB".into());
    }
    let mut manifest: rstrbot_plugin::Manifest = serde_json::from_slice(&fs::read(source)?)?;
    rstrbot_plugin::validate_manifest(&manifest)?;
    if ["openai", "diagnostic", "webhook"].contains(&manifest.id.as_str()) {
        return Err("package id is reserved for a bundled plugin".into());
    }
    manifest.executable = manifest.executable.canonicalize()?;
    if !manifest.executable.is_file() {
        return Err("plugin executable must be a regular file".into());
    }
    let mut file = File::open(&manifest.executable)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = format!("{:x}", hasher.finalize());
    if manifest
        .sha256
        .as_ref()
        .is_some_and(|v| !v.eq_ignore_ascii_case(&digest))
    {
        return Err("plugin executable checksum does not match its manifest".into());
    }
    manifest.sha256 = Some(digest);
    let directory = data_dir.join("plugins");
    private_dir(&directory)?;
    let destination = directory.join(format!("{}.json", manifest.id));
    let contents = serde_json::to_vec_pretty(&manifest)?;
    if replace {
        atomic_private_write(&destination, &contents)?;
    } else {
        write_new_private(&destination, &contents)?;
    }
    println!(
        "Registered {} {}. Capabilities: {}. Permission disclosures: {}.",
        manifest.id,
        manifest.version,
        manifest.capabilities.join(", "),
        manifest.permissions.join(", ")
    );
    println!(
        "This local executable is trusted code, not an OS sandbox. Enable a separate workspace instance in the WebUI."
    );
    Ok(())
}
