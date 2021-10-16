//! Capture a bitmap image of a display. The resulting screenshot is stored in
//! the `Screenshot` type, which varies per platform.
//!
//! # Platform-specific details
//!
//! Despite OS X's CoreGraphics documentation, the bitmap returned has its
//! origin at the top left corner. It uses ARGB pixels.
//!
//! The Windows GDI bitmap has its coordinate origin at the bottom left. We
//! attempt to undo this by reordering the rows. Windows also uses ARGB pixels.


#![allow(unused_assignments)]

extern crate libc;

use std::mem::size_of;

// pub use ffi::get_screenshot;
pub use ffi::Screener;


#[derive(Clone, Copy)]
pub struct Pixel {
	pub a: u8,
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

/// An image buffer containing the screenshot.
/// Pixels are stored as [ARGB](https://en.wikipedia.org/wiki/ARGB).
pub struct Screenshot<'a> {
	img: *mut xlib::XImage,
	data: &'a mut [u8],
	height: usize,
	width: usize,
	row_len: usize, // Might be superfluous
	pixel_width: usize,
}

impl<'a> Screenshot<'a> {
	/// Height of image in pixels.
	#[inline]
	pub fn height(&self) -> usize { self.height }

	/// Width of image in pixels.
	#[inline]
	pub fn width(&self) -> usize { self.width }

	/// Number of bytes in one row of bitmap.
	#[inline]
	pub fn row_len(&self) -> usize { self.row_len }

	/// Width of pixel in bytes.
	#[inline]
	pub fn pixel_width(&self) -> usize { self.pixel_width }

	/// Raw bitmap.
	#[inline]
	pub unsafe fn raw_data(&self) -> *const u8 {
		(*self.img).data as *const u8
	}

	/// Raw bitmap.
	#[inline]
	pub unsafe fn raw_data_mut(&mut self) -> *mut u8 {
		(*self.img).data as *mut u8
	}

	/// Number of bytes in bitmap
	#[inline]
	pub fn raw_len(&self) -> usize {
		self.data.len() * size_of::<u8>()
	}

	/// Gets pixel at (row, col)
	pub fn get_pixel(&self, row: usize, col: usize) -> Pixel {
		let idx = (row*self.row_len() + col*self.pixel_width()) as isize;
		unsafe {
			let data = &self.data[0] as *const u8;
			if idx as usize > self.raw_len() { panic!("Bounds overflow"); }

			Pixel {
				b: *data.offset(idx),
				g: *data.offset(idx+1),
				r: *data.offset(idx+2),
				a: *data.offset(idx+3),
			}
		}
	}

	pub fn data(&self) -> &[u8] {
		self.data
	}
}

impl<'a> AsRef<[u8]> for Screenshot<'a> {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		self.data
	}
}

pub type ScreenResult<'a> = Result<Screenshot<'a>, &'a str>;

#[cfg(target_os = "linux")]
mod ffi {
	extern crate xlib;

	use ::{Screenshot, ScreenResult};
	use std::{ptr::null_mut, slice};
	use std::mem;
	use libc::{c_int, c_uint, c_void};
	use self::xlib::{XOpenDisplay, XCloseDisplay, XScreenOfDisplay, XRootWindowOfScreen,
		XDestroyWindow, XWindowAttributes, XGetWindowAttributes, XImage, XGetImage, XAllPlanes, ZPixmap};

	impl<'a> Drop for Screenshot<'a> {
		fn drop(&mut self) {
			unsafe {
				let destroy_image: extern fn(*mut XImage) -> c_int = mem::transmute((*self.img).f.destroy_image);
				destroy_image(self.img);
			}
		}
	}


	pub struct Screener {
		display: *mut c_void,
		root: u64,
		window_attrs: XWindowAttributes,
	}
	
	impl Screener {
		pub unsafe fn new(screen: u32) -> Screener {
			let display = XOpenDisplay(null_mut());
			let screen = XScreenOfDisplay(display, screen as c_int);
			let root = XRootWindowOfScreen(screen);
			let mut attr: XWindowAttributes = mem::MaybeUninit::uninit().assume_init();
			XGetWindowAttributes(display, root, &mut attr);
			Screener {
				display,
				root,
				window_attrs: attr
			}
		}

