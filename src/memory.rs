use std::fs;
use log::{info, warn};

pub const RAM_SIZE: usize = 128 * 1024;
pub const ROM_SIZE: usize = 64 * 1024;
pub const VIDEO_BASE: usize = 0x1A700;
pub const ROM_BASE: u32 = 0x400000;
pub const ROM_END: u32 = ROM_BASE + (ROM_SIZE as u32) - 1;
pub const IO_BASE: u32 = 0xDFF000;
pub const IO_END: u32 = 0xDFFFFF;

pub static mut RAM: [u8; RAM_SIZE] = [0; RAM_SIZE];
pub static mut ROM: [u8; ROM_SIZE] = [0; ROM_SIZE];
pub static mut ROM_MAPPED_AT_ZERO: bool = true;

pub fn load_rom(path: &str) -> Result<(), String> {
    let rom_data = fs::read(path).map_err(|e| format!("Failed to load ROM file: {}", e))?;
    
    if rom_data.len() != ROM_SIZE {
        return Err(format!("Invalid ROM size: expected {} bytes, got {} bytes", ROM_SIZE, rom_data.len()));
    }

    unsafe {
        ROM.copy_from_slice(&rom_data);
        ROM_MAPPED_AT_ZERO = true;  // Start with ROM at 0x0
    }
    
    Ok(())
}

pub fn remap_rom() {
    unsafe {
        ROM_MAPPED_AT_ZERO = false;
    }
}

pub fn read_u8(addr: u32) -> u8 {
    unsafe {
        if ROM_MAPPED_AT_ZERO && addr < ROM_SIZE as u32 {
            info!("read_u8 (ROM@0): 0x{:X}", addr);
            ROM[addr as usize]
        } else if addr < RAM_SIZE as u32 {
            info!("read_u8 (RAM): 0x{:X}", addr);
            RAM[addr as usize]
        } else if addr >= ROM_BASE && addr < ROM_BASE + ROM_SIZE as u32 {
            info!("read_u8 (ROM@400000): 0x{:X}", addr);
            ROM[(addr - ROM_BASE) as usize]
        } else if addr >= IO_BASE && addr < IO_END {
            info!("read_u8 (I/O): 0x{:X}", addr);
            0xFF // Placeholder for I/O reads
        } else {
            warn!("read_u8 unmapped address: 0x{:X}", addr);
            0xFF
        }
    }
}

pub fn write_u8(addr: u32, value: u8) {
    unsafe {
        if ROM_MAPPED_AT_ZERO && addr < ROM_SIZE as u32 {
            warn!("write_u8 attempt to write to ROM@0: 0x{:X}", addr);
        } else if addr < RAM_SIZE as u32 {
            info!("write_u8 (RAM): 0x{:X} = 0x{:X}", addr, value);
            RAM[addr as usize] = value;
        } else if addr >= ROM_BASE && addr < ROM_BASE + ROM_SIZE as u32 {
            warn!("write_u8 attempt to write to ROM@400000: 0x{:X}", addr);
        } else if addr >= IO_BASE && addr < IO_END {
            info!("write_u8 to I/O: 0x{:X} = 0x{:X}", addr, value);
            // Placeholder for I/O writes
        } else {
            warn!("write_u8 unmapped address: 0x{:X}", addr);
        }
    }
}

pub fn read_u16(addr: u32) -> u16 {
    info!("read_u16: 0x{:X}", addr);
    let high = read_u8(addr) as u16;
    let low = read_u8(addr + 1) as u16;
    (high << 8) | low
}

pub fn write_u16(addr: u32, value: u16) {
    write_u8(addr, (value >> 8) as u8);
    write_u8(addr + 1, value as u8);
}

pub fn read_u32(addr: u32) -> u32 {
    info!("read_u32: 0x{:X}", addr);
    let high = read_u16(addr) as u32;
    let low = read_u16(addr + 2) as u32;
    (high << 16) | low
}

pub fn write_u32(addr: u32, value: u32) {
    write_u16(addr, (value >> 16) as u16);
    write_u16(addr + 2, value as u16);
}
