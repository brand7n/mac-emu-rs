use std::fs;

pub const RAM_SIZE: usize = 128 * 1024;
pub const ROM_SIZE: usize = 64 * 1024;
pub const VIDEO_BASE: usize = 0x1A700;

pub static mut RAM: [u8; RAM_SIZE] = [0; RAM_SIZE];
pub static mut ROM: [u8; ROM_SIZE] = [0; ROM_SIZE];

// pub fn load_rom(path: &str) {
//     let rom_data = fs::read(path).expect("Failed to load ROM file");
//     assert_eq!(rom_data.len(), ROM_SIZE, "Expected 64KB ROM");

//     unsafe {
//         ROM.copy_from_slice(&rom_data);
//     }
// }

pub fn read_u8(addr: u32) -> u8 {
    unsafe {
        match addr {
            0x000000..=0x01FFFF => RAM[addr as usize],
            0x400000..=0x40FFFF => ROM[(addr - 0x400000) as usize],
            _ => 0xFF,
        }
    }
}

pub fn read_u16(addr: u32) -> u16 {
    let high = read_u8(addr) as u16;
    let low = read_u8(addr + 1) as u16;
    (high << 8) | low
}

pub fn write_u8(addr: u32, value: u8) {
    unsafe {
        if (0x000000..=0x01FFFF).contains(&addr) {
            RAM[addr as usize] = value;
        }
    }
}

pub fn read_u32(addr: u32) -> u32 {
    let high = read_u16(addr) as u32;
    let low = read_u16(addr + 2) as u32;
    (high << 16) | low
}

pub fn write_u32(addr: u32, value: u32) {
    write_u16(addr, (value >> 16) as u16);
    write_u16(addr + 2, value as u16);
}

pub fn write_u16(addr: u32, value: u16) {
    write_u8(addr, (value >> 8) as u8);
    write_u8(addr + 1, value as u8);
}
