mod cpu;
mod memory;
mod video;

use cpu::{init_cpu, step, get_reg};
use memory::{RAM, VIDEO_BASE, write_u16, write_u32, RAM_SIZE};
use video::MacVideo;

//#[tokio::main]
fn main() {
    env_logger::init();

    // Set up reset vectors
    // Vector 0: Initial Stack Pointer (top of RAM)
    write_u32(0x000000, RAM_SIZE as u32);
    // Vector 1: Initial PC (our test instruction)
    write_u32(0x000004, 0x1000);

    // Write test instruction MOVE.W #$1234,D0 at address 0x1000
    // MOVE.W #$1234,D0 = 0x303C 0x1234
    write_u16(0x1000, 0x303C);  // MOVE.W #imm,D0
    write_u16(0x1002, 0x1234);  // Immediate value

    // Initialize CPU (will read vectors from 0x000000 and 0x000004)
    init_cpu();

    // Print PC before stepping
    let pc_before = get_reg(16); // Register 16 is PC
    println!("PC before step: 0x{:X}", pc_before);

    // Step through the instruction (give plenty of cycles)
    let cycles = step(100);
    println!("Executed instruction, used {} cycles", cycles);

    // Print PC after stepping
    let pc_after = get_reg(16);
    println!("PC after step: 0x{:X}", pc_after);

    // Check D0's value
    let d0_value = get_reg(0);  // Register 0 is D0
    println!("D0 = 0x{:X}", d0_value);

    // Test pattern
    unsafe {
        for i in 0..(512 * 342 / 8) {
            RAM[VIDEO_BASE + i] = if i % 2 == 0 { 0xAA } else { 0x55 };
        }
    }

    let (video, event_loop) = MacVideo::new();
    video.run(event_loop);
}
