#![no_std]
#![no_main]

use core::panic::PanicInfo;

const WIDTH: usize = 80;
const HEIGHT: usize = 24;
static mut DROPS: [isize; WIDTH] = [-1; WIDTH];

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[polkavm_derive::polkavm_import]
extern "C" {
    fn render_char(x: u32, y: u32, c: u32) -> u32;
    fn clear_screen() -> u32;
    fn render_screen() -> u32;
}

#[no_mangle]
#[polkavm_derive::polkavm_export]
extern "C" fn start_matrix_effect() {
    loop {
        clear();
        update_drops();
        render();
        delay(10000); // Adjust delay value as needed
    }
}

fn clear() {
    unsafe {
        clear_screen();
    }
}

fn render() {
    unsafe {
        render_screen();
    }
}

fn update_drops() {
    unsafe {
        for (x, drop) in DROPS.iter_mut().enumerate().take(WIDTH) {
            if *drop < 0 && random_byte() % 10 < 1 {
                *drop = 0;
            }

            if *drop < 0 {
                continue;
            }

            if *drop as usize >= HEIGHT {
                *drop = -1;
                continue;
            }

            let y = *drop as u32;
            let c = (random_byte() % 94 + 33) as u32;
            render_char(x as u32, y, c);
            *drop += 1;
        }
    }
}

fn delay(count: u32) {
    for _ in 0..count {
        // Simple delay loop to create a pause in execution
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

// A simple pseudo-random number generator function
#[no_mangle]
fn random_byte() -> u8 {
    static mut SEED: u32 = 123456789;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        (SEED >> 16) as u8
    }
}
