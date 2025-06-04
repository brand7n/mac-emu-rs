use std::fs;
use log::{info, warn};
use crate::via::{VIA};
use std::io;
use std::io::Write;
use crate::cpu::{get_pc, disassemble_instruction};

// TODO: mac plus rom maps over our VIDEO_BASE
pub const RAM_SIZE: usize = 0x1000000;
pub const ROM_SIZE: usize = 0x10000;
pub const VIDEO_BASE: usize = 0x1A700;
pub const ROM_BASE: u32 = 0x400000;
pub const ROM_END: u32 = ROM_BASE + (ROM_SIZE as u32) - 1;

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

fn wait_for_keypress_hw(label: &str, addr: u32) {
    let pc = get_pc();
    let disasm = disassemble_instruction(pc);
    println!("{} at 0x{:X}\n  PC: 0x{:08X}  {}\nPress Enter to continue...", label, addr, pc, disasm);
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut String::new());
}

pub fn read_u8(addr: u32) -> u8 {
    // IWM: ((addr & 0xFFFFFF) >= 0xDFE1FF) && ((addr & 0xFFFFFF) < 0xE001FF)
    if (addr & 0xFFFFFF) >= 0xDFE1FF && (addr & 0xFFFFFF) < (0xDFE1FF + 0x2000) {
        log::warn!("IWM hardware read at 0x{:X}", addr);
        wait_for_keypress_hw("IWM hardware read", addr);
    }
    // SCC_RD: ((addr & 0xF00000) == 0x900000)
    if (addr & 0xF00000) == 0x900000 {
        log::warn!("SCC_RD hardware read at 0x{:X}", addr);
        wait_for_keypress_hw("SCC_RD hardware read", addr);
    }
    // SCC_WR: ((addr & 0xF00000) == 0xB00000)
    if (addr & 0xF00000) == 0xB00000 {
        log::warn!("SCC_WR hardware read at 0x{:X}", addr);
        wait_for_keypress_hw("SCC_WR hardware read", addr);
    }
    if (addr & 0xE80000) == 0xE80000 {
        let mut via_lock = VIA.lock().unwrap();
        if let Some(via) = via_lock.as_mut() {
            return via.read(addr);
        } else {
            log::warn!("VIA not initialized for read at 0x{:X}", addr);
            return 0xFF;
        }
    }
    unsafe {
        if ROM_MAPPED_AT_ZERO && addr < ROM_SIZE as u32 {
            //info!("read_u8 (ROM@0): 0x{:X}", addr);
            ROM[addr as usize]
        } else if addr >= ROM_BASE && addr < ROM_BASE + ROM_SIZE as u32 {
            //info!("read_u8 (ROM@400000): 0x{:X}", addr);
            ROM[(addr - ROM_BASE) as usize]
        } else if addr < RAM_SIZE as u32 {
            //info!("read_u8 (RAM): 0x{:X}", addr);
            RAM[addr as usize]
        } else {
            warn!("read_u8 unmapped address: 0x{:X}", addr);
            0xFF
        }
    }
}

pub fn write_u8(addr: u32, value: u8) {
    // IWM: ((addr & 0xFFFFFF) >= 0xDFE1FF) && ((addr & 0xFFFFFF) < 0xE001FF)
    if (addr & 0xFFFFFF) >= 0xDFE1FF && (addr & 0xFFFFFF) < (0xDFE1FF + 0x2000) {
        log::warn!("IWM hardware write at 0x{:X} = 0x{:X}", addr, value);
        wait_for_keypress_hw("IWM hardware write", addr);
    }
    // SCC_RD: ((addr & 0xF00000) == 0x900000)
    if (addr & 0xF00000) == 0x900000 {
        log::warn!("SCC_RD hardware write at 0x{:X} = 0x{:X}", addr, value);
        wait_for_keypress_hw("SCC_RD hardware write", addr);
    }
    // SCC_WR: ((addr & 0xF00000) == 0xB00000)
    if (addr & 0xF00000) == 0xB00000 {
        log::warn!("SCC_WR hardware write at 0x{:X} = 0x{:X}", addr, value);
        wait_for_keypress_hw("SCC_WR hardware write", addr);
    }
    if (addr & 0xE80000) == 0xE80000 {
        let mut via_lock = VIA.lock().unwrap();
        if let Some(via) = via_lock.as_mut() {
            via.write(addr, value);
        } else {
            log::warn!("VIA not initialized for write at 0x{:X}", addr);
        }
        return;
    }
    unsafe {
        if ROM_MAPPED_AT_ZERO && addr < ROM_SIZE as u32 {
            warn!("write_u8 attempt to write to ROM@0: 0x{:X}", addr);
        } else if addr >= ROM_BASE && addr < ROM_BASE + ROM_SIZE as u32 {
            warn!("write_u8 attempt to write to ROM@400000: 0x{:X}", addr);
        } else if addr < RAM_SIZE as u32 {
            info!("write_u8 (RAM): 0x{:X} = 0x{:X}", addr, value);
            RAM[addr as usize] = value;
        } else {
            warn!("write_u8 unmapped address: 0x{:X}", addr);
        }
    }
}

pub fn read_u16(addr: u32) -> u16 {
    //info!("read_u16: 0x{:X}", addr);
    let high = read_u8(addr) as u16;
    let low = read_u8(addr + 1) as u16;
    (high << 8) | low
}

pub fn write_u16(addr: u32, value: u16) {
    write_u8(addr, (value >> 8) as u8);
    write_u8(addr + 1, value as u8);
}

pub fn read_u32(addr: u32) -> u32 {
    //info!("read_u32: 0x{:X}", addr);
    let high = read_u16(addr) as u32;
    let low = read_u16(addr + 2) as u32;
    (high << 16) | low
}

pub fn write_u32(addr: u32, value: u32) {
    write_u16(addr, (value >> 16) as u16);
    write_u16(addr + 2, value as u16);
}

#[no_mangle]
pub extern "C" fn m68k_read_disassembler_8(address: u32) -> u32 {
    read_u8(address) as u32
}

#[no_mangle]
pub extern "C" fn m68k_read_disassembler_16(address: u32) -> u32 {
    read_u16(address) as u32
}

#[no_mangle]
pub extern "C" fn m68k_read_disassembler_32(address: u32) -> u32 {
    read_u32(address)
}
