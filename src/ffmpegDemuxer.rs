use std::{ffi::{CStr, CString}, ptr::{null, null_mut}};

use rusty_ffmpeg::ffi::{AVCodecContext, AVFormatContext, AVMediaType, AVPacket, AVStream, av_find_best_stream, avcodec_alloc_context3, avcodec_find_decoder, avcodec_open2, avcodec_parameters_copy, avcodec_parameters_to_context, avformat_alloc_context, avformat_find_stream_info, avformat_open_input};
use crate::{ffmpegDemuxer, producer};

struct FFmpegDemuxer {
    file_uri: String,
    context: AVFormatContext,
    producer_channel: Option<std::sync::mpsc::Sender<AVPacket>>
}

impl FFmpegDemuxer {
    pub fn new(file_uri: String, producer_channel: Option<std::sync::mpsc::Sender<AVPacket>>) -> Self {
        let context = match Self::open(&file_uri) {
            Ok(v) => v,
            Err(e) => panic!("Unable to open file: {}", e)
        };

        return FFmpegDemuxer {
            file_uri: file_uri, context: context, producer_channel: producer_channel
        };
    }
    fn open(file_uri: &String) -> Result<AVFormatContext, String> {
        let format_ctx = unsafe {
            let mut format_context = avformat_alloc_context();
            if format_context.is_null() {
                panic!("avformat_alloc_context failed");
            }

            let uri = CString::new(file_uri.as_str()).expect("Failed to alloc c_str from URI");

            let mut res = avformat_open_input(&mut format_context, uri.as_ptr(), null(), null_mut());
            if res != 0 {
                return Err(format!("Unable to open Input with Error: {}", res));
            }

            res = avformat_find_stream_info(format_context, null_mut());
            if res != 0 {
                return Err(format!("Unable to find stream info with code: {}", res));
            }

            *format_context
        };

        return Ok(format_ctx);
    }


}

impl producer::Producer<AVPacket> for FFmpegDemuxer {
    fn produce(&self) {

    }

    fn set_channel(&mut self, channel: std::sync::mpsc::Sender<AVPacket>) {
        self.producer_channel = Some(channel);
    }
}

