extern crate rppal; //<- rppal only works on linux
use rppal::uart::{Parity, Uart};
#[allow(dead_code)]
pub struct DMX {
    pub msg: Vec<u8>,
    uart: rppal::uart::Uart,
    angle: u8,
}
impl Default for DMX {
    fn default() -> DMX {
        DMX {
            msg: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            uart: Uart::with_path("/dev/ttyAMA0", 115_200, Parity::None, 8, 2).unwrap(),
            angle: 0,
        }
    }
}
#[allow(dead_code)]
impl DMX {
    // this message just moves the lights down depending on transient intensity
    pub fn simple_move(&mut self, transientlevel: i32) {
        //FIXME: To do a full wub it needs to move up again. Shortly after a message should be sent with the opposite direction
        let level_u8: u8 = ((transientlevel as f32 / std::i32::MAX as f32) * 255 as f32) as u8;
        //setting up the arc of motion
        self.angle = level_u8;
        self.msg[1] = self.angle;
        self.msg[6] = self.angle;
        //setting up direction
        //maybe use char instead so we can send u, d, l or r instead
        self.msg[2] = 0b00000001;
        self.msg[7] = 0b00000001;

        self.uart.write(&self.msg[..]).unwrap();
        //FIXME: Color choice should probably be handled here by nicho but not sure
    }
    pub fn change_color(&mut self, bass: f32, mid: f32, high: f32) {
        bass.min(0.);
        mid.min(0.);
        high.min(0.);
        //convert bass to u8
        let bass_converted = ((255. + bass).floor()) as u8;
        let mid_converted = ((255. + mid).floor()) as u8;
        let high_converted = ((255. + high).floor()) as u8;
        let mut rgb_vec = vec![(bass_converted, 0), (mid_converted, 1), (high_converted, 2)];
        rgb_vec.sort(); // Sorting largest values to the end of vector
        // Floor is rounding downwards so we dont loose data when converting to u8
        // The damp_effect is implemented to make the lighting more colorful, by increasing the margin between the highest RGB value and the two lower values
        // rgb_vec[rgb_vec.len()-1].0 is  the index of the last value of sorted vector, thus providing the highest RBG value. We multiply by 0.5 to get 50% of this value
        // The for loop runs to the end of the rgb_vec by using '0..rgb_vec.len() - 1' we subract 1 because the 'len()' function returns 3
        let damp_effect = (rgb_vec[rgb_vec.len() - 1].0 as f32 * 0.5).floor() as u8;
        for i in 0..rgb_vec.len() - 1 {
            rgb_vec[i].0 -= damp_effect;
        }
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        // using the index to match the values back to where they belong
        for i in 0..rgb_vec.len() {
            match rgb_vec[i].1 {
                0 => r = rgb_vec[i].0,
                1 => g = rgb_vec[i].0,
                2 => b = rgb_vec[i].0,
                _ => (),
            }
        }
        // Unit 1
        self.msg[3] = r;
        self.msg[4] = g;
        self.msg[5] = b;
        // Unit 2
        self.msg[8] = r;
        self.msg[9] = g;
        self.msg[10] = b;
        // avoid moves happening from this msg
        self.msg[1] = 0;
        self.msg[6] = 0;
        // sending the message
        self.uart.write(&self.msg[..]).unwrap();
    }
    pub fn simple_move_back(&mut self) {
        self.msg[1] = self.angle;
        self.msg[6] = self.angle;

        self.msg[2] = self.msg[2] ^ 0b00000001;
        self.msg[7] = self.msg[7] ^ 0b00000001;

        self.uart.write(&self.msg[..]).unwrap();
    }

    pub fn left_right_move(&mut self) {
        //directions
        self.msg[2] = 0b00000010;
        self.msg[7] = 0b00000010;

        self.uart.write(&self.msg[..]).unwrap();
    }
    pub fn left_right_back(&mut self) {
        self.msg[2] = self.msg[2] ^ 0b00000010;
        self.msg[7] = self.msg[7] ^ 0b00000010;

        self.uart.write(&self.msg[..]).unwrap();
    }
    /*pub fn four_move(&mut self, transientlevel: i32)){ //prolly not possible, here should be made in main but its annoying to do???
        let level_u8: u8 = ((transientlevel as f32 / std::i32::MAX as f32) * 255 as f32) as u8;

        self.msg[1] = level_u8;
        self.msg[6] = level_u8;

        self.msg[2] = 0b00000010;
        self.msg[7] = 0b00000010;
    }*/
}
