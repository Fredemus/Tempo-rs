// Nicho's crates:
extern crate rustfft;
extern crate num_complex;
extern crate num_traits;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

pub struct RGB {
    
    //fft: std::sync::Arc<dyn :rustfft::FFT>,
    bass_bands: usize,
    mid_bands: usize,
    high_bands: usize,
    bass_max: f32,
    mid_max: f32,
    high_max: f32,
    bass: f32,
    mid: f32,
    high: f32,
}


impl Default for RGB{
    fn default() -> RGB {
        RGB{
            //fft: rustfft::FFTplanner::new(false).plan_fft(1536),
            bass_bands: 4,
            mid_bands: 100,
            high_bands: 664,
            bass_max: 9000., // The denominator is chosen from max observed value of the fft 
            mid_max: 8000., // The denominator is chosen from max observed value of the fft
            high_max: 100., // The denominator is chosen from max observed value of the fft
            bass: 0.,
            mid: 0.,
            high: 0.,
        }
    }
}

impl RGB{

    pub fn rgb_fft(&mut self, mut input: Vec<Complex<f32>>){
        let fft = rustfft::FFTplanner::new(false).plan_fft(1536);

        let mut bass_sum = 0.;         
        let mut mid_sum = 0.;
        let mut high_sum = 0.;

        let mut output: Vec<Complex<f32>> = vec![Complex::zero(); 1536];
        fft.process(&mut input, &mut output);
        let amps = output.iter().map(|x| x.norm()).collect::<Vec<f32>>();
        let mut amps_iter = amps.iter();
        for _i in 1..self.bass_bands {
            // Do average of the first 4 values and send it out as HEX
            bass_sum += amps_iter.next().unwrap();
        }

        for _i in self.bass_bands..self.mid_bands {
            // Do average of the next 100 values and send it out as HEX
            mid_sum += amps_iter.next().unwrap();
        }
        for _i in self.mid_bands..self.high_bands {
            // Do average over the last values and send out as HEX
            let x = amps_iter.next();
            if x == None{
               break;
            }
           high_sum += x.unwrap();
        }
        self.set_bass(bass_sum);
        self.set_mid(mid_sum);
        self.set_high(high_sum);
    }
    fn set_bass(&mut self, bass_sum: f32){
        //Low bands operations
        let avg_bass = bass_sum / (self.bass_bands as f32);
        let bass_ref = avg_bass/ self.bass_max; 
        self.bass = 20.*bass_ref.log(10.);
    }
    fn set_mid(&mut self, mid_sum: f32){
        // Mid bands operations
        let avg_mid = mid_sum / (self.mid_bands as f32);
        let mid_ref = avg_mid / self.mid_max;
        self.mid = 20.*mid_ref.log(10.0);
    }
    fn set_high(&mut self, high_sum: f32){
        // High bands operations
        let avg_high = high_sum / (self.high_bands as f32);
        let high_ref = avg_high / self.high_max;
        self.high = 20.*high_ref.log(10.0);
    }
    pub fn get_bass(&self)->f32 {
        self.bass
    }
    pub fn get_mid(&self)->f32 {
        self.mid
    }
    pub fn get_high(&self)->f32 {
        self.high
    }
}