use std::net::SocketAddr;
use futures::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::FramedRead;
use crate::server::config::ListenerConfig;
use crate::protocol::gb26875::codec::Gb26875Codec;

pub async fn run_listener(config: ListenerConfig) {
    let addr = format!("{}:{}", config.bind_addr, config.port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            println!("[{}] 监听启动，协议: {}", addr, config.protocol);
            l
        }
        Err(e) => {
            eprintln!("[{}] 绑定失败: {}", addr, e);
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let protocol = config.protocol.clone();
                tokio::spawn(async move {
                    handle_connection(stream, peer_addr, &protocol).await;
                });
            }
            Err(e) => eprintln!("[{}] accept 错误: {}", addr, e),
        }
    }
}

async fn handle_connection(stream: TcpStream, peer_addr: SocketAddr, protocol: &str) {
    println!("[{}] 新连接，协议: {}", peer_addr, protocol);

    match protocol {
        "Gb26875" => {
            let mut framed = FramedRead::new(stream, Gb26875Codec::new());
            while let Some(result) = framed.next().await {
                match result {
                    Ok(frame) => {
                        println!("[{}] 收到完整帧，长度 {} 字节", peer_addr, frame.len());
                        // TODO: 将 frame 交给 ProtocolParser 做进一步解析
                    }
                    Err(e) => {
                        eprintln!("[{}] 帧错误: {}", peer_addr, e);
                        break;
                    }
                }
            }
        }
        unknown => {
            eprintln!("[{}] 未知协议: {}", peer_addr, unknown);
        }
    }

    println!("[{}] 连接断开", peer_addr);
}
