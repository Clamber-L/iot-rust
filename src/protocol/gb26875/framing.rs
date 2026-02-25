use bytes::BytesMut;
use crate::error::IotResult;
use crate::protocol::traits::FrameDetector;

pub struct Gb26875FrameDetector {
    max_frame_size: usize,
}

impl Gb26875FrameDetector {
    pub fn new(max_frame_size: usize) -> Self {
        Self { max_frame_size }
    }
}

impl FrameDetector for Gb26875FrameDetector {
    fn check_frame(&mut self, frame: &BytesMut) -> IotResult<bool> {
        // 验证消息准确性 长度 校验和 启动符 结束符
        todo!()
    }

    fn detector_frame(&mut self, frame: &BytesMut) -> IotResult<Option<usize>> {
        todo!()
    }
}