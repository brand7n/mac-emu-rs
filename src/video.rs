use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use crate::memory::{RAM, VIDEO_BASE};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 342;

pub struct MacVideo {
    pixels: Pixels,
    window: winit::window::Window,
}

impl MacVideo {
    pub fn new() -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Mac 128K Emulator")
            .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();

        (MacVideo { pixels, window }, event_loop)
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _ => {}
                },
                Event::RedrawRequested(_) => {
                    let frame = self.pixels.frame_mut();

                    unsafe {
                        for y in 0..HEIGHT as usize {
                            for x in 0..WIDTH as usize {
                                let offset = (y * (WIDTH as usize / 8)) + (x / 8);
                                let byte = RAM.get(VIDEO_BASE + offset).copied().unwrap_or(0);
                                let bit = 7 - (x % 8);
                                let pixel_on = (byte >> bit) & 1 != 0;

                                let idx = (y * WIDTH as usize + x) * 4;
                                let color = if pixel_on { 0x00 } else { 0xFF };
                                frame[idx..idx + 4].copy_from_slice(&[color, color, color, 0xFF]);
                            }
                        }
                    }

                    self.pixels.render().unwrap();
                }
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}
