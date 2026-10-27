use rstrbot_plugin::{provider, sdk};

#[tokio::main]
async fn main() {
    if let Err(error) = sdk::serve(&["model"], provider::dispatch).await {
        eprintln!("{}", error.code());
        std::process::exit(1);
    }
}
