/// This project is meant for opening a wave file and calculating its tempo. Meant as a prototype

// hound is a wav file reading library
extern crate hound;
// file operations
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
// use std::io::BufReader;
// use std::io::BufRead;

struct SoundFile {
    /// Sound samples
    samples: Vec<f32>,
    //the name of the file that was read into SoundFile
    file_name: String,
    fs: usize,
}
#[allow(dead_code)]
impl SoundFile {
    fn remove_file_extension(&mut self) {
        // splits the string in 2 at the . sign
        let split : Vec<&str> = self.file_name.splitn(2, '.').collect();
        self.file_name = split[0].to_string();
    }

    fn load_sound(&mut self, path: String) {
        self.file_name = path;
        let mut reader = hound::WavReader::open(self.file_name.clone()).unwrap();
        self.samples = reader.samples().collect::<Result<Vec<_>, _>>().unwrap();
        self.remove_file_extension();
    }
    // GET THIS TO WORK
    fn search_for_file(&self) -> bool {
        // FIXME: Find a way to remove .wav 
        // name should be file_name with .txt instead of .wav
        let name = format!("{}.txt",self.file_name);
        Path::new(&name).exists()
    }
    fn generate_analysis_file(&mut self, analysis: Analysis) {
        self.remove_file_extension();
        println!("{}", self.file_name);
        let name = format!("{}.txt",self.file_name);
        println!("{}", name);
        //FIXME: Only works if the file exists
        let mut file = OpenOptions::new().write(true).open(name).expect("Filen kunne ikke åbnes");
        // file should be filled with the attributes in the AnalysisFile created
        //FIXME: Might have to handle the vector differently
        let string : String = format!("{}\n{:?}", analysis.tempo, analysis.rhythm);
        file.write(string.as_bytes()).expect("Der kunne ikke skrives til filen");
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
    let mut sound = SoundFile::default();
    let mut analysis = Analysis { tempo: 2000., rhythm: vec![0.]};
    println!("wok");
    sound.load_sound(r"C:\Users\rasmu\RustProjects\Projekt4\Tempo\Songs\Basicbeat120.wav".to_string());
    if sound.search_for_file() != true {
        sound.generate_analysis_file(analysis);
    }
    else {
        println!("already exists boy");
    }
    println!("Hello, world!");
}
