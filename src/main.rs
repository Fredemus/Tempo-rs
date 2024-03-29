/*
    TODO: top is most important
    Make buttons trigger only on rising edge
    Best guess bpm algo
    Program crashes in bpm_by_frames if the file doesn't have any transients
    IDEAS:
    Maybe a "best guess" based algorithm, where it cycles through bpms to see which best fits the transients

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
mod rgb;
// Nicho's crates:
use std::collections::VecDeque;
extern crate rustfft;
use rustfft::num_complex::Complex;

extern crate rppal; // <- rppal only works on linux
use rppal::gpio::Gpio;
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
    // SoundFile setup
    let mut sound = sound_file::SoundFile::default();

    // grabbing all files in Songs and adding their paths to a vector
    let path = Path::new(r"./Songs");
    // create a vector of file paths
    let mut entries: Vec<std::path::PathBuf> = vec![];
    // add every file path in folder ./Songs to entries
    for entry in fs::read_dir(path).expect("Unable to list") {
        entries.push(entry.expect("unable to get entry").path());
    }
    // check for analysis file on each
    for (i, entry) in entries.iter().enumerate() {
        sound.set_file_name(entries[i].clone());
        if !sound.search_for_file() {
            println!("please wait. analysing {}", entry.display());
            sound.load_sound(entries[i].clone());
            sound._detect_transients_by_stft();
            // sound._bpm_by_guess();
            sound._bpm_in_frames();
            sound.generate_analysis_file();
            // println!("BPM is {}", sound.analysis.get_tempo());
        }
    }
    println!("which of the songs do you want to play?");
    for (i, entry) in entries.iter().enumerate() {
        println!("{}: {}", i, entry.display());
    }
    // choose a sound: below is for choosing with command line
    // let mut n = String::new();
    // std::io::stdin()
    //     .read_line(&mut n)
    //     .expect("failed to read input.");
    // let n: usize = n.trim().parse().expect("invalid input");
    // let mut n : usize = 0;

    // creating atomic reference counting pointers for safely sharing data between threads
    // these 3 are for choosing sound. plus_arc and minus_arc lets plus and minus buttons change n
    let n = Arc::new(util::AtomicUsize::new(4));
    let plus_arc = Arc::clone(&n);
    let minus_arc = Arc::clone(&n);
    // these 3 are to see if playback can start/continue
    let go_ahead = Arc::new(util::AtomicBool::new(false));
    let play_arc = Arc::clone(&go_ahead);
    let stop_arc = Arc::clone(&go_ahead);

    // for passing and editing the SoundFile struct so we can change song
    let sound_arc = Arc::new(Mutex::new(sound));
    let sound_arc_playback = Arc::clone(&sound_arc);

    // for refreshing the iterater when starting a new song
    let play_flag_arc1 = Arc::new(util::AtomicBool::new(false));
    let play_flag_arc2 = Arc::clone(&play_flag_arc1);
    // Creation of threads
    let _plus_button_thread = thread::spawn(move || {
        // GPIO pin setup
        let gpio = Gpio::new().unwrap();
        let mut pin = gpio.get(25).unwrap().into_input_pullup();
        pin.set_reset_on_drop(false);
        loop {
            // If button is pressed
            if pin.read() == rppal::gpio::Level::Low {
                plus_arc.set(plus_arc.get() + 1);
                thread::sleep(time::Duration::from_millis(250));
            }
        }
    });
    let _minus_button_thread = thread::spawn(move || {
        let gpio = Gpio::new().unwrap();
        let mut pin = gpio.get(24).unwrap().into_input_pullup();
        pin.set_reset_on_drop(false);
        loop {
            if pin.read() == rppal::gpio::Level::Low {
                minus_arc.set(minus_arc.get() - 1);
                thread::sleep(time::Duration::from_millis(250));
            }
        }
    });
    let _play_button_thread = thread::spawn(move || {
        let gpio = Gpio::new().unwrap();
        let mut pin = gpio.get(18).unwrap().into_input_pullup();
        pin.set_reset_on_drop(false);
        loop {
            if pin.read() == rppal::gpio::Level::Low {
                // Lock the mutex connected to sound_arc
                let mut lock = sound_arc.try_lock();
                // If lock was successful
                if let Ok(ref mut mutex) = lock {
                    mutex.load_sound(entries[n.get()].clone());
                    println!("sound loaded with sound_arc");
                    mutex.read_analysis_file();
                    println!("analysis file read");
                    println!("tempo is {}", mutex.analysis.get_tempo());
                } else {
                    println!("try_lock for loading sound failed");
                }
                drop(lock);
                //raise flag to update the iterator in main thread
                play_arc.set(true);
                play_flag_arc1.set(true);
                // printing what song is being played
                println!("playing song: {:?}", entries[n.get()]);
                thread::sleep(time::Duration::from_millis(250));
            }
        }
    });
    let _stop_button_thread = thread::spawn(move || {
        let gpio = Gpio::new().unwrap();
        let mut pin = gpio.get(23).unwrap().into_input_pullup();
        pin.set_reset_on_drop(false);
        loop {
            if pin.read() == rppal::gpio::Level::Low {
                stop_arc.set(false);
                println!("playback stopped");
                thread::sleep(time::Duration::from_millis(250));
            }
        }
    });
    let mut transient_iter = 0;
    let mut curr_trans = 0;
    // rhythm and sample_iter is to bring sound's samples and transient information properly into the scope of event_loop
    let mut rhythm: Vec<i32> = vec![0];
    let dummy_vec = vec![0.];
    let mut sample_iter = dummy_vec.into_iter();
    let mut dmx = dmx::DMX::default();
    // Nicho's RGB variables
    let mut fft_deque: VecDeque<Complex<f32>> = VecDeque::new();
    // this for loop defines the length of the deque / fourier transform
    for _i in 0..1536 {
        fft_deque.push_back(num_complex::Complex::new(0., 0.));
    }
    // count is used to know when to send color messages 
    let mut count = 0;
    let mut rgb = rgb::RGB::default();
    // event_loop.run takes control of the main thread and turns it into a playback thread
    event_loop.run(move |id, result| {
        // this if statement evaluates to true when a new song is being played
        // lets us update sample_iter and and rhythm safely
        if play_flag_arc2.get() {
            println!("updating iter");
            // for cloning samples vector and creating an iterator over it
            let sound_guard = sound_arc_playback.lock().unwrap();
            let samples = sound_guard.samples.clone();
            rhythm = sound_guard.analysis.rhythm.clone();
            // dropping the guard to unlock mutex:
            drop(sound_guard);
            sample_iter = samples.into_iter();
            transient_iter = 0;
            curr_trans = 0;
            play_flag_arc2.store(false, Ordering::Relaxed);
        } else {
            sample_iter.next();
            let data = match result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", id, err);
                    return;
                }
            };
            // This match could be used to support other bit rates
            match data {
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        if !go_ahead.get() {
                            for out in sample.iter_mut() {
                                *out = 0.;
                            }
                        } else {
                            let value = sample_iter.next();
                            if value == None {
                                // print!("song over");
                                break;
                            } else {
                                // move new sample into the deque
                                fft_deque.pop_front();
                                fft_deque.push_back(num::Complex::new(value.unwrap(), 0.));
                                if count == 4410 {
                                    // Main RGB code
                                    rgb.rgb_fft(Vec::from(fft_deque.clone()));
                                    count = 0;
                                    // Write output
                                    dmx.change_color(rgb.get_bass(), rgb.get_mid(), rgb.get_high());
                                    count = 0;
                                }
                                // Sending a move message on transient
                                if curr_trans * 2 + 1 < rhythm.len() {
                                    if transient_iter >= rhythm[curr_trans * 2] as usize {
                                        dmx.simple_move(rhythm[curr_trans * 2 + 1]);
                                        transient_iter = 0;
                                        curr_trans += 1;
                                        println!("transient number {}", curr_trans);
                                    }
                                    // Moving back to previous position halfway to next transient
                                    else if transient_iter
                                        == (rhythm[curr_trans * 2] / 2) as usize
                                    {
                                        dmx.simple_move_back();
                                    }
                                }
                                transient_iter += 1;
                                count += 1;
                                // sending the same sample out to all channels (2 if stereo)
                                for out in sample.iter_mut() {
                                    // println!("playing sample");
                                    *out = value.unwrap();
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
