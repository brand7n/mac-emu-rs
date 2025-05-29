mod cpu;
mod memory;
mod video;

use cpu::{init_cpu, step};
use memory::{RAM, VIDEO_BASE};
use video::MacVideo;

//#[tokio::main]
fn main() {
    env_logger::init();

    //load_rom("roms/mac128k.rom");
    init_cpu();

    // Test pattern
    unsafe {
        for i in 0..(512 * 342 / 8) {
            RAM[VIDEO_BASE + i] = if i % 2 == 0 { 0xAA } else { 0x55 };
        }
    }

    let (video, event_loop) = MacVideo::new();
    video.run(event_loop);
}