		pub fn get_screenshot(&mut self) -> ScreenResult {
			unsafe {
				let img_ptr = XGetImage(
					self.display,
					self.root, 0, 0,
					self.window_attrs.width as c_uint,
					self.window_attrs.height as c_uint,
					XAllPlanes(), 
					ZPixmap);

				let img = &mut *img_ptr;

				// This is the function which XDestroyImage macro calls.
				// servo/rust-xlib doesn't handle function pointers correctly.
				// We have to transmute the variable.
				let destroy_image: extern fn(*mut XImage) -> c_int = mem::transmute(img.f.destroy_image);
				let height = img.height as usize;
				let width = img.width as usize;
				let row_len = img.bytes_per_line as usize;
				let pixel_bits = img.bits_per_pixel as usize;
				if pixel_bits % 8 != 0 {
					destroy_image(img);
					return Err("Pixels aren't integral bytes.");
				}
				let pixel_width = pixel_bits / 8;
	
				// Create a data slice for image
				let size = (width * height * pixel_width) as isize;
				// Data has Bgra8 format
				let data = slice::from_raw_parts_mut(img.data as *mut u8, size as usize);
				
				Ok(Screenshot {
					img,
					data,
					height,
					width,
					row_len,
					pixel_width,
				})
			}
		}
	}

	impl Drop for Screener {
		fn drop(&mut self) {
			unsafe {
				XDestroyWindow(self.display, self.root);
				XCloseDisplay(self.display);
			}
		}
	}

	// pub fn get_screenshot(screen: u32) -> ScreenResult<'static> {
	// 	unsafe {
	// 		let display = XOpenDisplay(null_mut());
	// 		let screen = XScreenOfDisplay(display, screen as c_int);
	// 		let root = XRootWindowOfScreen(screen);

	// 		let mut attr: XWindowAttributes = mem::MaybeUninit::uninit().assume_init();
	// 		XGetWindowAttributes(display, root, &mut attr);

	// 		let img = &mut *XGetImage(display, root, 0, 0, attr.width as c_uint, attr.height as c_uint,
	// 			XAllPlanes(), ZPixmap);
	// 		XDestroyWindow(display, root);
	// 		XCloseDisplay(display);
	// 		// This is the function which XDestroyImage macro calls.
	// 		// servo/rust-xlib doesn't handle function pointers correctly.
	// 		// We have to transmute the variable.
	// 		let destroy_image: extern fn(*mut XImage) -> c_int = mem::transmute(img.f.destroy_image);
	// 		let height = img.height as usize;
	// 		let width = img.width as usize;
	// 		let row_len = img.bytes_per_line as usize;
	// 		let pixel_bits = img.bits_per_pixel as usize;
	// 		if pixel_bits % 8 != 0 {
	// 			destroy_image(&mut *img);
	// 			return Err("Pixels aren't integral bytes.");
	// 		}
	// 		let pixel_width = pixel_bits / 8;

	// 		// Create a Vec for image
	// 		let size = (width * height * pixel_width) as isize;
	// 		let mut data = slice::from_raw_parts(img.data as *const u8, size as usize);
	// 		// let mut data = Vec::new();
	// 		// let data_ptr = img.data as *const u8;
			
	// 		// for i in (0..size).step_by(4) {
	// 		// 	let p = data_ptr.offset(i);
	// 		// 	data.extend([*p.offset(2), *p.offset(1), *p, *p.offset(3)].iter());
	// 		// }
			
	// 		destroy_image(&mut *img);

	// 		// Fix Alpha channel when xlib cannot retrieve info correctly
	// 		let has_alpha = data.iter().enumerate().any(|(n, x)| n % 4 == 3 && *x != 0);
	// 		if !has_alpha {
	// 			let mut n = 0;
	// 			for channel in data.iter_mut() {
	// 				if n % 4 == 3 { *channel = 255; }
	// 				n += 1;
	// 			}
	// 		}

	// 		Ok(Screenshot {
	// 			data: data,
	// 			height: height,
	// 			width: width,
	// 			row_len: row_len,
	// 			pixel_width: pixel_width,
	// 		})
	// 	}
	// }
}

#[cfg(target_os = "macos")]
mod ffi {
	#![allow(non_upper_case_globals, dead_code)]

	use std::slice;
	use libc;
	use ::Screenshot;
	use ::ScreenResult;

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

