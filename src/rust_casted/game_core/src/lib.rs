// TODO: watch for update https://github.com/rust-lang/rust/issues/29596
// #![no_std] // TODO:
#![feature(const_fn)]
#![feature(const_if_match)]
#![feature(const_loop)]
#![feature(const_mut_refs)]
#![feature(const_extern_fn)]
#![feature(core_intrinsics)] // likely / unlikely branch predictions
#![feature(clamp)]
#![feature(tau_constant)]
#![feature(test)]

// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
// 	loop {}
// }

// ==== DISPlAY ====

// mod profile;
mod timing;
mod controls;
// mod trig;
// mod vector;
// mod sft_renderer;
// mod shader1;
// mod shader2;
mod minecraft4k;
//mod rcl;
//mod demo1;

use timing::*;
use controls::Controls;
// use sft_renderer::SoftwareRenderer;
// use shader1::Shader1;
// use shader2::Shader1;
use minecraft4k::Shader1;
//use demo1::Demo1;

pub const SCREEN_WIDTH:u16 = minecraft4k::w as u16 * 4;
pub const SCREEN_HEIGHT:u16 = minecraft4k::h as u16 * 4;
pub const SCREEN_SPACE:u32 = SCREEN_WIDTH as u32 * SCREEN_HEIGHT as u32;

pub struct State {
	timing:Timing,
	controls:Controls,
//	software_renderer:SoftwareRenderer,
//	demo:Demo1,
	shader:Shader1,
}

impl State {
	#[inline(always)]
	pub const fn new() -> State {
		State {
			timing: Timing::new(),
			controls: Controls::new(SCREEN_WIDTH, SCREEN_HEIGHT),
//			software_renderer: SoftwareRenderer::new(),
//			demo: Demo1::new(),
			shader:Shader1::new(),
		}
	}

	#[inline(always)]
	pub fn init(&mut self) {
//		self.software_renderer.init();
//		self.demo.main();
		self.shader.init();
	}

	#[inline(always)]
	pub fn tick(
		&mut self,
		time_stamp:i32,
		locked_pointer:bool,
		abs_x:f32, abs_y:f32,
		delta_x:f32, delta_y:f32,
		scale:f32,
		lmb:bool, rmb:bool,
		vk_u:bool, vk_d:bool, vk_l:bool, vk_r:bool, vk_space:bool,
		image:&mut [RGBA],
	) {
		self.controls.input_loop(
			locked_pointer,
			abs_x, abs_y,
			delta_x, delta_y,
			scale,

			lmb,
			rmb,

			vk_u,
			vk_d,
			vk_l,
			vk_r,
			vk_space,

			SCREEN_WIDTH, SCREEN_HEIGHT
		);
		let time = self.timing.timing_loop(time_stamp);
		if let Some(time) = time {
//			self.demo.update(image, &time, &self.controls);
//			self.software_renderer.render(image, time.tick_number(), &self.controls);
			self.shader.handle_event(&self.controls);
			self.shader.render(image, time.tick_number(), &self.controls);
		}
	}
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
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8,
}
impl RGB {
	#[inline]
	pub const fn rgb(r:u8, g:u8, b:u8) -> RGB {
		RGB { r, g, b }
	}
	#[inline(always)]
	pub const fn zeroed() -> RGB { RGB::rgb(0, 0, 0) }
}
impl RGBA {
	#[inline]
	pub const fn rgba(r:u8, g:u8, b:u8, a:u8) -> RGBA { RGBA { r, g, b, a } }
	#[inline(always)]
	pub const fn zeroed() -> RGBA { RGBA::rgba(0, 0, 0, 0) }
	#[inline]
	pub const fn to_rgb32(self) -> u32 {
		((self.r as u32) << 16) |
		((self.g as u32) <<  8) |
		 (self.b as u32)
	}

	pub const fn is_zero(&self) -> bool {
		u32::from_ne_bytes([self.r, self.g, self.b, self.a]) == 0
	}
}