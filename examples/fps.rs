extern crate screenshot;

use std::time;
use screenshot::Screener;

fn main() {
    let mut fps = 0;
let mut sum_frame_time = 0;

    let mut screener = unsafe { Screener::new(0) };

    loop {
        let start_time = time::SystemTime::now();
        let _s = screener.get_screenshot().unwrap();

        let elapsed_time = start_time.elapsed().unwrap().as_millis();
        // println!("Frame rate: {}", elapsed_time);
        let framerate_offset = 40 - elapsed_time as i32;
        if framerate_offset.is_positive() {
            std::thread::sleep(std::time::Duration::from_millis(framerate_offset as u64))
        }

        sum_frame_time += 40;

        if sum_frame_time >= 1000 {
            println!("FPS: {}", fps);
            fps = 0;
            sum_frame_time = 0;
        } else {
            fps += 1;
        }
    }
}