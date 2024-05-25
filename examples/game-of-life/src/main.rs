use polkavm::{CallArgs, Config, Engine, Linker, Module, ProgramBlob, StateArgs};
use std::sync::{Arc, Mutex};

const WIDTH: usize = 80;
const HEIGHT: usize = 24;

#[derive(Clone)]
struct FrameBuffer(Arc<Mutex<[[bool; WIDTH]; HEIGHT]>>);

impl FrameBuffer {
    fn new() -> Self {
        Self(Arc::new(Mutex::new([[false; WIDTH]; HEIGHT])))
    }

    fn clear(&self) {
        self.0.lock().unwrap().iter_mut().for_each(|row| row.fill(false));
    }

    fn render(&self) {
        print!("\x1B[2J\x1B[H");
        let buffer = self.0.lock().unwrap();
        for row in buffer.iter() {
            for &cell in row.iter() {
                print!("{}", if cell { 'x' } else { ' ' });
            }
            println!();
        }
    }

    fn set_pixel(&self, x: u32, y: u32, value: bool) {
        if x < WIDTH as u32 && y < HEIGHT as u32 {
            self.0.lock().unwrap()[y as usize][x as usize] = value;
        }
    }
}

fn main() {
    let frame_buffer = FrameBuffer::new();
    let raw_blob = include_bytes!("../../../guest-programs/output/example-game-of-life.polkavm");
    let blob = ProgramBlob::parse(raw_blob[..].into()).expect("Failed to parse blob");

    let config = Config::from_env().expect("Failed to get config");
    let engine = Engine::new(&config).expect("Failed to create engine");
    let module = Module::from_blob(&engine, &Default::default(), blob).expect("Failed to create module");
    let mut linker = Linker::new(&engine);

    let fb = frame_buffer.clone();
    linker.func_wrap("set_pixel", move |x: u32, y: u32, value: u32| {
        fb.set_pixel(x, y, value != 0);
        0
    }).expect("Failed to wrap set_pixel");

    let fb = frame_buffer.clone();
    linker.func_wrap("clear_screen", move || {
        fb.clear();
        0
    }).expect("Failed to wrap clear_screen");

    let fb = frame_buffer.clone();
    linker.func_wrap("render_screen", move || {
        fb.render();
        0
    }).expect("Failed to wrap render_screen");

    let instance_pre = linker.instantiate_pre(&module).expect("Failed to instantiate pre");
    let instance = instance_pre.instantiate().expect("Failed to instantiate");

    let export_index = instance.module().lookup_export("start_game_of_life").expect("Failed to lookup export");
    let mut user_data = ();

    loop {
        let call_args = CallArgs::new(&mut user_data, export_index);
        instance.call(StateArgs::new(), call_args).expect("Failed to call instance");
        frame_buffer.render();
    }
}
