//! Optional, disposable presence cache. Never an authority for permissions or money.
use crate::state::AppState;
use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use std::time::Duration;

pub async fn heartbeat(state: AppState) {
    let config = state.config.read().await.clone();
    let Some(url) = config.valkey_url else { return };
    let client = match redis::Client::open(url) {
        Ok(c) => c,
        Err(_) => {
            state
                .cache_status
                .write()
                .await
                .clone_from(&"invalid_configuration".to_owned());
            return;
        }
    };
    let key = format!(
        "rstrbot:{}:presence",
        &hex::encode(Sha256::digest(config.data_dir.to_string_lossy().as_bytes()))[..16]
    );
    let mut tick = tokio::time::interval(Duration::from_secs(5));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _=state.shutdown.cancelled()=>break,
            _=tick.tick()=>{
                let active=state.runs.lock().await.len();
                let payload=serde_json::json!({"observed_at":chrono::Utc::now().timestamp(),"active_runs":active}).to_string();
                let operation=async {
                    let mut connection=client.get_multiplexed_async_connection().await?;
                    connection.set_ex::<_,_,()>(&key,&payload,15).await?;
                    let observed:Option<String>=connection.get(&key).await?;
                    Ok::<bool,redis::RedisError>(observed.as_deref()==Some(payload.as_str()))
                };
                let status=if matches!(tokio::time::timeout(Duration::from_secs(2),operation).await,Ok(Ok(true))){"available"}else{"degraded"};
                *state.cache_status.write().await=status.into();
            }
        }
    }
}