	/// Get a screenshot of the requested display.
	pub fn get_screenshot(screen: usize) -> ScreenResult {
		unsafe {
			// Get number of displays
			let mut count: CGDisplayCount = 0;
			let mut err = CGDisplayNoErr;
			err = CGGetActiveDisplayList(0, 0 as *mut CGDirectDisplayID, &mut count);
			if err != CGDisplayNoErr {
				return Err("Error getting number of displays.");
			}

			// Get list of displays
			let mut disps: Vec<CGDisplayCount> = Vec::with_capacity(count as usize);
			disps.set_len(count as usize);
			err = CGGetActiveDisplayList(disps.len() as libc::uint32_t,
				&mut disps[0] as *mut CGDirectDisplayID,
				&mut count);
			if err != CGDisplayNoErr {
				return Err("Error getting list of displays.");
			}

			// Get screenshot of requested display
			let disp_id = disps[screen];
			let cg_img = CGDisplayCreateImage(disp_id);

			// Get info about image
			let width = CGImageGetWidth(cg_img) as usize;
			let height = CGImageGetHeight(cg_img) as usize;
			let row_len = CGImageGetBytesPerRow(cg_img) as usize;
			let pixel_bits = CGImageGetBitsPerPixel(cg_img) as usize;
			if pixel_bits % 8 != 0 {
				return Err("Pixels aren't integral bytes.");
			}

			// Copy image into a Vec buffer
			let cf_data = CGDataProviderCopyData(CGImageGetDataProvider(cg_img));
			let raw_len = CFDataGetLength(cf_data) as usize;

			let res = if width*height*pixel_bits != raw_len*8 {
				Err("Image size is inconsistent with W*H*D.")
			} else {
				let data = slice::from_raw_parts(CFDataGetBytePtr(cf_data), raw_len).to_vec();
				Ok(Screenshot {
					data: data,
					height: height,
					width: width,
					row_len: row_len,
					pixel_width: pixel_bits/8
				})
			};

			// Release native objects
			CGImageRelease(cg_img);
			CFRelease(cf_data as *const libc::c_void);

			return res;
		}
	}
}

#[cfg(target_os = "windows")]
mod ffi {
	#![allow(non_snake_case, dead_code)]

	use libc::{c_int, c_uint, c_long, c_void};
	use std::mem::size_of;

	use ::Screenshot;
	use ::ScreenResult;

	type PVOID = *mut c_void;
	type LPVOID = *mut c_void;
	type WORD = u16; // c_uint;
	type DWORD = u32; // c_ulong;
	type BOOL = c_int;
	type BYTE = u8;
	type UINT = c_uint;
	type LONG = c_long;
	type LPARAM = c_long;

	#[repr(C)]
	struct RECT {
		left: LONG,
		top: LONG,
		right: LONG, // immediately outside rect
		bottom: LONG, // immediately outside rect
	}
	type LPCRECT = *const RECT;
	type LPRECT = *mut RECT;

	type HANDLE = PVOID;
	type HMONITOR = HANDLE;
	type HWND = HANDLE;
	type HDC = HANDLE;
	#[repr(C)]
	struct MONITORINFO {
		cbSize: DWORD,
		rcMonitor: RECT,
		rcWork: RECT,
		dwFlags: DWORD,
	}
	type LPMONITORINFO = *mut MONITORINFO;
	type MONITORENUMPROC = fn(HMONITOR, HDC, LPRECT, LPARAM) -> BOOL;

	type HBITMAP = HANDLE;
	type HGDIOBJ = HANDLE;
	type LPBITMAPINFO = PVOID; // Hack

	const NULL: *mut c_void = 0usize as *mut c_void;
	const HGDI_ERROR: *mut c_void = -1isize as *mut c_void;
	const SM_CXSCREEN: c_int = 0;
	const SM_CYSCREEN: c_int = 1;

	/// TODO verify value
	const SRCCOPY: u32 = 0x00CC0020;
	const CAPTUREBLT: u32 = 0x40000000;
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

	#[repr(C)]
	struct RGBQUAD {
		rgbBlue: BYTE,
		rgbGreen: BYTE,
		rgbRed: BYTE,
		rgbReserved: BYTE,
	}

	/// WARNING variable sized struct
	#[repr(C)]
	struct BITMAPINFO {
		bmiHeader: BITMAPINFOHEADER,
		bmiColors: [RGBQUAD; 1],
	}

	#[link(name = "user32")]
	extern "system" {
		fn GetSystemMetrics(m: c_int) -> c_int;
		fn EnumDisplayMonitors(hdc: HDC, lprcClip: LPCRECT, lpfnEnum: MONITORENUMPROC, dwData: LPARAM) -> BOOL;
		fn GetMonitorInfo(hMonitor: HMONITOR, lpmi: LPMONITORINFO) -> BOOL;
		fn GetDesktopWindow() -> HWND;
		fn GetDC(hWnd: HWND) -> HDC;
	}

	#[link(name = "gdi32")]
	extern "system" {
		fn CreateCompatibleDC(hdc: HDC) -> HDC;
		fn CreateCompatibleBitmap(hdc: HDC, nWidth: c_int, nHeight: c_int) -> HBITMAP;
		fn SelectObject(hdc: HDC, hgdiobj: HGDIOBJ) -> HGDIOBJ;
		fn BitBlt(hdcDest: HDC, nXDest: c_int, nYDest: c_int, nWidth: c_int, nHeight: c_int,
                  hdcSrc: HDC, nXSrc: c_int, nYSrc: c_int, dwRop: DWORD) -> BOOL;
		fn GetDIBits(hdc: HDC, hbmp: HBITMAP, uStartScan: UINT, cScanLines: UINT,
					 lpvBits: LPVOID, lpbi: LPBITMAPINFO, uUsage: UINT) -> c_int;

		fn DeleteObject(hObject: HGDIOBJ) -> BOOL;
		fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
		fn DeleteDC(hdc: HDC) -> BOOL;
	}

