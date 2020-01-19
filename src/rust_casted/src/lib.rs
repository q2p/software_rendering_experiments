//#![feature(wasm_import_memory)]
//#![wasm_import_memory]

#![crate_type = "cdylib"]

// TODO: watch for update https://github.com/rust-lang/rust/issues/29596
#![feature(link_args)]
#![allow(unused_attributes)] // link_args actually is used
//#![link_args = "--import-memory"]
#![feature(const_fn)]
#![feature(const_if_match)]
#![feature(core_intrinsics)] // likely / unlikely branch predictions
#![feature(clamp)]
#![feature(tau_constant)]
#![feature(test)]

// #![no_std]
// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
// 	loop {}
// }

// ==== DISPlAY ====

mod profile;
mod timing;
mod controls;
mod trig;
mod vector;
mod sft_renderer;
//mod rcl_general;
//mod rcl;
//mod rcl_my_settings;
mod rcl_switch;
//mod demo1;

use timing::*;
use controls::Controls;
use sft_renderer::SoftwareRenderer;
//use demo1::Demo1;

const SCREEN_WIDTH:u16 = 512;
const SCREEN_HEIGHT:u16 = 512;

#[no_mangle]
pub unsafe extern fn w() -> u16 { SCREEN_WIDTH }
#[no_mangle]
pub unsafe extern fn h() -> u16 { SCREEN_HEIGHT }

struct State {
	timing:Timing,
	controls:Controls,
	software_renderer:SoftwareRenderer,
//	demo:Demo1,
}

static mut STATE:State = State {
	timing: Timing::new(),
	controls: Controls::new(SCREEN_WIDTH, SCREEN_HEIGHT),
	software_renderer: SoftwareRenderer::new(),
//	demo: Demo1::new(),
};

#[no_mangle]
static mut p:[u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4] = [255; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4];

impl State {
	#[inline(always)]
	fn init(&mut self) {
		self.software_renderer.init();
//		self.demo.main();
	}

	#[inline(always)]
	fn tick(
		&mut self,
		time_stamp:i32,
		locked_pointer:u8,
		abs_x:f32, abs_y:f32,
		delta_x:f32, delta_y:f32,
		scale:f32,
		mouse_down:u8, mouse_up:u8,
		image:&mut[u8],
	) {
		self.controls.input_loop(
			locked_pointer != 0,
			abs_x, abs_y,
			delta_x, delta_y,
			scale,
			mouse_down != 0,
			mouse_up != 0,

			false,
			false,
			false,
			false,
			false,

			SCREEN_WIDTH, SCREEN_HEIGHT
		);
		let time = self.timing.timing_loop(time_stamp);
		if let Some(time) = time {
			// self.demo.update(image, &time, &self.controls);
			self.software_renderer.render(image, time.tick_number(), &self.controls);
		}
	}
}

// ==== EXPORTS ====

#[no_mangle]
pub unsafe extern fn t(
	time_stamp:i32,
	locked_pointer:u8,
	abs_x:f32, abs_y:f32,
	delta_x:f32, delta_y:f32,
	scale:f32,
	mouse_down:u8, mouse_up:u8
) {
	STATE.tick(
		time_stamp,
		locked_pointer,
		abs_x, abs_y,
		delta_x, delta_y,
		scale,
		mouse_down, mouse_up,
		&mut p
	);
}

#[no_mangle]
pub unsafe extern fn i() {
	STATE.init();
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RGB {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RGBA {
	pub rgb: RGB,
	pub a: u8,
}
impl RGB {
	pub const fn rgb(r:u8, g:u8, b:u8) -> RGB {
		RGB { r, g, b }
	}
}
impl RGBA {
	pub const fn rgba(r:u8, g:u8, b:u8, a:u8) -> RGBA {
		RGBA { rgb:RGB::rgb(r, g, b), a }
	}
}