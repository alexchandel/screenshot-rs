#![allow(unstable)]

extern crate screenshot;
extern crate bmp;
extern crate image;

use screenshot::get_screenshot;
use bmp::{Image, Pixel};

fn main() {
	let s = get_screenshot(0).unwrap();

	println!("{} x {} x {} = {} bytes", s.height(), s.width(), s.pixel_width(), s.raw_len());

	let origin = s.get_pixel(0, 0);
	println!("(0,0): R: {}, G: {}, B: {}", origin.r, origin.g, origin.b);

	let end_col = s.get_pixel(0, s.width()-1);
	println!("(0,end): R: {}, G: {}, B: {}", end_col.r, end_col.g, end_col.b);

	let opp = s.get_pixel(s.height()-1, s.width()-1);
	println!("(end,end): R: {}, G: {}, B: {}", opp.r, opp.g, opp.b);


	let mut img = Image::new(s.height(), s.width());
	for row in range(0, s.height()) {
		for col in range(0, s.width()) {
			let p = s.get_pixel(row, col);
			img.set_pixel(row, col, Pixel {r: p.r, g: p.g, b: p.b});
		}
	}
	img.save("test.bmp");

	image::save_buffer(&Path::new("test.png"),
		s.as_slice(), s.width() as u32, s.height() as u32, image::RGBA(8))
	.unwrap();
}
