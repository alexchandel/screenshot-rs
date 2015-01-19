# screenshot-rs
Get a bitmap image of any display in Rust. This crate is hosted on [crates.io](https://crates.io/crates/screenshot).

Contributions welcome!

## Known Issues

* The BMP Image in the example is rotated +90 degrees because I don't adjust for BMP idiosyncrasy.
* The PNG Image in the example has its R & B channels exchanged, due to an issue in `image`.
