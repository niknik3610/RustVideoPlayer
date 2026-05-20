use std::fs;
use crate::{consumer::Consumer, sw_scale::ScalerOutput, wrappers::WrappedAVFrame};

pub struct FrameWriter{
}
impl FrameWriter {
    pub fn new() -> Self {
        return Self{};
    }
    fn write_frame_to_disk(&self, to_consume: ScalerOutput) {
        let number = to_consume.pts / to_consume.duration;

        //Data without padding bytes
        let mut cleansed_data = vec![];

        let header = format!("P6\n{} {}\n{}\n",to_consume.width, to_consume.height, 255);
        cleansed_data.extend_from_slice(header.as_bytes());
        
        //to_consume.data == u8*[] of length 8;
        let data = unsafe { std::slice::from_raw_parts(to_consume.data_ptrs[0], (to_consume.data_linesizes[0] * to_consume.height) as usize) };
        cleansed_data.extend_from_slice(data);
        fs::write(format!("/tmp/test_to_consume{}", number), cleansed_data)
            .expect(format!("Failed to write to_consume {} to disk", number).as_str());
    }
}
impl Consumer<ScalerOutput> for FrameWriter {
    fn consume(&mut self, to_consume: ScalerOutput) {
        self.write_frame_to_disk(to_consume);
    }
}
