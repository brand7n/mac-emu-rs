use crate::memory::{read_u8, read_u16, write_u8, write_u16, read_u32, write_u32};

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

pub fn init_cpu() {
    unsafe {
        println!("Initializing CPU...");
        m68k_init();
        println!("CPU initialized.");
        m68k_set_cpu_type(M68K_CPU_TYPE_68000);
        println!("CPU type set.");
        m68k_pulse_reset();
        println!("CPU reset complete.");
    }
}

pub fn step(cycles: i32) -> i32 {
    unsafe { m68k_execute(cycles) }
}

pub fn get_reg(reg: u32) -> u32 {
    unsafe { m68k_get_reg(core::ptr::null_mut(), reg) }
}

// pub fn set_reg(reg: i32, value: u32) {
//     //unsafe { m68k_set_reg(reg, value) }
// }
