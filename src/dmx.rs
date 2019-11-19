// extern crate rppal; <- rppal only works on linux
// use rppal::uart::{Parity, Uart};
#[allow(dead_code)]
pub struct DMX {
    msg: Vec<u8>,
}

impl Default for DMX {
    fn default() -> DMX {
        DMX {
            msg: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            // uart: Uart::with_path("/dev/ttyAMA0", 115_200, Parity::None, 8, 2),
        }
    }
}
#[allow(dead_code)]
impl DMX {
    // this message just moves the lights down depending on transient intensity
    pub fn simple_move(&mut self, transientlevel: u32) {
        //FIXME: To do a full wub it needs to move up again. Shortly after a message should be sent with the opposite direction
        let level_u8: u8 = ((transientlevel as f32 / std::i32::MAX as f32) * 255 as f32) as u8;
        //setting up the arc of motion
        self.msg[1] = level_u8;
        self.msg[6] = level_u8;
        //setting up direction
        self.msg[2] = 0b00000001;
        self.msg[7] = 0b00000001;
        //FIXME: Color choice should probably be handled here by nicho but not sure
    }
    fn change_color(&mut self, r: u8, g: u8, b: u8) {
        self.msg[3] = r;
        self.msg[4] = g;
        self.msg[5] = b;

        self.msg[8] = r;
        self.msg[9] = g;
        self.msg[10] = b;
    }
    fn change_dir(&mut self) {
        self.msg[2] = self.msg[2] ^ 0b00000001;
        self.msg[7] = self.msg[7] ^ 0b00000001;
    }
}
