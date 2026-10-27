use rstrbot_plugin::{
    sdk::{self, RpcFailure},
    webhook,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    if std::env::args().any(|argument| argument == "--mcp-stdio") {
        if webhook::serve_mcp_stdio().await.is_err() {
            eprintln!("webhook MCP service stopped");
            std::process::exit(1);
        }
        return;
    }
    let result=sdk::serve(&["platform"],|method,params,call|async move {
        match method.as_str() {
            "platform.capabilities"=>Ok(json!({"send":true,"receive":false,"receipts":"http_acceptance","deduplication_guaranteed":false,"formats":["text"]})),
            "platform.send"=>tokio::select!{
                _=call.cancellation.cancelled()=>Err(RpcFailure::cancelled()),
                result=webhook::send(&params)=>result,
            },
            _=>Err(RpcFailure::not_found()),
        }
    }).await;
    if let Err(error) = result {
        eprintln!("{}", error.code());
        std::process::exit(1);
    }
}
