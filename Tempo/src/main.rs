/*
    TODO:
    BPM frames for songs with different rhythms?
    If there's been a transient in the last n samples, it shouldn't be able to detect more
    Figure out how to read the analysis file into the variables
    Move to gaps between transients and transient level for creation of analysis files

    IDEAS:
    Maybe a "best guess" based algorithm, where it cycles through bpms to see which best fits the transients
    Idea for a pattern: If there's no transients for a while, slowly sweep up when short-time RMS increases


*/
/// This project is meant for opening a wave file and calculating its tempo. Meant as a prototype
// hound is a wav file reading library
extern crate hound;
// file operations
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
// use std::io::BufReader;
// use std::io::BufRead;
use std::collections::VecDeque;

// note: static variables are thread-local
// global variables for tweaking how the detection works
static AVG_LEN: usize = 512;
static SENSITIVITY: f32 = 0.9;
struct SoundFile {
    /// Sound samples
    samples: Vec<f32>,
    //the name of the file that was read into SoundFile
    file_name: String,
    fs: usize,
    power_buf: VecDeque<f32>,
    analysis: Analysis,
    transient_gap: usize,
    transient_no: usize,
}
#[allow(dead_code)]
impl SoundFile {
    // splits the string in 2 at the . sign and discards everything behind it
    fn remove_file_extension(&mut self) {
        let split: Vec<&str> = self.file_name.splitn(2, '.').collect();
        self.file_name = split[0].to_string();
    }
    // loads a wav file and saves the samples in it
    fn load_sound(&mut self, path: String) {
        self.file_name = path;
        let mut reader = hound::WavReader::open(self.file_name.clone()).unwrap();
        self.samples = reader.samples().collect::<Result<Vec<_>, _>>().unwrap();
        self.remove_file_extension();
    }
    // checks if an analysis file exists for the loaded wav file
    fn search_for_file(&self) -> bool {
        // name should be file_name with .txt instead of .wav
        let name = format!("{}.txt", self.file_name);
        Path::new(&name).exists()
    }
    // generates an analysis file and fills it with the relevant data
    fn generate_analysis_file(&mut self) {
        println!("{}", self.file_name);
        let name = format!("{}.txt", self.file_name);
        //FIXME: Only works if the file exists
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(name)
            .expect("Filen kunne ikke åbnes");
        // file should be filled with the attributes in the AnalysisFile created
        let string: String = format!("{}\n{:?}", self.analysis.tempo, self.analysis.rhythm);
        file.write(string.as_bytes())
            .expect("Der kunne ikke skrives til filen");
    }
    fn bpm_from_rhythm(&mut self) {
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

            // limiting bpm to a rational interval
            // FIXME: Can this end in an infinite loop?
            let current = bpm_frames.len() - 1;
            println!("bpm of this frame is {}!", bpm_frames[current]);
            while bpm_frames[current] > 200. || bpm_frames[current] < 70. {
                if bpm_frames[current] < 1. {
                    break;
                } else if bpm_frames[current] > 200. {
                    bpm_frames[current] /= 2.;
                } else {
                    bpm_frames[current] *= 2.;
                }
            }
        }
        // sorting the vector
        bpm_frames.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // Finding the median tempo
        self.analysis.tempo = bpm_frames[(bpm_frames.len() / 2)];
        println!("BPM is {}", self.analysis.tempo);
    }
    fn detect_transients(&mut self) {
        self.analysis.rhythm = vec![0.; self.samples.len()];
        let mut short_rms: f32;
        self.transient_no = 0;
        for i in 0..self.samples.len() {
            self.power_buf.push_back(self.samples[i]);
            self.power_buf.pop_front();
            // finding rms over the buffer
            short_rms = self.power_buf.iter().map(|&x| x.powi(2)).sum::<f32>()
                / self.power_buf.len() as f32;
            // println!("short_rms is {}", short_rms);
            //
            if self.samples[i].abs() - short_rms > SENSITIVITY && self.transient_gap > AVG_LEN {
                self.analysis.rhythm[i] = 1.;
                self.transient_gap = 0;
                self.transient_no += 1;
            }
            self.transient_gap += 1;
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
            file_name: format!(""),
            fs: 44100,
            power_buf: vecdeque,
            analysis: Analysis::default(),
            transient_gap: 0,
            transient_no: 0,
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
    fn _read_analysis_file(&mut self) {}
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
    // let mut analysis = Analysis { tempo: 2000., rhythm: vec![0.]};
    sound.load_sound(
        // r"C:\Users\rasmu\Documents\RustProjects\Projekt4\Tempo\Songs\Daft Punk - Da Funk.wav".to_string(),
        r"C:\Users\rasmu\Documents\RustProjects\Projekt4\Tempo\Songs\busybeat100.wav".to_string(),
    );

    if sound.search_for_file() != true {
        sound.detect_transients();
        sound.bpm_in_frames();
        sound.generate_analysis_file();
    } else {
        sound.detect_transients();
        sound.bpm_in_frames();
        println!("BPM is {}", sound.analysis.tempo);
        sound.generate_analysis_file();
        println!("already exists boy");
    }
}
