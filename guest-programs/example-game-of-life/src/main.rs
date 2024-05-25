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
        clear();
        update_grid();
        render();
        delay(10000);
    }
}

fn initialize_grid() {
    unsafe {
        // Clear the grid first
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                GRID[y][x] = false;
            }
        }

        // Top left block
        GRID[5][1] = true;
        GRID[5][2] = true;
        GRID[6][1] = true;
        GRID[6][2] = true;

        // Left glider
        GRID[3][13] = true;
        GRID[3][14] = true;
        GRID[4][12] = true;
        GRID[4][16] = true;
        GRID[5][11] = true;
        GRID[5][17] = true;
        GRID[6][11] = true;
        GRID[6][15] = true;
        GRID[6][17] = true;
        GRID[6][18] = true;
        GRID[7][11] = true;
        GRID[7][17] = true;
        GRID[8][12] = true;
        GRID[8][16] = true;
        GRID[9][13] = true;
        GRID[9][14] = true;

        // Right glider
        GRID[1][25] = true;
        GRID[2][23] = true;
        GRID[2][25] = true;
        GRID[3][21] = true;
        GRID[3][22] = true;
        GRID[4][21] = true;
        GRID[4][22] = true;
        GRID[5][21] = true;
        GRID[5][22] = true;
        GRID[6][23] = true;
        GRID[6][25] = true;
        GRID[7][25] = true;

        // Right block
        GRID[3][35] = true;
        GRID[3][36] = true;
        GRID[4][35] = true;
        GRID[4][36] = true;
    }
}

fn clear() {
    unsafe { clear_screen(); }
}

fn render() {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let value = unsafe { GRID[y][x] };
            unsafe { set_pixel(x as u32, y as u32, value as u32); }
        }
    }
    unsafe { render_screen(); }
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

    // Copy new grid to current grid
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            unsafe {
                GRID[y][x] = NEW_GRID[y][x];
            }
        }
    }
}

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

fn delay(count: u32) {
    for _ in 0..count {
        // Simple delay loop
    }
}
