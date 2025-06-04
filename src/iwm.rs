pub struct Iwm {
    control: u8,
    status: u8,
    // possibly: track, data, etc.
}

impl Iwm {
    pub fn new() -> Self {
        Iwm {
            control: 0,
            status: 0x80, // e.g. "ready" bit set
        }
    }

    pub fn read(&self, addr: u32) -> u8 {
        match addr & 0xFFFF {
            0xFFFF => self.control,
            0xFDFF => self.status,
            _ => {
                log::warn!("Unhandled IWM read: 0x{:06X}", addr);
                0xFF
            }
        }
    }

    pub fn write(&mut self, addr: u32, val: u8) {
        match addr & 0xFFFF {
            0xFFFF => {
                self.control = val;
                log::info!("IWM: control write = 0x{:02X}", val);
            }
            _ => {
                log::warn!("Unhandled IWM write: 0x{:06X} = 0x{:02X}", addr, val);
            }
        }
    }
}
