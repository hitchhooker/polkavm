use polkavm::{CallArgs, Config, Engine, Linker, Module, ProgramBlob, StateArgs};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::os::raw::c_char;

const WIDTH: usize = 80;
const HEIGHT: usize = 24;

#[derive(Clone)]
struct FrameBuffer {
    buffer: Arc<Mutex<[[bool; WIDTH]; HEIGHT]>>,
}

impl FrameBuffer {
    fn new() -> Self {
        FrameBuffer {
            buffer: Arc::new(Mutex::new([[false; WIDTH]; HEIGHT])),
        }
    }

    fn clear(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                buffer[y][x] = false;
            }
        }
    }

    fn render(&self) {
        let buffer = self.buffer.lock().unwrap();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let symbol = if buffer[y][x] { 'x' } else { ' ' };
                print!("{}", symbol);
            }
            println!();
        }
        println!();
    }

    fn set_pixel(&self, x: u32, y: u32, value: bool) -> u32 {
        if x < WIDTH as u32 && y < HEIGHT as u32 {
            let mut buffer = self.buffer.lock().unwrap();
            buffer[y as usize][x as usize] = value;
        } else {
            println!("Invalid coordinates: ({}, {})", x, y);
        }
        0
    }
}

fn log_message(ptr: *const c_char, len: u32) {
    if ptr.is_null() || len == 0 {
        println!("Guest log: (null or empty message)");
        return;
    }

    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let message = String::from_utf8_lossy(slice);
    println!("Guest log: {}", message);
}

fn main() {
    env_logger::init();

    let frame_buffer = FrameBuffer::new();

    let raw_blob = include_bytes!("../../../guest-programs/output/example-game-of-life.polkavm");
    let blob = ProgramBlob::parse(raw_blob[..].into()).unwrap();

    let config = Config::from_env().unwrap();
    let engine = Engine::new(&config).unwrap();
    let module = Module::from_blob(&engine, &Default::default(), blob).unwrap();
    let mut linker = Linker::new(&engine);

    // Define host functions
    linker.func_wrap("set_pixel", {
        let frame_buffer = frame_buffer.clone();
        move |x: u32, y: u32, value: u32| -> u32 {
            let value = value != 0;
            frame_buffer.set_pixel(x, y, value)
        }
    }).unwrap();

    linker.func_wrap("clear_screen", {
        let frame_buffer = frame_buffer.clone();
        move || -> u32 {
            frame_buffer.clear();
            0
        }
    }).unwrap();

    linker.func_wrap("render_screen", {
        let frame_buffer = frame_buffer.clone();
        move || -> u32 {
            frame_buffer.render();
            0
        }
    }).unwrap();

    linker.func_wrap("log_message", move |ptr: u32, len: u32| {
        let ptr = ptr as *const c_char;
        log_message(ptr, len);
        0
    }).unwrap();

    // Link the host functions with the module
    let instance_pre = linker.instantiate_pre(&module).unwrap();
    let instance = instance_pre.instantiate().unwrap();

    // Call the guest function to start the Game of Life
    let export_index = instance.module().lookup_export("start_game_of_life").unwrap();
    let mut user_data = ();
    let call_args = CallArgs::new(&mut user_data, export_index);
    instance.call(StateArgs::new(), call_args).unwrap();
    println!("Game of Life started");

    // Rendering loop
    loop {
        std::thread::sleep(Duration::from_millis(100)); // refresh rate
    }
}
