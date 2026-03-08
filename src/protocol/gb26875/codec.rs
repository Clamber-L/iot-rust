use bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;
use crate::error::IotError;
use crate::protocol::traits::FrameDetector;
use super::framing::Gb26875FrameDetector;

pub struct Gb26875Codec {
    detector: Gb26875FrameDetector,
}

impl Gb26875Codec {
    pub fn new() -> Self {
        Self {
            detector: Gb26875FrameDetector::new(4096),
        }
    }
}

impl Decoder for Gb26875Codec {
    type Item = BytesMut;
    type Error = IotError;

    /// 粘包/半包处理核心逻辑：
    ///
    /// - 半包：缓冲区中尚无完整的 @@ ... ## 序列时返回 Ok(None)，等待更多数据到达。
    /// - 粘包：split_to 精确消费一帧的字节，剩余字节留在 src，下次调用自动处理。
    /// - 噪声：@@ 之前的无效字节直接丢弃。
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            // 1. 找到 @@ 起始符，丢弃之前的噪声字节
            let start = match src.windows(2).position(|w| w == [0x40u8, 0x40u8]) {
                Some(pos) => pos,
                None => {
                    // 保留最后一个字节，防止 @@ 恰好被拆分到两次读取之间
                    if src.len() > 1 {
                        src.advance(src.len() - 1);
                    }
                    return Ok(None);
                }
            };
            if start > 0 {
                src.advance(start);
            }

            // 2. 防止超大帧卡死：缓冲区已超出最大帧限制仍未找到 ##，跳过当前 @@
            if src.len() > self.detector.max_frame_size {
                src.advance(2);
                continue;
            }

            // 3. 查找 ## 结束符，确认完整帧边界
            match self.detector.detector_frame(src)? {
                None => return Ok(None), // 半包，等待更多数据
                Some(frame_len) => {
                    // 4. 精确提取一帧（split_to 使剩余字节自动留在 src 中，解决粘包）
                    let frame = src.split_to(frame_len);

                    // 5. 校验帧的合法性（起始符、结束符、校验和）
                    if self.detector.check_frame(&frame)? {
                        return Ok(Some(frame));
                    }
                    // 校验不通过：帧已从 src 中移除，继续寻找下一帧
                }
            }
        }
    }
}
