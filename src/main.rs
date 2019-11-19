/*
    TODO:
    Next step is playing sound realtime on laptop, while printing transients

    No reason for transient level to be i32. Consider splitting into u8 and u32, even if it makes reading more complicated
    u32 might make more sense than i32 too.

    IDEAS:
    Maybe a "best guess" based algorithm, where it cycles through bpms to see which best fits the transients
    Idea for a pattern: If there's no transients for a while, slowly sweep up when short-time RMS increases
    parallelize by not loading the entire wav at once and just use the samples iterator?


*/
/// This project is meant for opening a wave file and calculating its tempo. Meant as a prototype
//cpal, or cross-platform audio library handles audio playback
extern crate anyhow;
extern crate cpal;
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
// hound is a wav file reading library
extern crate hound;
// extern crate rppal; <- rppal only works on linux
// use rppal::uart::{Parity, Uart};
use std::fs;
use std::path::Path;

mod dmx;
mod sound_file;

fn main() -> Result<(), anyhow::Error> {
    // cpal setup
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let format = device.default_output_format()?;
    let event_loop = host.event_loop();
    let stream_id = event_loop.build_output_stream(&device, &format)?;
    event_loop.play_stream(stream_id.clone())?;
    let sample_rate = format.sample_rate.0 as f32;
    if sample_rate != 44100. {
        eprintln!("wrong sample rate boy. it has to be 44100 and yours is {}", sample_rate);
        return Ok(()); // FIXME: This really shouldn't be return Ok(()), but I can't be bothered figuring out anyhow right now
    }
    println!("format sample rate is {}", sample_rate);
    // SoundFile setup
    let mut sound = sound_file::SoundFile::default();
    // grabbing all files in Songs and adding their paths to a vector
    let path = Path::new(r"./Songs"); //FIXME: this \ should be / on linux >:(
    let mut entries: Vec<std::path::PathBuf> = vec![];
    println!("which of the songs do you want to play? Write a number.");
    for entry in fs::read_dir(path).expect("Unable to list") {
        entries.push(entry.expect("unable to get entry").path());
    }
    for (i, entry) in entries.iter().enumerate() {
        println!("{}: {}", i, entry.display());
    }
    //choose a sound:
    let mut n = String::new();
    std::io::stdin()
        .read_line(&mut n)
        .expect("failed to read input.");
    let n: usize = n.trim().parse().expect("invalid input");
    // let n = 5;
    sound.load_sound(entries[n].clone());
    if sound.search_for_file() != true {
        sound.detect_transients_by_rms();
        sound._bpm_by_guess();
        sound.generate_analysis_file();
        println!("BPM is {}", sound.analysis.get_tempo());
    // println!("{:?}", sound.analysis.rhythm);
    } else {
        println!("Analysis file already exists boy");
        sound.read_analysis_file();
        // sound._bpm_by_guess();
        println!("BPM is {}", sound.analysis.get_tempo());
        // println!("{:?}", sound.analysis.rhythm);
    }
    println!("playing song: {:?}", entries[n]);
    let mut sample_iter = sound.samples.iter();
    let mut transient_iter = 0;
    let mut curr_trans = 0;
    
    // event_loop.run takes control of the playback thread and handles it
    event_loop.run(move |id, result| {
        let data = match result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("an error occurred on stream {:?}: {}", id, err);
                return;
            }
        };
        // println!("got here at least lol");
        match data {
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = sample_iter.next();
                    if value == None {
                        println!("song over");
                        break; // FIXME: How do we get from here to "choose new song"?
                    } else {
                        transient_iter += 1;
                        // Below doesn't work because it make event_loop take ownership of sound
                        // if transient_iter >= sound.analysis.rhythm[curr_trans * 2] as usize {
                        //     // Send uart message with sound.analysis.rhythm[curr_trans * 2 - 1];
                        //     print!(
                        //         "now there's a transient with volume {} ",
                        //         sound.analysis.rhythm[curr_trans * 2 + 1] as f32 / std::i32::MAX as f32
                        //     ); // FIXME: Replace with what we actually send out sound with
                        //     transient_iter = 0;
                        //     curr_trans += 1;
                        // }

                        for out in sample.iter_mut() {
                            // println!("playing sample");
                            *out = *value.unwrap();
                        }
                    }
                }
            }
            _ => {
                println!("can't play back. sample type not supported");
                ()
            }
        }
    });
    // for x in sound.samples.iter() {
    //     // Send out sample
    //     // println!("playing sample: {}", x); //FIXME: Replace with what we actually send out sound with
    //     transient_iter += 1;
    //     if curr_trans * 2 + 1 < sound.analysis.rhythm.len() {
    //         // Bounds checking. can this be avoided?
    //         if transient_iter >= sound.analysis.rhythm[curr_trans * 2] as usize {
    //             // Send uart message with sound.analysis.rhythm[curr_trans * 2 - 1];
    //             print!(
    //                 "now there's a transient with volume {} ",
    //                 sound.analysis.rhythm[curr_trans * 2 + 1] as f32 / std::i32::MAX as f32
    //             ); // FIXME: Replace with what we actually send out sound with
    //             transient_iter = 0;
    //             curr_trans += 1;
    //         }
    //     }
    // }
}
