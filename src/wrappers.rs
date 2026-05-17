use rusty_ffmpeg::ffi::{AVFrame, AVPacket, av_frame_clone, av_frame_free, av_packet_clone, av_packet_free};

pub struct WrappedAVPacket(pub *mut AVPacket);
//Tells compiler it is safe to transfer ownership of this object to another thread
unsafe impl Send for WrappedAVPacket {}             
impl Drop for WrappedAVPacket {
    fn drop(&mut self) {
        unsafe {
            av_packet_free(&mut self.0);
        }
    }
}
impl Clone for WrappedAVPacket {
    fn clone(&self) -> Self {
        unsafe {
            Self(av_packet_clone(self.0))
        }
    }
}

pub struct WrappedAVFrame(pub *mut AVFrame);
unsafe impl Send for WrappedAVFrame {}             
impl Drop for WrappedAVFrame {
    fn drop(&mut self) {
        unsafe {
            av_frame_free(&mut self.0);
        }
    }
}
impl Clone for WrappedAVFrame {
    fn clone(&self) -> Self {
        unsafe {
            Self(av_frame_clone(self.0))
        }
    }
}
