use std::env;

pub mod consumer;
pub mod producer;
pub mod ffmpegDemuxer;
pub mod ffmpegDecoder;
pub mod wrappers;
pub mod frameWriter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("You must include a file to play as a commandline arg");
        return;
    }

    //For now just assume the first input is a file, if not we go kaboom anyway
    let file = args[1].clone();
    let mut demuxer = ffmpegDemuxer::FFmpegDemuxer::new(file);

    let v_stream = demuxer.get_video_stream().expect("Expected a video stream from demuxer");
    let mut decoder = ffmpegDecoder::FFmpegDecoder::new(v_stream);

    let frame_writer = frameWriter::FrameWriter::new();


    //This has messed up ownership semantics, probably Box<&Consumer> is the way to go
    decoder.producer.add_consumer(Box::new(frame_writer));
    demuxer.producer.add_consumer(Box::new(decoder));
    demuxer.run();
}
