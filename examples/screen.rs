#![allow(unstable)]

extern crate screenshot;
extern crate bmp;

use std::io::fs::File;
use screenshot::{Screenshot, get_screenshot};
use bmp::{Image, Pixel};

fn main() {
	let s: Screenshot = get_screenshot(0);

	println!("{} x {} x {} = {} bytes", s.height(), s.width(), s.pixel_width(), s.raw_len());

	let mut img = Image::new(s.height(), s.width());
	for row in range(0, s.height()) {
		for col in range(0, s.width()) {
			let (r, g, b) = s.get_pixel(row, col);
			img.set_pixel(row, col, Pixel {r: r, g: g, b: b});
		}
	}
	img.save("/Users/alex/Desktop/test.bmp");
}
