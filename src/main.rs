/*
    TODO: top is most important
    Make buttons trigger only on rising edge
    improved transient formula
    Best guess bpm algo

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

// Nicho's crates:
use std::collections::VecDeque;
extern crate basic_dsp_vector;
extern crate rustfft;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FFTplanner;

extern crate rppal; // <- rppal only works on linux
use rppal::gpio::Gpio;
use rppal::uart::{Parity, Uart};
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

    // creating atomic reference counting pointers for safely sharing data between threads
    // these 3 are for choosing sound. plus_arc and minus_arc lets plus and minus buttons change n
    let n = Arc::new(util::AtomicUsize::new(0));
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

    // Spawning button threads: FIXME: plus and minus button should only trigger on rising edge
    // let _plus_button_thread = thread::spawn(move || {
    //     let gpio = Gpio::new().unwrap();
    //     let mut pin = gpio.get(25).unwrap().into_input_pullup();
    //     pin.set_reset_on_drop(false);
    //     loop {
    //         if pin.read() == rppal::gpio::Level::Low {
    //             plus_arc.set(plus_arc.get() + 1);
    //             thread::sleep(time::Duration::from_millis(250)); //FIXME: Not needed if we get trigger on edge working
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
    //             thread::sleep(time::Duration::from_millis(250)); //FIXME: Not needed if we get trigger on edge working

    //         }
    //     }
    // });

    // Nicho's fft_queue:
    let mut fft_deque: VecDeque<Complex<f32>> = VecDeque::new();
    // this for loop defines the length of the deque / fourier transfomr
    for _i in 0..1536 {
        fft_deque.push_back(num_complex::Complex::new(0., 0.));
    }

    let _play_button_thread = thread::spawn(move || {
        // let gpio = Gpio::new().unwrap();
        // let mut pin = gpio.get().unwrap().into_input_pullup();
        // pin.set_reset_on_drop(false);
        loop {
            // if pin.read() == rppal::gpio::Level::Low {
            if plus_arc.get() == 0 {
                plus_arc.set(2);
            } else if plus_arc.get() == 2 {
                plus_arc.set(4);
            } else {
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
            // }
            thread::sleep(time::Duration::from_millis(10000));
        }
    });
    // while !go_ahead.load(Ordering::Relaxed) {
    //     thread::sleep(time::Duration::from_millis(250));
    // }
    let mut transient_iter = 0;
    let mut curr_trans = 0;
    // rhythm and sample_iter is to bring sound's samples and transient information properly into the scope of event_loop
    let mut rhythm: Vec<i32> = vec![0];
    let dummy_vec = vec![0.];
    let mut sample_iter = dummy_vec.into_iter();
    // Nicho's RGB variables
    // Main RGB code
    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(1536);
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); 1536];
    let mut count = 0;
    let mut uart = Uart::with_path("/dev/ttyAMA0", 115_200, Parity::None, 8, 2).unwrap();
    //uart.set_write_mode(false)?;
    // event_loop.run takes control of the main thread and turns it into a playback thread
    event_loop.run(move |id, result| {
        // this if statement evaluates to true when a new song is being played
        // lets us update sample_iter and and rhythm safely
        if play_flag_arc2.load(Ordering::Relaxed) {
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
                        if !go_ahead.load(Ordering::Relaxed) {
                            for out in sample.iter_mut() {
                                *out = 0.;
                            }
                        } else {
                            let value = sample_iter.next();
                            if value == None {
                                // print!("song over");
                                break;
                            } else {
                                fft_deque.pop_front();
                                fft_deque.push_back(num::Complex::new(value.unwrap(), 0.));

                                count += 1;
                                if count == 4410 {
                                    // Main RGB code
                                    // let mut planner = FFTplanner::new(false);
                                    // let fft = planner.plan_fft(1536);
                                    let mut _fft_vec: Vec<Complex<f32>> =
                                        Vec::from(fft_deque.clone());
                                    fft.process(&mut _fft_vec, &mut output);
                                    let amps =
                                        output.iter().map(|x| x.norm_sqr()).collect::<Vec<f32>>();
                                    let mut amps_iter = amps.iter();

                                    let mut bass_sum = 0.;
                                    let mut mid_sum = 0.;
                                    let mut high_iter = 0.;

                                    let bass_bands = 4;
                                    let mid_bands = 100;
                                    let high_bands = 664;

                                    for _i in 1..bass_bands {
                                        // Do average of the first 4 values and send it out as HEX
                                        bass_sum += amps_iter.next().unwrap();
                                    }
                                    let bass_max = 10000.; // The denominator is chosen from max observed value of the fft
                                    let avg_bass = bass_sum / (bass_bands as f32);
                                    let bass_ref = avg_bass / bass_max;
                                    let bass = 20. * bass_ref.log(10.);

                                    for _i in bass_bands..mid_bands {
                                        // Do average of the next 100 values and send it out as HEX
                                        mid_sum += amps_iter.next().unwrap();
                                    }
                                    let mid_max = 8000.; // The denominator is chosen from max observed value of the fft
                                    let avg_mid = mid_sum / (mid_bands as f32);
                                    let mid_ref = avg_mid / mid_max;
                                    let mid = 20. * mid_ref.log(10.0);

                                    for _i in mid_bands..high_bands {
                                        // Do average over the last values and send out as HEX
                                        let x = amps_iter.next();
                                        if x == None {
                                            break;
                                        }
                                        high_iter += x.unwrap();
                                        break;
                                    }
                                    let high_max = 0.99; // The denominator is chosen from max observed value of the fft
                                    let avg_high = high_iter / (high_bands as f32);
                                    let high_ref = avg_high / high_max;
                                    let high = 20. * high_ref.log(10.0);

                                    count = 0;

                                    // Write output

                                    let mut dmx = dmx::DMX::default();
                                    dmx.change_color(bass, mid, high);
                                    uart.write(&dmx.msg[..]).unwrap();
                                }
                                if transient_iter >= rhythm[curr_trans * 2] as usize
                                    && curr_trans * 2 + 1 < rhythm.len()
                                {
                                    // Send uart message with sound.analysis.rhythm[curr_trans * 2 - 1];
                                    let mut dmx = dmx::DMX::default();
                                    dmx.simple_move(sound.analysis.rhythm[curr_trans * 2 + 1]);
                                    uart.write(&dmx.msg[..]).unwrap();
                            
                                    transient_iter = 0;
                                    curr_trans += 1;
                                    
                                }
                                if transient_iter >= rhythm[curr_trans * 2] as usize
                                    && curr_trans * 2 + 1 < rhythm.len(){
                                    let mut dmx = dmx::DMX::default();
                                    dmx.change_dir();
                                    uart.write(&dmx.msg[..]).unwrap();
                                }
                                transient_iter += 1;
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
