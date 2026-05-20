use std::{ptr::null_mut, sync::mpsc::{Receiver, Sender}};
use rusty_ffmpeg::ffi::{AV_PIX_FMT_RGB24, AV_PIX_FMT_RGB32, AVCodec, AVCodecContext, AVCodecParameters, AVFrame, AVPacket, AVStream, av_frame_alloc, avcodec_alloc_context3, avcodec_find_decoder, avcodec_open2, avcodec_parameters_to_context, avcodec_receive_frame, avcodec_send_packet};

use crate::{consumer::Consumer, producer::Producer, wrappers::{WrappedAVFrame, WrappedAVPacket}};

pub struct FFmpegDecoder {
    context: *mut AVCodecContext,
    decoder: *const AVCodec,
    pub producer: Producer<WrappedAVFrame>,
    stream_index: i32,
}

impl FFmpegDecoder {
    pub fn new(stream: &AVStream) -> Self {
        let (decoder_ctx, dec) = match Self::open(stream) {
            Ok(v) => v,
            Err(e) => panic!("Unable to open decoder: {}", e)
        };

        return Self {
            context: decoder_ctx,
            decoder: dec,
            producer: Producer::new(),
            stream_index: stream.index,
        }
    }

    fn open(stream: &AVStream) -> Result<(*mut AVCodecContext, *const AVCodec), String> {
        // TODO: Should refactor this to be its own function, to allow finding audio codecs & streams
        let (decoder_ctx, dec) = unsafe {
            let codec_par = &*stream.codecpar;

            let dec = avcodec_find_decoder((*stream.codecpar).codec_id);
            if dec.is_null() {
                return Err(format!("Unable to find decoder"));
            }

            let dec_ctx = avcodec_alloc_context3(dec);
            if dec_ctx.is_null() {
                panic!("avformat_alloc_context failed");
            }

            let res = avcodec_parameters_to_context(dec_ctx, codec_par);
            if res < 0 {
                return Err(format!("Unable copy codecPar to codecContext with code: {}", res));
            }

            let res = avcodec_open2(dec_ctx, dec, null_mut());
            if res < 0 {
                return Err(format!("Unable to open Codec with code: {}", res));
            }

            (dec_ctx, dec)
        };

        return Ok((decoder_ctx, dec));
    }
    fn decode_packet(&mut self, to_decode: &WrappedAVPacket) {
        unsafe {
            let res = avcodec_send_packet(self.context, to_decode.0);
            if res < 0 {
                println!("Error sending packet: {}", res);
                return;
            }

            let mut frame = av_frame_alloc();
            if frame.is_null() {
                panic!("Failed to alloc avFrame");
            }

            while avcodec_receive_frame(self.context, frame) >= 0 {
                self.producer.produce(WrappedAVFrame(frame));
                frame = av_frame_alloc();
            }
        }
    }
    //TODO: close with flush here
}

impl Consumer<WrappedAVPacket> for FFmpegDecoder {
    fn consume(&mut self, to_consume: WrappedAVPacket) {
        let idx = unsafe { (*to_consume.0).stream_index };
        if idx == self.stream_index {
            self.decode_packet(&to_consume);
        }
    }
}
