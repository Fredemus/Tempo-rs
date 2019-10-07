/// This project is meant for opening a wave file and calculating its tempo. Meant as a prototype

// hound is a wav file reading library
extern crate hound;

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

    }
    fn generate_analysis_file(&self) {
        
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



fn main() {
    println!("Hello, world!");
}
