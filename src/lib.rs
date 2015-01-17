//! Capture a bitmap image of a display. The resulting screenshot is stored in
//! the `Screenshot` type, which varies per platform.
//!
//! TODO Windows & Linux support. Contributions welcome.

extern crate libc;

pub use ffi::{Screenshot, get_screenshot};

#[cfg(target_os = "macos")]
mod ffi {
	use std::intrinsics::offset;
	use std::ops::Drop;
	use libc;

	type CFIndex = libc::c_long;
	type CFDataRef = *const u8; // *const CFData

	#[cfg(target_arch = "x86")]
	type CGFloat = libc::c_float;
	#[cfg(target_arch = "x86_64")]
	type CGFloat = libc::c_double;
	type CGError = libc::int32_t;

	type CGDirectDisplayID = libc::uint32_t;
	type CGDisplayCount = libc::uint32_t;
	type CGImageRef = *mut u8; // *mut CGImage
	type CGDataProviderRef = *mut u8; // *mut CGDataProvider

	const kCGErrorSuccess: CGError = 0;
	const kCGErrorFailure: CGError = 1000;
	const CGDisplayNoErr: CGError = kCGErrorSuccess;

	#[link(name = "CoreGraphics", kind = "framework")]
	extern "C" {
		fn CGGetActiveDisplayList(max_displays: libc::uint32_t,
	                              active_displays: *mut CGDirectDisplayID,
	                              display_count: *mut CGDisplayCount) -> CGError;
		fn CGDisplayCreateImage(displayID: CGDirectDisplayID) -> CGImageRef;
		fn CGImageRelease(image: CGImageRef);

		fn CGImageGetBitsPerComponent(image: CGImageRef) -> libc::size_t;
		fn CGImageGetBitsPerPixel(image: CGImageRef) -> libc::size_t;
		fn CGImageGetBytesPerRow(image: CGImageRef) -> libc::size_t;
		fn CGImageGetDataProvider(image: CGImageRef) -> CGDataProviderRef;
		fn CGImageGetHeight(image: CGImageRef) -> libc::size_t;
		fn CGImageGetWidth (image: CGImageRef) -> libc::size_t;

		fn CGDataProviderCopyData(provider: CGDataProviderRef) -> CFDataRef;
	}
	#[link(name = "CoreFoundation", kind = "framework")]
	extern "C" {
		fn CFDataGetLength (theData: CFDataRef) -> CFIndex;
		fn CFDataGetBytePtr(theData: CFDataRef) -> *const u8;
		fn CFRelease(cf: *const libc::c_void);
	}

	pub struct Screenshot {
		cf_data: CFDataRef,
		height: usize,
		width: usize,
		row_len: usize,
		pixel_width: usize,
	}

	/// An image buffer containing the screenshot.
	impl Screenshot {
		/// Height of image in pixels.
		pub fn height(&self) -> usize { self.height }

		/// Width of image in pixels.
		pub fn width(&self) -> usize { self.width }

		/// Number of pixels in one row of bitmap.
		pub fn row_len(&self) -> usize { self.row_len }

		/// Width of pixel in bytes.
		pub fn pixel_width(&self) -> usize { self.pixel_width }

		/// Raw bitmap.
		pub unsafe fn raw_data(&self) -> *const u8 {
			CFDataGetBytePtr(self.cf_data)
		}

		/// Returns an RGB tuple.
		pub fn get_pixel(&self, number: usize) -> (u8, u8, u8) {
			unsafe {
				let data = self.raw_data();
				let len = CFDataGetLength(data) as usize;
				let width = self.pixel_width();
				if number > len { panic!("Bounds overflow"); }
				let idx = (len*width) as isize;
				(
					*offset(data, idx),
					*offset(data, idx+1),
					*offset(data, idx+2),
				)
			}
		}
	}

	impl Drop for Screenshot {
		fn drop(&mut self) {
			unsafe {CFRelease(self.cf_data as *const libc::c_void);}
		}
	}

	/// Get a screenshot of the requested display.
	pub fn get_screenshot(screen: usize) -> Screenshot {
		let mut count: CGDisplayCount = 0;
		let mut err = CGDisplayNoErr;
		unsafe {
			err = CGGetActiveDisplayList(0,
				0 as *mut CGDirectDisplayID,
				&mut count);
		};

		let mut disps: Vec<CGDisplayCount> = Vec::with_capacity(count as usize);
		unsafe {
			disps.set_len(count as usize);
			let err = CGGetActiveDisplayList(disps.len() as libc::uint32_t,
				&mut disps[0] as *mut CGDirectDisplayID,
				&mut count);
		};

		if err != CGDisplayNoErr {
			panic!("CoreGraphics reported an error.");
		}

		let disp_id = disps[screen];

		unsafe {
			let cg_img = CGDisplayCreateImage(disp_id);

			let pixwid = CGImageGetBitsPerPixel(cg_img);
			if pixwid % 8 != 0 { panic!("Pixels aren't integral bytes."); }

			let img = Screenshot {
				cf_data: CGDataProviderCopyData(CGImageGetDataProvider(cg_img)),
				height: CGImageGetHeight(cg_img) as usize,
				width: CGImageGetWidth(cg_img) as usize,
				row_len: CGImageGetBytesPerRow(cg_img) as usize,
				pixel_width: (pixwid/8) as usize
			};
			CGImageRelease(cg_img); // should release Image + DataProvider
			img
		}
	}
}
