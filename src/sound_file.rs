use std::collections::VecDeque;
// byteorder helps create files the right way so they can be read on an ARM device (Raspberry Pi), since we can specify endianness
extern crate byteorder;
use byteorder::{WriteBytesExt, ReadBytesExt, LittleEndian};
// file operations
use std::fs::OpenOptions;
// use std::io::{Write, BufWriter, BufReader};
use std::io::prelude::*;
use std::io::Cursor;
use std::i32;
static AVG_LEN: usize = 768;
static SKIP_AMT: usize = 4096; // SKIP_AMT should never be less than AVG_LEN
static SENSITIVITY: f32 = 0.7; // lower is more sensitive
pub struct SoundFile {
    /// Sound samples
    samples: Vec<f32>,
    //the name of the file that was read into SoundFile
    file_name: std::path::PathBuf,
    fs: usize,
    power_buf: VecDeque<f32>,
    pub analysis: Analysis,
    transient_no: usize,
    transient_gap: usize,
}
// #[allow(dead_code)]
impl SoundFile {
    pub fn load_sound(&mut self, path: std::path::PathBuf) {
        self.file_name = path;
        self.file_name.set_extension("wav");
        let mut reader = hound::WavReader::open(&self.file_name).unwrap();
        self.samples = reader.samples().collect::<Result<Vec<_>, _>>().unwrap();
        self.file_name.set_extension("txt");
    }
    // checks if an analysis file exists for the loaded wav file
    pub fn search_for_file(&self) -> bool {
        self.file_name.exists()
    }
    // generates an analysis file and fills it with the relevant data
    pub fn generate_analysis_file(&mut self) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.file_name)
            .expect("Filen kunne ikke åbnes");
        // file should be filled with the attributes in the AnalysisFile created
        // writing the vector into the analysis file with endianness specified. Should be safe?
        let slice_i32: &[i32] = &*self.analysis.rhythm;
        //writing in bpm
        let _ = file.write_f32::<LittleEndian>(self.analysis.tempo).expect("Der kunne ikke skrives til filen");
        // writing in the vector
        for &n in slice_i32 {
            let _ = file.write_i32::<LittleEndian>(n).expect("Der kunne ikke skrives til filen");
        }
    }
    pub fn read_analysis_file(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .create(false)
            .open(&self.file_name)
            .expect("Filen kunne ikke åbnes");
            // reading the raw bytes from file to a vector: (since Byteorder crate requires it)
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).expect("couldn't read file");
            let num_u8 = buffer.len();
            //reading the raw bytes into rhythm
            let mut reader = Cursor::new(buffer);
            self.analysis.rhythm.clear();
            // reading in tempo as a f32
            self.analysis.tempo = reader.read_f32::<LittleEndian>().unwrap();
            // reading in analysis as i32
            // FIXME: This might go out of scope since buffer is over u8
            for _i in 0..(num_u8 - 4) / 4 {
                self.analysis.rhythm.push(reader.read_i32::<LittleEndian>().unwrap())
            }
            // self.analysis.tempo = self.analysis.rhythm.remove(0);
    }

    fn _bpm_from_rhythm(&mut self) {
        let mut transientsum = 0;
        // for i in 0..self.analysis.rhythm.len() {
        //     if self.analysis.rhythm[i] != 0. {
        //         transientsum += 1;
        //     }
        // }
        transientsum = self.analysis.rhythm.len() / 2;
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

    pub fn bpm_in_frames(&mut self) {
        let mut bpm_frames: Vec<f32> = vec![];
        let mut it = 0;
        // j is frames. how to find number of? just loop until it's through the vector? When to increase the number of
        // for _i in 0..self.transient_no {
        //     let mut len = 0;
        //     // loops until the next transient
        //     while it < self.samples.len() && self.analysis.rhythm[it] < 0.1 {
        //         it += 1;
        //         len += 1;
        //     }
        //     it += 1;
        //     len += 1;
        //     // average transients per second * 60 gives us our bpm
        //     bpm_frames.push(1. / (len as f32 / self.fs as f32) * 60.);
        // }
        for i in 1..self.analysis.rhythm.len() / 2 {
            bpm_frames.push(1. / (self.analysis.rhythm[i*2] as f32 / self.fs as f32) * 60.);
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
    pub fn detect_transients_by_rms(&mut self) {
        self.analysis.rhythm.clear();
        let mut short_rms: f32;
        self.transient_gap = 0;
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
                // pushing in the amplitude of the transient and how many samples passed since last transient
                self.analysis.rhythm.push(((current_sample.1.abs() - short_rms)  * i32::MAX as f32) as i32);
                self.analysis.rhythm.push(self.transient_gap as i32);
                self.transient_no += 1;
                self.transient_gap = SKIP_AMT;
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
pub struct Analysis {
    /// tempo in beats per minute
    tempo: f32,
    /// Contains every time a transient is detected. Same time format as the SoundFile
    pub rhythm: Vec<i32>,
}
impl Analysis {
    pub fn get_tempo(&self) -> f32 {
        self.tempo
    }
}
impl Default for Analysis {
    fn default() -> Analysis {
        Analysis {
            tempo: 0.,
            rhythm: vec![0],
        }
    }
}