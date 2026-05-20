use std::env;

use rusty_ffmpeg::ffi::AV_PIX_FMT_RGB24;

use crate::sw_scale::Scaler;

pub mod consumer;
pub mod producer;
pub mod ffmpegDemuxer;
pub mod ffmpegDecoder;
pub mod wrappers;
pub mod frameWriter;
pub mod open_gl_renderer;
pub mod sw_scale;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("You must include a file to play as a commandline arg");
        return;
    }

    // For now just assume the first input is a file, if not we go kaboom anyway
    let file = args[1].clone();
    let mut demuxer = ffmpegDemuxer::FFmpegDemuxer::new(file);
    let v_stream = demuxer.get_video_stream().expect("Expected a video stream from demuxer");

    let mut decoder = ffmpegDecoder::FFmpegDecoder::new(v_stream);
    let mut scaler = Scaler::new(v_stream, AV_PIX_FMT_RGB24);
    let frame_writer = frameWriter::FrameWriter::new();


    //This has messed up ownership semantics, probably Box<&Consumer> is the way to go, but that
    //has some lifetime stuff i dont wanna deal with
    scaler.producer.add_consumer(Box::new(frame_writer));  
    decoder.producer.add_consumer(Box::new(scaler));
    demuxer.producer.add_consumer(Box::new(decoder));

    // let height = 600; let width = 800;
    // let mut renderer = open_gl_renderer::GLRenderer::new(width, height, 30);

    demuxer.run().unwrap();
}
