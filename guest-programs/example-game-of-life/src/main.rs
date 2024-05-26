#![no_std]
#![no_main]

use core::panic::PanicInfo;

const WIDTH: usize = 80;
const HEIGHT: usize = 24;

static mut GRID: [[bool; WIDTH]; HEIGHT] = [[false; WIDTH]; HEIGHT];
static mut NEW_GRID: [[bool; WIDTH]; HEIGHT] = [[false; WIDTH]; HEIGHT];

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[polkavm_derive::polkavm_import]
extern "C" {
    fn set_pixel(x: u32, y: u32, value: u32) -> u32;
    fn clear_screen() -> u32;
    fn render_screen() -> u32;
}

#[no_mangle]
#[polkavm_derive::polkavm_export]
extern "C" fn start_game_of_life() {
    initialize_grid();
    loop {
        clear_and_render();
        update_grid();
    }
}

// Encoded initial patterns
const PATTERNS: &[(usize, usize)] = &[
    // Top left block
    (5, 1),
    (5, 2),
    (6, 1),
    (6, 2),
    // Left glider
    (3, 13),
    (3, 14),
    (4, 12),
    (4, 16),
    (5, 11),
    (5, 17),
    (6, 11),
    (6, 15),
    (6, 17),
    (6, 18),
    (7, 11),
    (7, 17),
    (8, 12),
    (8, 16),
    (9, 13),
    (9, 14),
    // Right glider
    (1, 25),
    (2, 23),
    (2, 25),
    (3, 21),
    (3, 22),
    (4, 21),
    (4, 22),
    (5, 21),
    (5, 22),
    (6, 23),
    (6, 25),
    (7, 25),
    // Right block
    (3, 35),
    (3, 36),
    (4, 35),
    (4, 36),
];

fn initialize_grid() {
    unsafe {
        for &(y, x) in PATTERNS.iter() {
            GRID[y][x] = true;
        }
    }
}

fn clear_and_render() {
    unsafe {
        clear_screen();
    }
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let value = unsafe { GRID[y][x] };
            unsafe {
                set_pixel(x as u32, y as u32, value as u32);
            }
        }
    }
    unsafe {
        render_screen();
    }
}

fn update_grid() {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let live_neighbors = count_live_neighbors(x, y);
            unsafe {
                NEW_GRID[y][x] = match (GRID[y][x], live_neighbors) {
                    (true, 2) | (_, 3) => true,
                    _ => false,
                };
            }
        }
    }

    unsafe {
        core::ptr::swap(&mut GRID, &mut NEW_GRID);
    }
}

#[inline(always)]
fn count_live_neighbors(x: usize, y: usize) -> usize {
    let mut count = 0;
    for dy in [-1, 0, 1].iter().cloned() {
        for dx in [-1, 0, 1].iter().cloned() {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = (x as isize + dx).rem_euclid(WIDTH as isize) as usize;
            let ny = (y as isize + dy).rem_euclid(HEIGHT as isize) as usize;
            if unsafe { GRID[ny][nx] } {
                count += 1;
            }
        }
    }
    count
}
