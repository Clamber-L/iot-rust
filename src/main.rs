use crate::error::IotError;
use crate::server::config::ServerConfig;
use crate::server::listener::run_listener;

mod server;
mod error;
mod protocol;

// 40406768010C2009001E07158100000000000000000000003000020201010200290008000200BEC6B5EAB8BAD2BBB2E3C8E2C0E0BCD3B9A4BCE42020202020202020202020111912180715882323

#[tokio::main]
async fn main() -> Result<(), IotError> {
    let config = ServerConfig::from_file("config.toml")?;

    let mut handles = Vec::new();
    for listener_config in config.listeners {
        let handle = tokio::spawn(run_listener(listener_config));
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
