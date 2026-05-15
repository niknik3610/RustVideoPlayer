use std::{ffi::{CStr, CString}, ptr::{null, null_mut}};

use rusty_ffmpeg::ffi::{AVFormatContext, AVMediaType, AVPacket, av_find_best_stream, avcodec_alloc_context3, avcodec_find_decoder, avcodec_open2, avcodec_parameters_copy, avcodec_parameters_to_context, avformat_alloc_context, avformat_find_stream_info, avformat_open_input};
use crate::{ffmpegDemuxer, producer};

struct FFmpegDemuxer {
    file_uri: String,
    context: AVFormatContext,
    producer_channel: Option<std::sync::mpsc::Sender<AVPacket>>
}

impl FFmpegDemuxer {
    pub fn new(file_uri: String) -> Self {
        let context = ffmpegDemuxer::open(&file_uri);
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


fn open(file_uri: &String) -> Result<AVFormatContext, String> {
    let mut format_ctx = unsafe {
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

    let video_stream_idx;

    // TODO: Should refactor this to be its own function, to allow finding audio codecs & streams
    let decoder_ctx = unsafe {
        video_stream_idx = av_find_best_stream(&mut format_ctx, rusty_ffmpeg::ffi::AVMEDIA_TYPE_VIDEO, -1, -1, null_mut(), 0);
        if video_stream_idx < 0 {
            return Err(format!("Unable to find best stream with code: {}", video_stream_idx));
        }
        let stream = *format_ctx.streams.add(video_stream_idx as usize);

        let dec = avcodec_find_decoder((*(*stream).codecpar).codec_id);
        if dec.is_null() {
            return Err(format!("Unable to find decoder"));
        }

        let dec_ctx = avcodec_alloc_context3(dec);
        if dec_ctx.is_null() {
            panic!("avformat_alloc_context failed");
        }

        let res = avcodec_parameters_to_context(dec_ctx, (*(stream)).codecpar);
        if res < 0 {
            return Err(format!("Unable copy codecPar to codecContext with code: {}", res));
        }

        let res = avcodec_open2(dec_ctx, dec, null_mut());
        if res < 0 {
            return Err(format!("Unable to open Codec with code: {}", res));
        }
    };

    return Ok(format_ctx);
}
