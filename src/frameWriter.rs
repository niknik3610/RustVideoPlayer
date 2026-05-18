use std::fs;

use rusty_ffmpeg::ffi::AVFrame;

use crate::{consumer::Consumer, wrappers::WrappedAVFrame};

pub struct FrameWriter();
impl FrameWriter {
    pub fn new() -> Self {
        return Self{};
    }
    fn write_frame_to_disk(&self, to_consume: WrappedAVFrame) {
        let frame = unsafe { *to_consume.0 };
        let number = frame.pts / frame.duration;

        let header = format!("P5\n{} {}\n{}\n", frame.width, frame.height, 255);
        
        //frame.data == u8*[] of length 8;
        let data = unsafe { 
            //this data has unitilzed bytes for alignment purposes
            let data = std::slice::from_raw_parts(frame.data[0], (frame.linesize[0] * frame.height) as usize);
            let mut cleansed_data = vec![];
            cleansed_data.extend_from_slice(header.as_bytes());

            for y in 0..frame.height {
                let slice = &data[(y * frame.linesize[0]) as usize..(y * frame.linesize[0] + frame.width) as usize];
                cleansed_data.extend_from_slice(slice); 
            };
            cleansed_data
        };
        fs::write(format!("/tmp/test_frame{}", number), data);
    }
}
impl Consumer<WrappedAVFrame> for FrameWriter {
    fn consume(&mut self, to_consume: WrappedAVFrame) {
        self.write_frame_to_disk(to_consume);
    }
}
