use tokio::net::TcpListener;
use crate::error::IotError;
use crate::server::config::ServerConfig;

mod server;
mod error;
mod protocol;

#[tokio::main]
async fn main() -> Result<(), IotError> {
    let config = ServerConfig::example();
    for listener_config in config.listeners {
        let addr = format!("{}:{}", listener_config.bind_addr, listener_config.port);
        println!("协议内容：{:?}, 协议监听地址：{:?}", listener_config, addr);
        let listener = TcpListener::bind(&addr).await?;
        
        let port = listener_config.port;
    }
    Ok(())
}


struct MessageParser {
    data: Vec<u8>,
    position: usize,
}

impl MessageParser {
    fn new(data: Vec<u8>) -> Self {
        Self { data, position: 0 }
    }

    // 读取指定长度的字节
    fn read_bytes(&mut self, len: usize) -> Option<&[u8]> {
        if self.position + len <= self.data.len() {
            let slice = &self.data[self.position..self.position + len];
            self.position += len;
            Some(slice)
        } else {
            None
        }
    }

    // 读取单个字节
    fn read_u8(&mut self) -> Option<u8> {
        self.read_bytes(1).map(|b| b[0])
    }

    // 读取两个字节（大端序）
    fn read_u16(&mut self) -> Option<u16> {
        self.read_bytes(2).map(|b| u16::from_be_bytes([b[0], b[1]]))
    }

    // 读取四个字节（大端序）
    fn read_u32(&mut self) -> Option<u32> {
        self.read_bytes(4)
            .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    // 跳过指定字节数
    fn skip(&mut self, len: usize) {
        self.position += len;
    }

    // 获取当前位置
    fn position(&self) -> usize {
        self.position
    }
}

fn hex_example() {
    let hex_str =
        "40406768010C2009001E07158100000000000000000000003000020201010200290008000200BEC6B5EAB8BAD2BBB2E3C8E2C0E0BCD3B9A4BCE42020202020202020202020111912180715882323";

    match hex::decode(hex_str) {
        Ok(bytes) => {
            println!("=== 报文完整解析 ===\n");
            println!("总长度: {} 字节\n", bytes.len());

            let mut parser = MessageParser::new(bytes);

            // 1. 起始符 (2字节)
            if let Some(header) = parser.read_bytes(2) {
                println!("[偏移 0-1] 起始符: {:02X} {:02X} (ASCII: {})",
                    header[0], header[1], String::from_utf8_lossy(header));
            }

            // 2. 设备标识 (2字节)
            if let Some(device_id) = parser.read_bytes(2) {
                println!("[偏移 2-3] 设备标识: {:02X} {:02X}", device_id[0], device_id[1]);
            }

            // 3. 命令字 (1字节)
            if let Some(cmd) = parser.read_u8() {
                println!("[偏移 4] 命令字: 0x{:02X}", cmd);
            }

            // 4. 数据长度 (1字节)
            if let Some(length) = parser.read_u8() {
                println!("[偏移 5] 数据长度: 0x{:02X} ({} 字节)", length, length);
            }

            // 5. 地址域 (2字节)
            if let Some(addr) = parser.read_bytes(2) {
                println!("[偏移 6-7] 地址域: {:02X} {:02X}", addr[0], addr[1]);
            }

            // 6. 数据域长度 (2字节)
            if let Some(data_len) = parser.read_u16() {
                println!("[偏移 8-9] 数据域长度: 0x{:04X} ({} 字节)", data_len, data_len);
            }

            // 7. 时间标签 (7字节) - 年月日时分秒毫秒
            if let Some(time_bytes) = parser.read_bytes(7) {
                println!("[偏移 10-16] 时间标签: {:02X?}", time_bytes);
                println!("  -> 年: 20{:02X}, 月: {:02X}, 日: {:02X}, 时: {:02X}, 分: {:02X}, 秒: {:02X}, 毫秒: {:02X}",
                    time_bytes[0], time_bytes[1], time_bytes[2],
                    time_bytes[3], time_bytes[4], time_bytes[5], time_bytes[6]);
            }

            // 8. 保留字段或其他数据 (10字节全0)
            if let Some(reserved) = parser.read_bytes(10) {
                println!("[偏移 17-26] 保留字段: {:02X?}", reserved);
            }

            // 9. 未知字段
            if let Some(unknown1) = parser.read_bytes(7) {
                println!("[偏移 27-33] 数据字段1: {:02X?}", unknown1);
            }

            // 10. 数据内容长度标识
            if let Some(content_len) = parser.read_u16() {
                println!("[偏移 34-35] 内容长度: 0x{:04X} ({} 字节)", content_len, content_len);
            }

            // 11. 文本内容 (GBK编码的中文)
            if let Some(text_bytes) = parser.read_bytes(32) {
                println!("[偏移 36-67] 文本数据: {:02X?}", text_bytes);
                let (text, _, _) = encoding_rs::GBK.decode(text_bytes);
                println!("  -> 文本内容: \"{}\"", text.trim());
            }

            // 12. 时间戳字段
            if let Some(timestamp) = parser.read_bytes(10) {
                println!("[偏移 68-77] 时间戳: {:02X?}", timestamp);
                let timestamp_str: String = timestamp.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect();
                println!("  -> 时间字符串: {}", timestamp_str);
                // 解析为: 年年月月日日时时分分秒秒
                if timestamp.len() >= 6 {
                    println!("  -> 20{:02X}-{:02X}-{:02X} {:02X}:{:02X}:{:02X}",
                        timestamp[0], timestamp[1], timestamp[2],
                        timestamp[3], timestamp[4], timestamp[5]);
                }
            }

            println!("\n解析完成！当前位置: {} 字节", parser.position());
        }
        Err(e) => {
            eprintln!("十六进制解析失败: {}", e);
        }
    }
}