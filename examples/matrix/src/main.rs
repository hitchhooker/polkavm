use polkavm::{CallArgs, Config, Engine, Linker, Module, ProgramBlob, StateArgs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const WIDTH: usize = 80;
const HEIGHT: usize = 24;

#[derive(Clone)]
struct FrameBuffer {
    buffer: Arc<Mutex<[[char; WIDTH]; HEIGHT]>>,
}

impl FrameBuffer {
    fn new() -> Self {
        FrameBuffer {
            buffer: Arc::new(Mutex::new([[' '; WIDTH]; HEIGHT])),
        }
    }

    fn clear(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                buffer[y][x] = ' ';
            }
        }
    }

    fn render(&self) {
        let buffer = self.buffer.lock().unwrap();
        print!("\x1B[2J\x1B[H"); // Clear the screen and move cursor to top-left corner
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                print!("{}", buffer[y][x]);
            }
            println!();
        }
    }

    fn render_char(&self, x: u32, y: u32, c: u32) -> u32 {
        if x < WIDTH as u32 && y < HEIGHT as u32 {
            let mut buffer = self.buffer.lock().unwrap();
            buffer[y as usize][x as usize] = c as u8 as char;
        }
        0
    }
}

fn main() {
    env_logger::init();

    let frame_buffer = FrameBuffer::new();

    let raw_blob = include_bytes!("../../../guest-programs/output/example-matrix.polkavm");
    let blob = ProgramBlob::parse(raw_blob[..].into()).unwrap();

    let config = Config::from_env().unwrap();
    let engine = Engine::new(&config).unwrap();
    let module = Module::from_blob(&engine, &Default::default(), blob).unwrap();
    let mut linker = Linker::new(&engine);

    // Define host functions
    let frame_buffer_clone = frame_buffer.clone();
    linker
        .func_wrap("render_char", move |x: u32, y: u32, c: u32| -> u32 {
            frame_buffer_clone.render_char(x, y, c)
        })
        .unwrap();

    let frame_buffer_clone = frame_buffer.clone();
    linker
        .func_wrap("clear_screen", move || -> u32 {
            frame_buffer_clone.clear();
            0
        })
        .unwrap();

    let frame_buffer_clone = frame_buffer.clone();
    linker
        .func_wrap("render_screen", move || -> u32 {
            frame_buffer_clone.render();
            0
        })
        .unwrap();

    // Link the host functions with the module
    let instance_pre = linker.instantiate_pre(&module).unwrap();
    let instance = instance_pre.instantiate().unwrap();

    // Start the rendering in a separate thread
    let frame_buffer_clone = frame_buffer.clone();
    thread::spawn(move || loop {
        frame_buffer_clone.render();
        thread::sleep(Duration::from_millis(100)); // Adjust the refresh rate
    });

    // Call the guest function to trigger the matrix effect
    let export_index = instance.module().lookup_export("start_matrix_effect").unwrap();
    #[allow(clippy::let_unit_value)]
    let mut user_data = ();
    let call_args = CallArgs::new(&mut user_data, export_index);
    instance.call(StateArgs::new(), call_args).unwrap();
}
