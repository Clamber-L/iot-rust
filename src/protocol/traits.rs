use bytes::BytesMut;
use crate::error::IotResult;

pub trait FrameDetector: Send + Sync {
    fn check_frame(&mut self, frame: &BytesMut) -> IotResult<bool>;

    fn detector_frame(&mut self, frame: &BytesMut) -> IotResult<Option<usize>>;
}

pub trait ProtocolParser: Send + Sync {
    fn parse(&self, bytes: BytesMut) -> IotResult<BytesMut>;
}