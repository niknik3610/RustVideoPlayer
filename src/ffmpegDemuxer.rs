use std::{ffi::CString, ptr::{null, null_mut}};

use rusty_ffmpeg::ffi::{AVFormatContext, AVPacket, av_packet_alloc, av_read_frame, avformat_alloc_context, avformat_find_stream_info, avformat_open_input};
use crate::{producer::{self, Producer}, wrappers::WrappedAVPacket};

struct FFmpegDemuxer {
    file_uri: String,
    context: AVFormatContext,
    producer: Producer<WrappedAVPacket>
}

impl FFmpegDemuxer {
    pub fn new(file_uri: String, producer_channel: Option<std::sync::mpsc::Sender<AVPacket>>) -> Self {
        let context = match Self::open(&file_uri) {
            Ok(v) => v,
            Err(e) => panic!("Unable to open file: {}", e)
        };

        return FFmpegDemuxer {
            file_uri: file_uri, 
            context: context, 
            producer: Producer::new()
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
    pub fn run(&mut self) -> Result<(), String> {
        unsafe {
            let mut packet = av_packet_alloc();
            if packet.is_null() {
                return panic!("Unable to allocate a packet");
            }

            while av_read_frame(&mut self.context, packet) >= 0 {
                let wrapped_packet = WrappedAVPacket(packet);
                self.producer.produce(wrapped_packet); 
                packet = av_packet_alloc();
            }


        }

        return Ok(())
    }
}
