/*
    TODO:
    Next step is playing sound realtime on laptop, while printing transients

    No reason for transient level to be i32. Consider splitting into u8 and u32, even if it makes reading more complicated
    u32 might make more sense than i32 too.

    FIXME: Skift til active-low


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

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{fs, thread, time};
mod dmx;
mod sound_file;
mod util;
// extern crate rppal; // <- rppal only works on linux
// use rppal::gpio::Gpio;
// use rppal::uart::{Parity, Uart};
use std::sync::atomic::{AtomicBool, Ordering};
#[allow(dead_code)]
fn main() -> Result<(), anyhow::Error> {
    // cpal setup
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let format = device.default_output_format()?;
    let event_loop = host.event_loop();
    // if we were to add a microphone, we could also build an input stream
    let stream_id = event_loop.build_output_stream(&device, &format)?;
    // This might be a problem, since we haven't added submitted any data to the stream, but it seems to work
    event_loop.play_stream(stream_id.clone())?;
    let sample_rate = format.sample_rate.0 as f32;
    if sample_rate != 44100. {
        eprintln!(
            "wrong sample rate boy. it has to be 44100 and yours is {}",
            sample_rate
        );
        return Ok(()); // FIXME: This really shouldn't be return Ok(()), but I can't be bothered figuring out anyhow right now
    }
    println!("format sample rate is {}", sample_rate);
    // SoundFile setup
    let mut sound = sound_file::SoundFile::default();
    // grabbing all files in Songs and adding their paths to a vector
    let path = Path::new(r"./Songs");
    let mut entries: Vec<std::path::PathBuf> = vec![];
    for entry in fs::read_dir(path).expect("Unable to list") {
        entries.push(entry.expect("unable to get entry").path());
    }
    for (i, entry) in entries.iter().enumerate() {
        sound.set_file_name(entries[i].clone());
        if !sound.search_for_file() {
            println!("please wait. analysing {}", entry.display());
            sound.load_sound(entries[i].clone());
            sound.detect_transients_by_rms();
            // sound._bpm_by_guess();
            sound._bpm_in_frames();
            sound.generate_analysis_file();
            println!("BPM is {}", sound.analysis.get_tempo());
        }
    }
    println!("which of the songs do you want to play? Write a number.");
    for (i, entry) in entries.iter().enumerate() {
        println!("{}: {}", i, entry.display());
    }
    //choose a sound: below is for choosing with command line
    // let mut n = String::new();
    // std::io::stdin()
    //     .read_line(&mut n)
    //     .expect("failed to read input.");
    // let n: usize = n.trim().parse().expect("invalid input");
    // let mut n : usize = 0;
    // creating atomic reference counting pointers for sharing data between threads
    // these 3 are for choosing sound. plus_arc and minus_arc lets plus and minus buttons change n
    let n = Arc::new(util::AtomicUsize::new(4));
    let plus_arc = Arc::clone(&n);
    let minus_arc = Arc::clone(&n);
    // these 3 are to see if playback can start/continue
    let go_ahead = Arc::new(AtomicBool::new(false));
    let play_arc = Arc::clone(&go_ahead);
    let stop_arc = Arc::clone(&go_ahead);

    // for passing and editing sound so we can change sound
    let sound_arc = Arc::new(Mutex::new(sound));
    let sound_arc_playback = Arc::clone(&sound_arc);

    // for refreshing the iterater when starting a new song
    let play_flag_arc1 = Arc::new(AtomicBool::new(false));
    let play_flag_arc2 = Arc::clone(&play_flag_arc1);

    // for cloning samples vector and creating an iterator over it
    // let sound_guard =  &sound_arc_playback.lock().unwrap();
    // let samples_from_arc = sound_guard.samples.clone();
    // // dropping the guard to unlock mutex:
    // drop(sound_guard);
    // let mut sample_iter = samples_from_arc.iter();
    let dummy_vec = vec![0.,0.,0.,0.];
    let mut sample_iter_arc = Mutex::new(dummy_vec.iter());
    
    // Spawning button threads: FIXME: plus and minus button should only trigger on rising edge
    // let _plus_button_thread = thread::spawn(move || {
    //     let gpio = Gpio::new().unwrap();
    //     let mut pin = gpio.get(25).unwrap().into_input_pullup();
    //     pin.set_reset_on_drop(false);
    //     loop {
    //         if pin.read() == rppal::gpio::Level::Low {
    //             plus_arc.set(plus_arc.get() + 1);
    //         }
    //     }
    // });
    // let _minus_button_thread = thread::spawn(move || {
    //     let gpio = Gpio::new().unwrap();
    //     let mut pin = gpio.get(24).unwrap().into_input_pullup(); 
    //     pin.set_reset_on_drop(false);
    //     loop {
    //         if pin.read() == rppal::gpio::Level::Low {
    //             minus_arc.set(minus_arc.get() - 1);
    //         }
    //     }
    // });
    let _play_button_thread = thread::spawn(move || {
        // let gpio = Gpio::new().unwrap();
        // let mut pin = gpio.get().unwrap().into_input_pullup();
        // pin.set_reset_on_drop(false);
        loop {
            // if pin.read() == rppal::gpio::Level::Low {
                println!("true stored in play arc");
                // sound_arc.lock().unwrap().load_sound(entries[n.get()].clone());
                // println!("sound loaded with sound_arc");
                if plus_arc.get() == 0 {
                    plus_arc.set(2);
                }
                else if plus_arc.get() == 2 {
                    plus_arc.set(4);
                }
                else {
                    plus_arc.set(0);
                }
                
                let mut lock = sound_arc.try_lock();
                if let Ok(ref mut mutex) = lock {
                    mutex.load_sound(entries[n.get()].clone());
                    println!("sound loaded with sound_arc");
                    mutex.read_analysis_file();
                    println!("analysis file read");
                } else {
                    println!("try_lock for loading sound failed");
                }
                drop(lock);
                // sound_arc.lock().unwrap().read_analysis_file();
                // println!("analysis file read");
                //raise flag to update the iterator
                play_arc.store(true, Ordering::Relaxed);
                play_flag_arc1.store(true, Ordering::Relaxed);
                println!("playing song: {:?}", entries[n.get()]);
                thread::sleep(time::Duration::from_millis(10000));
            // }
        }
    });
    let _stop_button_thread = thread::spawn(move || {
        // let gpio = Gpio::new().unwrap();
        // let mut pin = gpio.get(23).unwrap().into_input_pullup(); 
        // pin.set_reset_on_drop(false);
        thread::sleep(time::Duration::from_millis(5000));
        loop {
            // if pin.read() == rppal::gpio::Level::Low {
                println!("playback stopped");
                stop_arc.store(false, Ordering::Relaxed);
                //FIXME: How do we use this to end the playback thread?
            // }
            thread::sleep(time::Duration::from_millis(10000));

        }
    });
    while !go_ahead.load(Ordering::Relaxed) {
        thread::sleep(time::Duration::from_millis(250));
    }
    // sound.load_sound(entries[n.get()].clone());
    // sound.read_analysis_file();
    // let playback = Arc::clone(&sound.samples);
    // let samples_arc = Arc::new(&sound.samples);
    // since samples_from_arc is a reference to
    
    let mut transient_iter = 0;
    // let mut curr_trans = 0;
    // event_loop.run takes control of the main thread and turns it into a playback thread
    event_loop.run(move |id, result| {
        // should only happen exactly when playback happens
        if play_flag_arc2.load(Ordering::Relaxed) {
            println!("updating iter");
            // for cloning samples vector and creating an iterator over it
            let sound_guard =  sound_arc_playback.lock().unwrap();
            let to_be_played = sound_guard.samples.clone();
            // dropping the guard to unlock mutex:
            drop(sound_guard);
            let iter_guard = sample_iter_arc.get_mut().unwrap();
            *iter_guard = to_be_played.iter();                        
            // = samples_from_arc.iter();
            play_flag_arc2.store(false, Ordering::Relaxed);
        } 
        // we need to do a refresh of the iterator if playback is going to start again.
        // maybe some flag?
        else {
            let data = match result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", id, err);
                    return;
                }
            };
            // This match could be used to support outher bit rates
            match data {
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        if !go_ahead.load(Ordering::Relaxed) {
                            // this should stop it from playing after pressing stop.
                            // but how do we make play load a sound, then?
                            for out in sample.iter_mut() {
                                *out = 0.;
                            }
                        }
                        else {
                            // let value = sample_iter.next();
                            let value = Some(&0.);
                            if value == None {
                                // print!("song over");
                                break; // FIXME: How do we get from here to "choose new song"?
                            } else {
                                
                                // Below doesn't work because it make event_loop take ownership of sound
                                // if transient_iter >= sound.analysis.rhythm[curr_trans * 2] as usize
                                //     && curr_trans * 2 + 1 < sound.analysis.rhythm.len()
                                // {
                                //     // Send uart message with sound.analysis.rhythm[curr_trans * 2 - 1];
                                //     println!(
                                //         "now there's a transient with volume {} ",
                                //         sound.analysis.rhythm[curr_trans * 2 + 1] as f32
                                //             / std::i32::MAX as f32
                                //     ); // FIXME: Replace with what we actually send out sound with
                                //     transient_iter = 0;
                                //     curr_trans += 1;
                                // }
                                // transient_iter += 1;
                                for out in sample.iter_mut() {
                                    // println!("playing sample");
                                    *out = *value.unwrap();
                                }
                            }
                        }
                    }
                }
                _ => {
                    println!("can't play back. sample type not supported");
                    ()
                }
            }
        }
    });
    // Ok(())
}
