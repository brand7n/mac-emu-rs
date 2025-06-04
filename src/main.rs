mod cpu;
mod memory;
mod video;
mod via;
use via::{Via, ViaCallbacks, set_via};

use cpu::{init, step, get_pc, display_registers};
use memory::{write_u16, write_u32, RAM_SIZE, load_rom};
use video::MacVideo;
use std::time::{Duration, Instant};
use std::env;
use log::{info, error};
use std::io::{self, Write};

const CYCLES_PER_BATCH: i32 = 1024;
const TARGET_FPS: u32 = 60;
const FRAME_TIME: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);

fn wait_for_keypress() {
    print!("Press Enter to continue...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn dummy_irq_set(_irq: bool) {}

//#[tokio::main]
fn main() {
    // Set default log level if not specified
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
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

    // Initialize test pattern in video memory
    for y in 0..342 {
        for x in 0..64 {
            let offset = (y * 64) + x;
            // Create 8x8 pixel squares by dividing coordinates by 8
            let value = if ((x / 8) + (y / 8)) % 2 == 0 { 0xFF } else { 0x00 };
            memory::write_u8(0x1A700 + offset as u32, value);
        }
    }

    // Initialize CPU (will read vectors from 0x000000 and 0x000004)
    init();
    wait_for_keypress();

    // Initialize video
    let (video, event_loop) = MacVideo::new();
    
    // Initialize VIA
    let via = Via::new(ViaCallbacks {
        ra_change: None,
        rb_change: None,
        ra_in: None,
        rb_in: None,
        sr_tx: None,
        irq_set: dummy_irq_set,
    });
    set_via(via);

    // Run the video event loop, which calls the CPU execution step
    video.run(event_loop, || {
        let _ = step(CYCLES_PER_BATCH);
        display_registers();
        //wait_for_keypress();
    });
}
