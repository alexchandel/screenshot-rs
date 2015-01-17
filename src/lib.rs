//! Capture a bitmap image of a display. The resulting screenshot is stored in
//! the `Screenshot` type, which varies per platform.
//!
//! TODO Windows & Linux support. Contributions welcome.

#![allow(unstable, unused_assignments)]

extern crate libc;

pub use ffi::{Screenshot, get_screenshot};

#[cfg(target_os = "macos")]
mod ffi {
	#![allow(non_upper_case_globals, dead_code)]

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
		cg_img: CGImageRef,
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

		pub fn raw_len(&self) -> usize {
			unsafe { CFDataGetLength(self.cf_data) as usize }
		}

		pub fn get_pixel(&self, row: usize, col: usize) -> (u8, u8, u8) {
			unsafe {
				let data = self.raw_data();
				let len = self.raw_len();
				let idx = (row*self.row_len() + col*self.pixel_width()) as isize;
				if idx as usize > len { panic!("Bounds overflow"); }
				// Natively OS X has an ARGB pixel, where blue is lowest.
				(
					*offset(data, idx+2),
					*offset(data, idx+1),
					*offset(data, idx),
				)
			}
		}

		/// Returns an RGB tuple.
		pub fn get_index(&self, number: usize) -> (u8, u8, u8) {
			unsafe {
				let data = self.raw_data();
				let len = self.raw_len();
				let idx = (number*self.pixel_width()) as isize;
				if idx as usize > len { panic!("Bounds overflow"); }
				(
					*offset(data, idx+2),
					*offset(data, idx+1),
					*offset(data, idx),
				)
			}
		}
	}

	impl Drop for Screenshot {
		fn drop(&mut self) {
			unsafe {
				CGImageRelease(self.cg_img); // should've just released Image + DataProvider
				CFRelease(self.cf_data as *const libc::c_void);
			}
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
			err = CGGetActiveDisplayList(disps.len() as libc::uint32_t,
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
				cg_img: cg_img,
				cf_data: CGDataProviderCopyData(CGImageGetDataProvider(cg_img)),
				height: CGImageGetHeight(cg_img) as usize,
				width: CGImageGetWidth(cg_img) as usize,
				row_len: CGImageGetBytesPerRow(cg_img) as usize,
				pixel_width: (pixwid/8) as usize
			};
			img
		}
	}
}

#[cfg(target_os = "windows")]
mod ffi {
	use libc::{c_int, c_ulong, c_void};

	type PVOID = *mut c_void;
	type DWORD = c_ulong;
	type BOOL = c_int;
	type HANDLE = PVOID;
	type HWND = HANDLE;
	type HDC = HANDLE;
	type HBITMAP = HANDLE;
	type HGDIOBJ = HANDLE;

	/// TODO verify value
	const SRCCOPY: u32 = 0x00CC0020;

	#[link(name = "gdi32")]
	extern "system" {
		fn GetDC(hWnd: HWND) -> HDC;
		fn CreateCompatibleDC(hdc: HDC) -> HDC;
		fn CreateCompatibleBitmap(hdc: HDC, nWidth: c_int, nHeight: c_int) -> HBITMAP;
		fn SelectObject(hdc: HDC, hgdiobj: HGDIOBJ) -> HGDIOBJ;
		fn BitBlt(hdcDest: HDC, nXDest: c_int, nYDest: c_int, nWidth: c_int, nHeight: c_int,
                  hdcSrc: HDC, nXSrc: c_int, nYSrc: c_int, dwRop: DWORD) -> BOOL;
		fn DeleteObject(hObject: HGDIOBJ) -> BOOL;
	}

	pub struct Screenshot {
		hBmp: HBITMAP,
	}

	impl Drop for Screenshot {
		fn drop(&mut self) {
			unsafe {
				DeleteObject(self.hBmp);
			}
		}
	}

	/// TODO don't ignore screen number
	/// TODO get screen size
	pub fn get_screenshot(screen: usize) -> Screenshot {
		let width = 1024;
		let height = 1024;

		unsafe {
			let NULL = 0 as *mut c_void;
			let hDc = CreateCompatibleDC(NULL);
			let hBmp = CreateCompatibleBitmap(GetDC(NULL), width, height);
			SelectObject(hDc, hBmp);
			BitBlt(hDc, 0, 0, width, height, GetDC(NULL), 0, 0, SRCCOPY);
			Screenshot {
				hBmp: hBmp
			}
		}
	}
}
