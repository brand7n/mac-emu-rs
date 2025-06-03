mod cpu;
mod memory;
mod video;

use cpu::{init_cpu, step};
use memory::{write_u16, write_u32, RAM_SIZE};
use video::MacVideo;
use std::time::{Duration, Instant};

const CYCLES_PER_BATCH: i32 = 128;
const TARGET_FPS: u32 = 60;
const FRAME_TIME: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);

//#[tokio::main]
fn main() {
    env_logger::init();

    // Set up reset vectors
    // Vector 0: Initial Stack Pointer (top of RAM)
    write_u32(0x000000, RAM_SIZE as u32);
    // Vector 1: Initial PC (our test instruction)
    write_u32(0x000004, 0x1000);

    // Set up test loop
    write_u16(0x1000, 0x7042);  // $42 -> D0
    write_u16(0x1002, 0x4ef8);  // JMP $1000
    write_u16(0x1004, 0x1000);

    // Initialize CPU (will read vectors from 0x000000 and 0x000004)
    init_cpu();

    // Initialize video
    let (video, event_loop) = MacVideo::new();
    
    // Run the video event loop with our emulation logic
    video.run(event_loop, || {
        let mut total_cycles = 0;
        let frame_start = Instant::now();
        
        // Process cycles in batches
        for _ in 0..(TARGET_FPS as i32 * CYCLES_PER_BATCH) {
            let cycles = step(CYCLES_PER_BATCH);
            total_cycles += cycles;
        }
        
        // Calculate time to wait for next frame
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - elapsed);
        }
        
        // Optional: Print some debug info every second
        static mut LAST_FRAME_TIME: Option<Instant> = None;
        unsafe {
            if let Some(last_time) = LAST_FRAME_TIME {
                if frame_start.duration_since(last_time) >= Duration::from_secs(1) {
                    println!("Cycles per second: {}", total_cycles);
                    LAST_FRAME_TIME = Some(frame_start);
                }
            } else {
                LAST_FRAME_TIME = Some(frame_start);
            }
        }
    });
}
