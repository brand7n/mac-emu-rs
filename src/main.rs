mod cpu;
mod memory;
mod video;

use cpu::{init, step};
use memory::{write_u16, write_u32, RAM_SIZE, load_rom};
use video::MacVideo;
use std::time::{Duration, Instant};
use std::env;
use log::{info, error};

const CYCLES_PER_BATCH: i32 = 128;
const TARGET_FPS: u32 = 60;
const FRAME_TIME: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);

//#[tokio::main]
fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        error!("Usage: {} [path_to_rom]", args[0]);
        return;
    }

    if let Err(e) = memory::load_rom(&args[1]) {
        error!("Error loading ROM: {}", e);
        return;
    }

    // Set up reset vectors
    // Vector 0: Initial Stack Pointer (top of RAM)
    // write_u32(0x000000, RAM_SIZE as u32);
    // // Vector 1: Initial PC (our test instruction)
    // write_u32(0x000004, 0x1000);

    // // Set up test loop
    // write_u16(0x1000, 0x7042);  // $42 -> D0
    // write_u16(0x1002, 0x4ef8);  // JMP $1000
    // write_u16(0x1004, 0x1000);

    // Initialize CPU (will read vectors from 0x000000 and 0x000004)
    init();

    // Initialize video
    let (video, event_loop) = MacVideo::new();
    
    // Run the video event loop, which calls the CPU execution step
    video.run(event_loop, || {
        let cycles = step(CYCLES_PER_BATCH);
        info!("Cycles executed: {}", cycles);
    });
}
