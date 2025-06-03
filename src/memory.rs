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
    //println!("read_u8: 0x{:X}", addr);
    unsafe {
        match addr {
            0x000000..=0x01FFFF => {
                if addr as usize >= RAM_SIZE {
                    println!("WARNING: read_u8 out of bounds: 0x{:X}", addr);
                    0xFF
                } else {
                    RAM[addr as usize]
                }
            },
            0x400000..=0x40FFFF => {
                let rom_addr = (addr - 0x400000) as usize;
                if rom_addr >= ROM_SIZE {
                    println!("WARNING: read_u8 ROM out of bounds: 0x{:X}", addr);
                    0xFF
                } else {
                    ROM[rom_addr]
                }
            },
            _ => {
                println!("WARNING: read_u8 unmapped address: 0x{:X}", addr);
                0xFF
            },
        }
    }
}

pub fn read_u16(addr: u32) -> u16 {
    //println!("read_u16: 0x{:X}", addr);
    let high = read_u8(addr) as u16;
    let low = read_u8(addr + 1) as u16;
    (high << 8) | low
}

pub fn write_u8(addr: u32, value: u8) {
    unsafe {
        if (0x000000..=0x01FFFF).contains(&addr) {
            if addr as usize >= RAM_SIZE {
                println!("WARNING: write_u8 out of bounds: 0x{:X}", addr);
            } else {
                RAM[addr as usize] = value;
            }
        } else {
            println!("WARNING: write_u8 unmapped address: 0x{:X}", addr);
        }
    }
}

pub fn read_u32(addr: u32) -> u32 {
    //println!("read_u32: 0x{:X}", addr);
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
