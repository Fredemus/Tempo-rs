/*
    TODO:
    Move to samples to next transient and transient level for creation of analysis files
    https://stackoverflow.com/questions/30838358/what-is-the-correct-way-to-write-vecu16-content-to-a-file

    IDEAS:
    Maybe a "best guess" based algorithm, where it cycles through bpms to see which best fits the transients
    Idea for a pattern: If there's no transients for a while, slowly sweep up when short-time RMS increases
    parallelize by not loading the entire wav at once and just use the samples iterator?

*/
/// This project is meant for opening a wave file and calculating its tempo. Meant as a prototype
// hound is a wav file reading library
extern crate hound;
// byteorder helps create files the right way so they can be read on an ARM device (Raspberry Pi), since we can specify endianness
extern crate byteorder;
use byteorder::{WriteBytesExt, ReadBytesExt, LittleEndian};
// file operations
use std::fs::OpenOptions;
// use std::io::{Write, BufWriter, BufReader};
use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;
use std::collections::VecDeque;
use std::fs;
// note: static variables are thread-local
// global variables for tweaking how the detection works
static AVG_LEN: usize = 768;
static SKIP_AMT: usize = 4096; //SKIP_AMT should never be less than AVG_LEN
static SENSITIVITY: f32 = 0.7; // lower is more sensitive
struct SoundFile {
    /// Sound samples
    samples: Vec<f32>,
    //the name of the file that was read into SoundFile
    file_name: std::path::PathBuf,
    fs: usize,
    power_buf: VecDeque<f32>,
    analysis: Analysis,
    transient_no: usize,
    transient_gap: usize,
}
// #[allow(dead_code)]
impl SoundFile {
    // splits the string in 2 at the . sign and discards everything behind it
    // fn remove_file_extension(&mut self) {
    //     let split: Vec<&str> = self.file_name.splitn(2, '.').collect();
    //     self.file_name = split[0].to_string();
    // }
    // FIXME: if the analysis file exists, it would be fine to just stream audio from the samples iterator
    // would save a lot of load time, since the collect takes forever
    // loads a wav file and saves the samples in it
    fn load_sound(&mut self, path: std::path::PathBuf) {
        self.file_name = path;
        self.file_name.set_extension("wav");
        let mut reader = hound::WavReader::open(&self.file_name).unwrap();
        self.samples = reader.samples().collect::<Result<Vec<_>, _>>().unwrap();
        self.file_name.set_extension("txt");
    }
    // checks if an analysis file exists for the loaded wav file
    fn search_for_file(&self) -> bool {
        self.file_name.exists()
    }
    // generates an analysis file and fills it with the relevant data
    fn generate_analysis_file(&mut self) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.file_name)
            .expect("Filen kunne ikke åbnes");
        // file should be filled with the attributes in the AnalysisFile created
        // writing the vector into the analysis file with endianness specified. Should be safe?
        let slice_f32: &[f32] = &*self.analysis.rhythm;
        //writing in bpm
        let _ = file.write_f32::<LittleEndian>(self.analysis.tempo).expect("Der kunne ikke skrives til filen");
        // writing in the vector
        for &n in slice_f32 {
            let _ = file.write_f32::<LittleEndian>(n).expect("Der kunne ikke skrives til filen");
        }
    }
    fn read_analysis_file(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .create(false)
            .open(&self.file_name)
            .expect("Filen kunne ikke åbnes");
            // reading the raw bytes from file to a vector: (since Byteorder crate requires it)
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).expect("couldn't read file");
            //reading the raw bytes into rhythm
            let mut reader = Cursor::new(buffer);
            self.analysis.rhythm.clear();
            for _i in 0..self.samples.len() {
                self.analysis.rhythm.push(reader.read_f32::<LittleEndian>().unwrap())
            }
            self.analysis.tempo = self.analysis.rhythm.remove(0);
    }

    fn _bpm_from_rhythm(&mut self) {
        let mut transientsum = 0;
        for i in 0..self.analysis.rhythm.len() {
            if self.analysis.rhythm[i] != 0. {
                transientsum += 1;
            }
        }
        println!("transientsum is {}!", transientsum);
        println!(
            "number of minutes is {}!",
            (self.analysis.rhythm.len() as f32 / self.fs as f32 / 60.)
        );

        // average transients per second * 60 gives us our bpm
        self.analysis.tempo =
            transientsum as f32 / (self.analysis.rhythm.len() as f32 / self.fs as f32 / 60.);
        // limiting bpm to a rational interval
        // FIXME: Can this end in an infinite loop?
        while self.analysis.tempo > 200. || self.analysis.tempo < 70. {
            if self.analysis.tempo > 200. {
                self.analysis.tempo /= 2.;
            } else {
                self.analysis.tempo *= 2.;
            }
        }
    }

    fn bpm_in_frames(&mut self) {
        let mut bpm_frames: Vec<f32> = vec![];
        let mut it = 0;
        // j is frames. how to find number of? just loop until it's through the vector? When to increase the number of
        for _i in 0..self.transient_no {
            let mut len = 0;
            // loops until the next transient
            while it < self.samples.len() && self.analysis.rhythm[it] < 0.1 {
                it += 1;
                len += 1;
            }
            it += 1;
            len += 1;
            // average transients per second * 60 gives us our bpm
            bpm_frames.push(1. / (len as f32 / self.fs as f32) * 60.);
        }
        // filtering out frames with a bpm over 300 FIXME: Maybe filter out too low bpms too?
        // for (i, element) in bpm_frames.iter().enumerate() {
        let mut toberemoved : Vec<usize> = vec![];
        for i in 0..bpm_frames.len() {
            if bpm_frames[i] > 300. {
                toberemoved.push(i);
            }
        }
        //remove in reverse direction so we don't remove the wrong ones because of right-shift
        for i in toberemoved.iter().rev() {
            bpm_frames.remove(*i);
        }
        // sorting the vector
        bpm_frames.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // println!("All BPM frames: {:?}", bpm_frames);
        // Finding the median tempo
        self.analysis.tempo = bpm_frames[(bpm_frames.len() / 2)];
        println!("BPM is {}", self.analysis.tempo);
    }
    fn detect_transients(&mut self) {
        self.analysis.rhythm = vec![0.; self.samples.len()];
        let mut short_rms: f32;
        self.transient_no = 0;
        let mut iter = self.samples.iter().enumerate();
        let mut current: Option<(usize, &f32)> = iter.next();
        while current != None {
            let mut current_sample = current.unwrap();
            self.power_buf.push_back(current_sample.1.powi(2));
            self.power_buf.pop_front();
            // finding rms over the buffer
            short_rms = (self.power_buf.iter().map(|&x| x).sum::<f32>()
                / self.power_buf.len() as f32).sqrt();
            if current_sample.1.abs() - short_rms > SENSITIVITY
            // && self.transient_gap > AVG_LEN
            {
                self.analysis.rhythm[current_sample.0] = current_sample.1.abs() - short_rms;
                self.transient_no += 1;
                self.transient_gap = 0;
                // fast forwarding through the samples, since the next n samples won't have any transients anyway
                current = iter.nth(SKIP_AMT);
                if current == None {
                    break;
                }
                else {
                    current_sample = current.unwrap();
                }
                // skipping the power_buf forward too
                self.power_buf.clear();
                if (current_sample.0 as isize - AVG_LEN as isize)  < 0 {
                    for _i in 0..AVG_LEN {
                        self.power_buf.push_back(0.);
                    }
                    for j in 0..current_sample.0 {
                        self.power_buf.push_back(self.samples[j].powi(2));
                        self.power_buf.pop_front();
                    }
                }
                // is this even faster? is there any reason to have an if-else in the loop?
                else {
                    self.power_buf.extend((&self.samples[current_sample.0-AVG_LEN..current_sample.0]).into_iter().map(|x| x.powi(2)));
                }
            }
            self.transient_gap += 1;
            current = iter.next();
        }
        println!("Sum is {}", self.transient_no);
    }
}
impl Default for SoundFile {
    fn default() -> SoundFile {
        let mut vecdeque = VecDeque::new();
        //rms found over 256 samples
        for _i in 0..AVG_LEN {
            vecdeque.push_back(0.);
        }
        SoundFile {
            samples: vec![0.],
            file_name: std::path::PathBuf::new(),
            fs: 44100,
            power_buf: vecdeque,
            analysis: Analysis::default(),
            transient_no: 0,
            transient_gap: 0,
        }
    }
}
/// contains the information that controls the lightshow. Found on background of SoundFile
struct Analysis {
    /// tempo in beats per minute
    tempo: f32,
    /// Contains every time a transient is detected. Same time format as the SoundFile
    rhythm: Vec<f32>,
}
impl Analysis {
}
impl Default for Analysis {
    fn default() -> Analysis {
        Analysis {
            tempo: 0.,
            rhythm: vec![0.],
        }
    }
}

fn main() {
    let mut sound = SoundFile::default();
    // grabbing all files in Songs and adding their paths to a vector
    let path = Path::new(r"./Songs"); //FIXME: this \ should be / on linux >:(
    let mut entries : Vec<std::path::PathBuf> = vec![];
    println!("which of the songs do you want to play? Write a number");
    for entry in fs::read_dir(path).expect("Unable to list") {
        entries.push(entry.expect("unable to get entry").path());
    }
    for (i,entry) in entries.iter().enumerate() {
        println!("{}: {}", i, entry.display());
    }
    //choose a sound:
    let mut n = String::new();
    std::io::stdin()
        .read_line(&mut n)
        .expect("failed to read input.");
    let n: usize = n.trim().parse().expect("invalid input");
    println!("playing song: {:?}", entries[n]);
    // println!("{}", entries[1]);
    sound.load_sound(
        entries[n].clone()
    );
    if sound.search_for_file() != true {
        sound.detect_transients();
        sound.bpm_in_frames();
        sound.generate_analysis_file();
    } else {
        println!("Analysis file already exists boy");
        sound.read_analysis_file();
        println!("BPM is {}", sound.analysis.tempo);
    }
}
