use bytes::BytesMut;
use crate::error::IotResult;
use crate::protocol::traits::FrameDetector;

const START: [u8; 2] = [0x40, 0x40]; // @@
const END: [u8; 2] = [0x23, 0x23];   // ##

pub struct Gb26875FrameDetector {
    pub max_frame_size: usize,
}

impl Gb26875FrameDetector {
    pub fn new(max_frame_size: usize) -> Self {
        Self { max_frame_size }
    }
}

impl FrameDetector for Gb26875FrameDetector {
    /// 验证已提取帧的合法性：起始符 @@、结束符 ##，以及（TODO）XOR 校验和。
    fn check_frame(&mut self, frame: &BytesMut) -> IotResult<bool> {
        if frame.len() < 4 {
            return Ok(false);
        }
        if frame[0] != START[0] || frame[1] != START[1] {
            return Ok(false);
        }
        let len = frame.len();
        if frame[len - 2] != END[0] || frame[len - 1] != END[1] {
            return Ok(false);
        }
        // Validate XOR checksum: CS is the byte immediately before ##
        // Coverage: frame[2..len-3] (everything between @@ and CS, exclusive)
        let cs_pos = len - 3;
        if cs_pos < 2 {
            return Ok(false);
        }
        let computed: u8 = frame[2..cs_pos].iter().fold(0u8, |acc, &b| acc ^ b);
        if computed != frame[cs_pos] {
            return Ok(false);
        }
        Ok(true)
    }

    /// 在缓冲区中定位一个完整帧的结束位置（含 ## 两字节）。
    /// 要求 buf 中存在 @@ 起始符；返回从 buf 起点到帧末尾（含 ##）的字节数。
    fn detector_frame(&mut self, buf: &BytesMut) -> IotResult<Option<usize>> {
        let start = match buf.windows(2).position(|w| w == START) {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let search_from = start + 2;
        if search_from >= buf.len() {
            return Ok(None);
        }

        match buf[search_from..].windows(2).position(|w| w == END) {
            Some(offset) => Ok(Some(search_from + offset + 2)),
            None => Ok(None),
        }
    }
}
