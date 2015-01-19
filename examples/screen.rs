#![allow(unstable)]

extern crate screenshot;
extern crate bmp;
extern crate image;

use screenshot::{Screenshot, get_screenshot};
use bmp::{Image, Pixel};

fn main() {
	let s: Screenshot = get_screenshot(0).unwrap();

	println!("{} x {} x {} = {} bytes", s.height(), s.width(), s.pixel_width(), s.raw_len());

	let mut img = Image::new(s.height(), s.width());
	for row in range(0, s.height()) {
		for col in range(0, s.width()) {
			let p = s.get_pixel(row, col);
			img.set_pixel(row, col, Pixel {r: p.r, g: p.g, b: p.b});
		}
	}
	img.save("test.bmp");

	unsafe {
		let rp = &s.raw_data();
		let buff = std::slice::from_raw_buf(rp, s.raw_len());
		image::save_buffer(&Path::new("test.png"),
			buff, s.width() as u32, s.height() as u32, image::RGBA(8));
	}
}