	/// Reorder rows in bitmap, last to first.
	/// TODO rewrite functionally
	fn flip_rows(data: Vec<u8>, height: usize, row_len: usize) -> Vec<u8> {
		let mut new_data = Vec::with_capacity(data.len());
		// unsafe {new_data.set_len(data.len())};

		data.iter().step_by(4).for_each(|x| {
			let p = x as *const u8;
			unsafe {
				new_data.extend([*p.offset(2), *p.offset(1), *p, *p.offset(3)].iter());
			}
		});

		// for row_i in 0..height {
		// 	for byte_i in (0..row_len).step_by(4) {
		// 		let old_idx = (height-row_i-1)*row_len + byte_i;
		// 		let new_idx = row_i*row_len + byte_i;
		// 		// println!("{}", old_idx);
		// 		// println!("{}", new_idx);
		// 		// println!("{}", data[old_idx]);
		// 		let p = data[old_idx] as *const u8;
		// 		unsafe {
		// 			// println!("{}", *p);
		// 			// new_data[new_idx] = *p.offset(2);
		// 			// new_data[new_idx+1] = *p.offset(1);
		// 			// new_data[new_idx+2] = *p;
		// 			// new_data[new_idx+3] = *p.offset(3);
		// 		}

		// 		// new_data[new_idx] = data[old_idx];
		// 	}
		// }
		new_data
	}

	/// TODO Support multiple screens
	/// This may never happen, given the horrific quality of Win32 APIs
	pub fn get_screenshot(_screen: usize) -> ScreenResult {
		unsafe {
			// Enumerate monitors, getting a handle and DC for requested monitor.
			// loljk, because doing that on Windows is worse than death
			let h_wnd_screen = GetDesktopWindow();
			let h_dc_screen = GetDC(h_wnd_screen);
			let width = GetSystemMetrics(SM_CXSCREEN);
			let height = GetSystemMetrics(SM_CYSCREEN);

			// Create a Windows Bitmap, and copy the bits into it
			let h_dc = CreateCompatibleDC(h_dc_screen);
			if h_dc == NULL { return Err("Can't get a Windows display.");}

			let h_bmp = CreateCompatibleBitmap(h_dc_screen, width, height);
			if h_bmp == NULL { return Err("Can't create a Windows buffer");}

			let res = SelectObject(h_dc, h_bmp);
			if res == NULL || res == HGDI_ERROR {
				return Err("Can't select Windows buffer.");
			}

			let res = BitBlt(h_dc, 0, 0, width, height, h_dc_screen, 0, 0, SRCCOPY|CAPTUREBLT);
			if res == 0 { return Err("Failed to copy screen to Windows buffer");}

			// Get image info
			let pixel_width: usize = 4; // FIXME
			let mut bmi = BITMAPINFO {
				bmiHeader: BITMAPINFOHEADER {
					biSize: size_of::<BITMAPINFOHEADER>() as DWORD,
					biWidth: width as LONG,
					biHeight: height as LONG,
					biPlanes: 1,
					biBitCount: 32 as WORD,
					biCompression: BI_RGB,
					biSizeImage: 0,
					biXPelsPerMeter: 0,
					biYPelsPerMeter: 0,
					biClrUsed: 0,
					biClrImportant: 0,
				},
				bmiColors: [RGBQUAD {
					rgbRed: 0,
					rgbGreen: 0,
					rgbBlue: 0,
					rgbReserved: 0
				}],
			};

			// Create a Vec for image
			let size: usize = (width*height) as usize * pixel_width;
			let mut data: Vec<u8> = Vec::with_capacity(size);
			data.set_len(size);

			// copy bits into Vec
			GetDIBits(h_dc, h_bmp, 0, height as DWORD,
				&mut data[0] as *mut u8 as *mut c_void,
				&mut bmi as *mut BITMAPINFO as *mut c_void,
				DIB_RGB_COLORS);

			// Release native image buffers
			ReleaseDC(h_wnd_screen, h_dc_screen); // don't need screen anymore
			DeleteDC(h_dc);
			DeleteObject(h_bmp);

			// data.reverse();

			let data = flip_rows(data, height as usize, width as usize*pixel_width);

			Ok(Screenshot {
				data,
				height: height as usize,
				width: width as usize,
				row_len: width as usize*pixel_width,
				pixel_width,
			})
		}
	}
}

// #[test]
// fn test_get_screenshot() {
// 	let s: Screenshot = get_screenshot(0).unwrap();
// 	println!("width: {}\n height: {}\npixel width: {}\n bytes: {}",
// 		s.width(), s.height(), s.pixel_width(), s.raw_len());
// }
