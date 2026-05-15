use std::{ffi::CStr, ptr::null};

use rusty_ffmpeg::ffi::{AVFormatContext, AVPacket, avformat_alloc_context, avformat_open_input};
use crate::{ffmpegDemuxer, producer};

struct FFmpegDemuxer {
    file_uri: String,
    context: AVFormatContext,
    producer_channel: Option<std::sync::mpsc::Sender<AVPacket>>
}

impl FFmpegDemuxer {
    pub fn new(file_uri: String) -> Self {
        let context = ffmpegDemuxer::open();
        return FFmpegDemuxer {
            file_uri: file_uri, context: context, producer_channel: Option::None
        };
    }
}

impl producer::Producer<AVPacket> for FFmpegDemuxer {
    fn produce(&self) {

    }

    fn set_channel(&mut self, channel: std::sync::mpsc::Sender<AVPacket>) {
        self.producer_channel = channel;
    }
}


fn open(file_uri: String) -> AVFormatContext {
    unsafe {
        let mut formatContext = avformat_alloc_context();
        let bla = c"hello";
        avformat_open_input(&mut formatContext, bla , null(), null());
    }
}
