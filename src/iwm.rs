pub struct Iwm {
    regs: [u8; 16],
}

impl Iwm {
    pub fn new() -> Self {
        Iwm {
            regs: [0; 16],
        }
    }

    pub fn write(&mut self, addr: u32, val: u8) {
        let r = ((addr >> 9) & 0xf) as usize;
        log::info!("[IWM: WR {:02x} -> {}]", val, r);
        match r {
            _ => {
                log::warn!("[IWM: unhandled WR {:02x} to reg {}]", val, r);
            }
        }
        self.regs[r] = val;
    }

    pub fn read(&self, addr: u32) -> u8 {
        let r = ((addr >> 9) & 0xf) as usize;
        let mut data = self.regs[r];
        match r {
            8 => {
                data = 0xff;
            }
            14 => {
                data = 0x1f;
            }
            _ => {
                log::warn!("[IWM: unhandled RD of reg {}]", r);
            }
        }
        log::info!("[IWM: RD {} <- {:02x}]", r, data);
        data
    }
}
