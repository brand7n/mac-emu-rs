use std::fs;
use log::{info, warn};
use crate::via::{VIA};
use std::io;
use std::io::Write;
use crate::cpu::{get_pc, disassemble_instruction};
use crate::iwm::Iwm;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref IWM_INSTANCE: Mutex<Iwm> = Mutex::new(Iwm::new());
}

pub(crate) static SINGLE_STEP: AtomicBool = AtomicBool::new(false);

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

pub(crate) fn wait_for_keypress_hw(label: &str, addr: u32) -> bool {
    let pc = get_pc();
    let disasm = disassemble_instruction(pc);
    crate::cpu::display_registers();
    println!("{} at 0x{:X}\n  PC: 0x{:08X}  {}\nPress Enter to continue, or 's' then Enter to single-step...", label, addr, pc, disasm);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    if input.trim() == "s" {
        SINGLE_STEP.store(true, Ordering::SeqCst);
        false
    } else {
        SINGLE_STEP.store(false, Ordering::SeqCst);
        true
    }
}

pub fn read_u8(addr: u32) -> u8 {
    // IWM: ((addr & 0xFFFFFF) >= 0xDFE1FF) && ((addr & 0xFFFFFF) < 0xE001FF)
    if (addr & 0xFFFFFF) >= 0xDFE1FF && (addr & 0xFFFFFF) < (0xDFE1FF + 0x2000) {
        log::warn!("IWM hardware read at 0x{:X}", addr);
        let cont = wait_for_keypress_hw("IWM hardware read", addr);
        if !cont {
            // single-step mode: pause after this instruction
            SINGLE_STEP.store(true, Ordering::SeqCst);
        }
        let iwm = IWM_INSTANCE.lock().unwrap();
        return iwm.read(addr);
    }
    // SCC_RD: ((addr & 0xF00000) == 0x900000)
    if (addr & 0xF00000) == 0x900000 {
        log::warn!("SCC_RD hardware read at 0x{:X}", addr);
        let cont = wait_for_keypress_hw("SCC_RD hardware read", addr);
        if !cont {
            SINGLE_STEP.store(true, Ordering::SeqCst);
        }
    }
    // SCC_WR: ((addr & 0xF00000) == 0xB00000)
    if (addr & 0xF00000) == 0xB00000 {
        log::warn!("SCC_WR hardware read at 0x{:X}", addr);
        let cont = wait_for_keypress_hw("SCC_WR hardware read", addr);
        if !cont {
            SINGLE_STEP.store(true, Ordering::SeqCst);
        }
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
            ROM[addr as usize]
        } else if addr >= ROM_BASE && addr < ROM_BASE + ROM_SIZE as u32 {
            ROM[(addr - ROM_BASE) as usize]
        } else if addr < RAM_SIZE as u32 {
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
        let cont = wait_for_keypress_hw("IWM hardware write", addr);
        if !cont {
            SINGLE_STEP.store(true, Ordering::SeqCst);
        }
        let mut iwm = IWM_INSTANCE.lock().unwrap();
        iwm.write(addr, value);
        return;
    }
    // SCC_RD: ((addr & 0xF00000) == 0x900000)
    if (addr & 0xF00000) == 0x900000 {
        log::warn!("SCC_RD hardware write at 0x{:X} = 0x{:X}", addr, value);
        let cont = wait_for_keypress_hw("SCC_RD hardware write", addr);
        if !cont {
            SINGLE_STEP.store(true, Ordering::SeqCst);
        }
    }
    // SCC_WR: ((addr & 0xF00000) == 0xB00000)
    if (addr & 0xF00000) == 0xB00000 {
        log::warn!("SCC_WR hardware write at 0x{:X} = 0x{:X}", addr, value);
        let cont = wait_for_keypress_hw("SCC_WR hardware write", addr);
        if !cont {
            SINGLE_STEP.store(true, Ordering::SeqCst);
        }
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
            let _ = wait_for_keypress_hw("write_u8 attempt to write to ROM@0", addr);
            warn!("write_u8 attempt to write to ROM@0: 0x{:X}", addr);
        } else if addr >= ROM_BASE && addr < ROM_BASE + ROM_SIZE as u32 {
            let _ = wait_for_keypress_hw("write_u8 attempt to write to ROM@400000", addr);
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
