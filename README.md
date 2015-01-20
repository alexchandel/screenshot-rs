# screenshot-rs
Get a bitmap image of any display in Rust. This crate is hosted on [crates.io](https://crates.io/crates/screenshot).

Contributions welcome!

## Examples

```rust
extern crate image;
extern crate screenshot;
use screenshot::get_screenshot;

fn main() {
	let s = get_screenshot(0).unwrap();

	println!("{} x {}", s.width(), s.height());

	image::save_buffer(&Path::new("test.png"),
		s.as_slice(), s.width() as u32, s.height() as u32, image::RGBA(8))
	.unwrap();
}
```

## Development
* screenshot-rs has its own systems bindings. I want to depend on [servo/rust-core-graphics](https://github.com/servo/rust-core-graphics) and [klutzy/rust-windows](https://github.com/klutzy/rust-windows), but neither supports Cargo.
* There is no Linux support.
* Screenshot should provide a `container_as_bytes()` or an `as_slice()` method, rather than requiring unsafe access to its Vec buffer.

## Known Issues
* The BMP Image in the example is rotated +90 degrees because I don't adjust for BMP idiosyncrasy.
* The PNG Image in the example has its R & B channels exchanged because `PistonDevelopers/image` doesn't support ARGB pixels.
