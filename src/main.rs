/*
    TODO:
    Move to samples to next transient and transient level for creation of analysis files
    https://stackoverflow.com/questions/30838358/what-is-the-correct-way-to-write-vecu16-content-to-a-file

    No reason for transient level to be i32. Consider splitting into u8 and u32, even if it makes reading more complicated
    u32 might make more sense than i32 too.


    IDEAS:
    Maybe a "best guess" based algorithm, where it cycles through bpms to see which best fits the transients
    Idea for a pattern: If there's no transients for a while, slowly sweep up when short-time RMS increases
    parallelize by not loading the entire wav at once and just use the samples iterator?

*/
/// This project is meant for opening a wave file and calculating its tempo. Meant as a prototype
// hound is a wav file reading library
extern crate hound;

use std::fs;
use std::path::Path;
// note: static variables are thread-local
// global variables for tweaking how the detection works
mod sound_file;

fn main() {
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
    // let n = 4;
    println!("playing song: {:?}", entries[n]);
    sound.load_sound(entries[n].clone());
    if sound.search_for_file() != true {
        sound.detect_transients_by_rms();
        sound.bpm_in_frames();
        sound.generate_analysis_file();
        println!("BPM is {}", sound.analysis.get_tempo());
        println!("{:?}", sound.analysis.rhythm);
    } else {
        println!("Analysis file already exists boy");
        sound.read_analysis_file();
        println!("BPM is {}", sound.analysis.get_tempo());
        println!("{:?}", sound.analysis.rhythm);
    }
}
