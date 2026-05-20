use std::{os::raw::c_int, ptr::{null_mut}};
use rusty_ffmpeg::ffi::{AVFrame, AVPixelFormat, AVStream, SWS_BILINEAR, SwsContext, av_image_alloc, sws_getContext, sws_scale};
use crate::{consumer::Consumer, producer::Producer, wrappers::WrappedAVFrame};

#[derive(Clone)]
pub struct ScalerOutput {
    pub data_ptrs: [*mut u8; 4],
    pub data_linesizes: [c_int; 4],
    pub pts: i64,
    pub duration: i64,
    pub width: i32,
    pub height: i32,
}
pub struct Scaler {
    src_pix_fmt: AVPixelFormat,
    dst_pix_fmt: AVPixelFormat,
    context: *mut SwsContext,
    pub producer: Producer<ScalerOutput>
}
impl Scaler {
    pub fn new(source_stream: &AVStream , dst_pix_fmt: AVPixelFormat) -> Self{
        let codec_par = unsafe { &*source_stream.codecpar };

        let context = unsafe {
            sws_getContext(
                codec_par.width,
                codec_par.height,
                codec_par.format,
                codec_par.width,
                codec_par.height,
                dst_pix_fmt,
                SWS_BILINEAR as c_int,
                null_mut(), 
                null_mut(), 
                null_mut()
            )
        };

        if context.is_null() {
            panic!("Failed to create swScale context");
        }

        return Scaler {
            src_pix_fmt: codec_par.format,
            dst_pix_fmt: dst_pix_fmt,
            context,
            producer: Producer::new(),
        }
    }
    fn convert(&mut self, frame: *mut AVFrame) -> Result<(), String> {
        let frame = unsafe {&mut *frame};

        let mut dst_data: [*mut u8; 4] = [null_mut(); 4];
        let mut dst_linesizes: [c_int; 4] = [0; 4];

        let output_frame = unsafe {av_image_alloc(dst_data.as_mut_ptr(), dst_linesizes.as_mut_ptr(), frame.width, frame.height, self.dst_pix_fmt, 1)};
        if output_frame < 0 {
            return Err(String::from("Failed to allocate output frame in swScale"));
        }

        unsafe {
            sws_scale(self.context, frame.data.as_ptr() as *const *const u8, frame.linesize.as_mut_ptr(), 0, frame.height, dst_data.as_mut_ptr(), dst_linesizes.as_mut_ptr());
        }

        let output = ScalerOutput { data_ptrs: dst_data, data_linesizes: dst_linesizes , pts: frame.pts, duration: frame.duration, width: frame.width, height: frame.height };
        self.producer.produce(output);
        return Ok(());
    }
}

impl Consumer<WrappedAVFrame> for Scaler {
    fn consume(&mut self, to_consume: WrappedAVFrame) {
        self.convert(to_consume.0).unwrap();
    }
}
