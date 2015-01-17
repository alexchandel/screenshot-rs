//! Capture a bitmap image of a display. The resulting screenshot is stored in
//! the `Screenshot` type, which varies per platform.
//!
//! TODO Windows & Linux support. Contributions welcome.

#![allow(unstable, unused_assignments)]

extern crate libc;

pub use ffi::{Screenshot, get_screenshot};

#[derive(Copy)]
pub struct Pixel {
	pub a: u8,
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

#[cfg(target_os = "macos")]
mod ffi {
	#![allow(non_upper_case_globals, dead_code)]

	use std::intrinsics::offset;
	use std::ops::Drop;
	use libc;
	use ::Pixel;

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

	/// An image buffer containing the screenshot.
	pub struct Screenshot {
		cg_img: CGImageRef, // Probably superfluous
		cf_data: CFDataRef,
		height: usize,
		width: usize,
		row_len: usize, // Might be superfluous
		pixel_width: usize,
	}

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

		pub fn get_pixel(&self, row: usize, col: usize) -> Pixel {
			unsafe {
				let data = self.raw_data();
				let len = self.raw_len();
				let idx = (row*self.row_len() + col*self.pixel_width()) as isize;
				if idx as usize > len { panic!("Bounds overflow"); }
				// Natively OS X has an ARGB pixel, where blue is lowest.
				Pixel {
					a: *offset(data, idx+3),
					r: *offset(data, idx+2),
					g: *offset(data, idx+1),
					b: *offset(data, idx),
				}
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
	#![allow(non_snake_case)]

	use libc::{c_int, c_uint, c_long, c_void};
	use std::intrinsics::{size_of, offset};

	use ::Pixel;

	type PVOID = *mut c_void;
	type LPVOID = *mut c_void;
	type WORD = u16; // c_uint;
	type DWORD = u32; // c_ulong;
	type BOOL = c_int;
	type UINT = c_uint;
	type LONG = c_long;
	type HANDLE = PVOID;
	type HWND = HANDLE;
	type HDC = HANDLE;
	type HBITMAP = HANDLE;
	type HGDIOBJ = HANDLE;

	type LPBITMAPINFO = PVOID; // Hack

	const NULL: *mut c_void = 0 as *mut c_void;
	const SM_CXSCREEN: c_int = 0;
	const SM_CYSCREEN: c_int = 1;

	/// TODO verify value
	const SRCCOPY: u32 = 0x00CC0020;
	const DIB_RGB_COLORS: UINT = 0;
	const BI_RGB: DWORD = 0;

	#[repr(C)]
	struct BITMAPINFOHEADER {
		biSize: DWORD,
		biWidth: LONG,
		biHeight: LONG,
		biPlanes: WORD,
		biBitCount: WORD,
		biCompression: DWORD,
		biSizeImage: DWORD,
		biXPelsPerMeter: LONG,
		biYPelsPerMeter: LONG,
		biClrUsed: DWORD,
		biClrImportant: DWORD,
	}

	#[link(name = "user32")]
	extern "system" {
		fn GetSystemMetrics(m: c_int) -> c_int;
	}

	#[link(name = "gdi32")]
	extern "system" {
		fn GetDC(hWnd: HWND) -> HDC;
		fn CreateCompatibleDC(hdc: HDC) -> HDC;
		fn CreateCompatibleBitmap(hdc: HDC, nWidth: c_int, nHeight: c_int) -> HBITMAP;
		fn SelectObject(hdc: HDC, hgdiobj: HGDIOBJ) -> HGDIOBJ;
		fn BitBlt(hdcDest: HDC, nXDest: c_int, nYDest: c_int, nWidth: c_int, nHeight: c_int,
                  hdcSrc: HDC, nXSrc: c_int, nYSrc: c_int, dwRop: DWORD) -> BOOL;
		fn GetDIBits(hdc: HDC, hbmp: HBITMAP, uStartScan: UINT, cScanLines: UINT,
					 lpvBits: LPVOID, lpbi: LPBITMAPINFO, uUsage: UINT) -> c_int;

		fn DeleteObject(hObject: HGDIOBJ) -> BOOL;
		fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
	}

	pub struct Screenshot {
		data: Vec<u8>,
		height: usize,
		width: usize,
		row_len: usize,
		pixel_width: usize,
	}

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
			&self.data[0] as *const u8
		}

		pub fn raw_len(&self) -> usize {
			self.data.len() * unsafe {size_of::<u8>()}
		}

		pub fn get_pixel(&self, row: usize, col: usize) -> Pixel {
			let idx = (row*self.row_len() + col*self.pixel_width()) as isize;
			unsafe {
				let data = &self.data[0] as *const u8;
				if idx as usize > self.raw_len() { panic!("Bounds overflow"); }
				Pixel {
					a: *offset(data, idx+3),
					r: *offset(data, idx+2),
					g: *offset(data, idx+1),
					b: *offset(data, idx),
				}
			}
		}
	}

	/// TODO don't ignore screen number
	pub fn get_screenshot(_screen: usize) -> Screenshot {
		unsafe {
			let width = GetSystemMetrics(SM_CXSCREEN);
			let height = GetSystemMetrics(SM_CYSCREEN);

			let h_dc_screen = GetDC(NULL);

			let h_dc = CreateCompatibleDC(h_dc_screen);
			let h_bmp = CreateCompatibleBitmap(h_dc_screen, width, height);
			SelectObject(h_dc, h_bmp);
			BitBlt(h_dc, 0, 0, width, height, h_dc_screen, 0, 0, SRCCOPY);

			ReleaseDC(NULL, h_dc_screen); // don't need screen anymore

			let pixel_width = 4; // FIXME
			let size = (width*height) as usize * pixel_width;
			let mut data: Vec<u8> = Vec::with_capacity(size);
			data.set_len(size);

			let mut bmi = BITMAPINFOHEADER {
				biSize: size_of::<BITMAPINFOHEADER>() as DWORD,
				biWidth: width as LONG,
				biHeight: height as LONG,
				biPlanes: 1,
				biBitCount: 8*pixel_width as WORD,
				biCompression: BI_RGB,
				biSizeImage: 0,
				biXPelsPerMeter: 0,
				biYPelsPerMeter: 0,
				biClrUsed: 0,
				biClrImportant: 0,
			};

			// copy bits into Vec
			GetDIBits(h_dc, h_bmp, 0, width as DWORD,
				&mut data[0] as *mut u8 as *mut c_void,
				&mut bmi as *mut BITMAPINFOHEADER as *mut c_void,
				DIB_RGB_COLORS);

			DeleteObject(h_dc); // don't need handle anymore
			DeleteObject(h_bmp);


			Screenshot {
				data: data,
				height: height as usize,
				width: width as usize,
				row_len: width as usize*pixel_width,
				pixel_width: pixel_width,
			}
		}
	}
}
