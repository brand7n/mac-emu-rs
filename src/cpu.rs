use crate::memory::{read_u8, read_u16, write_u8, write_u16, read_u32, write_u32};
use log::info;
use std::ffi::CStr;
use std::sync::atomic::Ordering;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[no_mangle]
pub extern "C" fn m68k_read_memory_8(address: u32) -> u8 {
    read_u8(address)
}
#[no_mangle]
pub extern "C" fn m68k_read_memory_16(address: u32) -> u16 {
    read_u16(address)
}
#[no_mangle]
pub extern "C" fn m68k_write_memory_8(address: u32, value: u8) {
    write_u8(address, value)
}
#[no_mangle]
pub extern "C" fn m68k_write_memory_16(address: u32, value: u16) {
    write_u16(address, value)
}

#[no_mangle]
pub extern "C" fn m68k_read_memory_32(address: u32) -> u32 {
    read_u32(address)
}

#[no_mangle]
pub extern "C" fn m68k_write_memory_32(address: u32, value: u32) {
    write_u32(address, value)
}

#[no_mangle]
pub extern "C" fn instruction_hook_callback(address: u32) {
    //info!("Executing instruction at: 0x{:X} {}", address, disassemble_instruction(address));
    // info!("Bytes: {:02X} {:02X} {:02X} {:02X}", 
    //     read_u8(address),
    //     read_u8(address + 1),
    //     read_u8(address + 2), 
    //     read_u8(address + 3)
    // );
    //display_registers();
}

pub fn init() {
    info!("Initializing CPU...");
    unsafe {
        m68k_init();
        info!("CPU initialized.");
        m68k_set_cpu_type(M68K_CPU_TYPE_68000);
        info!("CPU type set to 68000.");
        m68k_set_instr_hook_callback(Some(instruction_hook_callback));
        info!("Instruction hook set.");
        m68k_pulse_reset();
        info!("CPU reset complete. Initial PC: 0x{:X}", get_pc());
    }
}

pub fn step(cycles: i32) -> i32 {
    use crate::memory::{SINGLE_STEP, wait_for_keypress_hw};
    let mut cycles_left = cycles;
    let mut total_cycles = 0;
    while cycles_left > 0 {
        if SINGLE_STEP.load(Ordering::SeqCst) {
            let pc = get_pc();
            let _ = wait_for_keypress_hw("Single-step", pc);
        }
        let executed = unsafe { m68k_execute(1) };
        if executed <= 0 {
            break;
        }
        cycles_left -= executed;
        total_cycles += executed;
    }
    total_cycles
}

pub fn get_reg(reg: m68k_register_t) -> u32 {
    unsafe { m68k_get_reg(core::ptr::null_mut(), reg) }
}

pub fn get_pc() -> u32 {
    unsafe { m68k_get_reg(core::ptr::null_mut(), m68k_register_t_M68K_REG_PC) }
}

pub fn disassemble_instruction(pc: u32) -> String {
    let mut buffer = [0u8; 100];
    let mut pc_copy = pc;
    unsafe {
        m68k_disassemble(buffer.as_mut_ptr() as *mut i8, pc_copy, M68K_CPU_TYPE_68000);
        CStr::from_ptr(buffer.as_ptr() as *const i8)
            .to_string_lossy()
            .into_owned()
    }
}

pub fn display_registers() {
    println!("\nRegisters:");
    println!("PC: 0x{:08X}", get_reg(m68k_register_t_M68K_REG_PC));
    println!("SR: 0x{:04X}", get_reg(m68k_register_t_M68K_REG_SR));
    println!("D0: 0x{:08X}  D1: 0x{:08X}  D2: 0x{:08X}  D3: 0x{:08X}", 
        get_reg(m68k_register_t_M68K_REG_D0), get_reg(m68k_register_t_M68K_REG_D1), 
        get_reg(m68k_register_t_M68K_REG_D2), get_reg(m68k_register_t_M68K_REG_D3));
    println!("D4: 0x{:08X}  D5: 0x{:08X}  D6: 0x{:08X}  D7: 0x{:08X}", 
        get_reg(m68k_register_t_M68K_REG_D4), get_reg(m68k_register_t_M68K_REG_D5), 
        get_reg(m68k_register_t_M68K_REG_D6), get_reg(m68k_register_t_M68K_REG_D7));
    println!("A0: 0x{:08X}  A1: 0x{:08X}  A2: 0x{:08X}  A3: 0x{:08X}", 
        get_reg(m68k_register_t_M68K_REG_A0), get_reg(m68k_register_t_M68K_REG_A1), 
        get_reg(m68k_register_t_M68K_REG_A2), get_reg(m68k_register_t_M68K_REG_A3));
    println!("A4: 0x{:08X}  A5: 0x{:08X}  A6: 0x{:08X}  A7: 0x{:08X}", 
        get_reg(m68k_register_t_M68K_REG_A4), get_reg(m68k_register_t_M68K_REG_A5), 
        get_reg(m68k_register_t_M68K_REG_A6), get_reg(m68k_register_t_M68K_REG_A7));
    println!();
}

// pub fn set_reg(reg: m68k_register_t, value: u32) {
//     unsafe { m68k_set_reg(reg, value) }
// }
