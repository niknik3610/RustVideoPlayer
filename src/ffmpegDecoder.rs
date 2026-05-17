use std::{ptr::null_mut, sync::mpsc::{Receiver, Sender}};
use rusty_ffmpeg::ffi::{AVCodec, AVCodecContext, AVCodecParameters, AVFrame, AVPacket, AVStream, avcodec_alloc_context3, avcodec_find_decoder, avcodec_open2, avcodec_parameters_to_context};

use crate::{consumer::Consumer, producer::Producer};


struct FFmpegDecoder {
    context: AVCodecContext,
    decoder: AVCodec,
    producer: Producer<AVFrame>
}

impl FFmpegDecoder {
    pub fn new(stream: &AVStream, codec_par: &AVCodecParameters, consumer_channel: Option<Receiver<AVPacket>>, producer_channel: Option<Sender<AVFrame>>) -> Self {
        let (decoder_ctx, dec) = match Self::open(stream, codec_par) {
            Ok(v) => v,
            Err(e) => panic!("Unable to open decoder: {}", e)
        };

        return Self {
            context: decoder_ctx,
            decoder: dec,
            producer: Producer::new()
        }
    }

    fn open(stream: &AVStream, codec_par: &AVCodecParameters) -> Result<(AVCodecContext, AVCodec), String> {
        // TODO: Should refactor this to be its own function, to allow finding audio codecs & streams
        let (decoder_ctx, dec) = unsafe {
            let dec = avcodec_find_decoder((*stream.codecpar).codec_id);
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

            (*dec_ctx, *dec)
        };

        return Ok((decoder_ctx, dec));
    }
}

impl Consumer<AVPacket> for FFmpegDecoder {
    fn consume(&self, to_consume: &AVPacket) {
        todo!()
    }
}
