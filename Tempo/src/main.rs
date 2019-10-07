/// This project is meant for opening a wave file and calculating its tempo. Meant as a prototype

// hound is a wav file reading library
extern crate hound;
// file operations
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;

struct SoundFile {
    /// Sound samples
    samples: Vec<f32>,
    //the name of the file that was read into SoundFile
    file_name: String,
    fs: usize,
}
#[allow(dead_code)]
impl SoundFile {
    fn load_sound(&mut self, path: String) {
        let mut reader = hound::WavReader::open(path).unwrap();
        self.samples = reader.samples().collect::<Result<Vec<_>, _>>().unwrap();
    }
    fn search_for_file(&self) {
        // FIXME: Find a way to remove .wav 
        // name should be file_name with .txt instead of .wav
        let name = format!("{}.txt",self.file_name);
        let file = File::open(name).expect("Filen kunne ikke åbnes"); 
        // FIXME: if a file doesn't exist error, generate_analysis_file should be called  
    }
    fn generate_analysis_file(&self, analysis: &Analysis) {
        // FIXME: Find a way to remove .wav 
        // name should be file_name with .txt instead of .wav
        let name = format!("{}.txt",self.file_name);
        //FIXME: Needs a way to stop if the file exists
        let mut file = OpenOptions::new().write(true).open(name).expect("Filen kunne ikke åbnes");
        // file should be filled with the attributes in the AnalysisFile created
        //FIXME: Might have to handle the vector differently
        let string : String = format!("{}\n{:?}", analysis.tempo, analysis.rhythm);
        file.write(string.as_bytes()).expect("der kunne ikke skrives til filen");
    }
    fn bpm_from_rhythm(&self, file: &mut Analysis) {
        let mut transientsum = 0;
        for i in 0..file.rhythm.len() {
            if file.rhythm[i] != 0. {
                transientsum += 1;
            }
        }
        // average transients per second * 60 gives us our bpm
        file.tempo = (transientsum / file.rhythm.len()) as f32 * self.fs as f32 * 60.;
    }

}
impl Default for SoundFile {
    fn default() -> SoundFile {
        SoundFile { 
            samples: vec![0.],
            file_name: format!(""),
            fs: 44100,
        }
    }
}
/// contains the samples that controls the lightshow. Found on background of SoundFile
struct Analysis {
    /// tempo in beats per minute
    tempo: f32,
    /// Contains every time a transient is detected. Same time format as the SoundFile
    rhythm: Vec<f32>,
}
impl Analysis {
    fn read_analysis_file(&mut self) {
        
    }
}


fn main() {
    println!("Hello, world!");
}
