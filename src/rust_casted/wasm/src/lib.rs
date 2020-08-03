//#![feature(wasm_import_memory)]
//#![wasm_import_memory]

#![crate_type = "cdylib"]

// TODO: watch for update https://github.com/rust-lang/rust/issues/29596
#![feature(link_args)]
#![allow(unused_attributes)] // link_args actually is used
//#![link_args = "--import-memory"]
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

use game_core::*;

static mut STATE:State = State::new();

#[no_mangle]
pub const extern fn w() -> u16 { SCREEN_WIDTH }
#[no_mangle]
pub const extern fn h() -> u16 { SCREEN_HEIGHT }

#[no_mangle]
pub static mut p:[RGBA; SCREEN_SPACE as usize] = [RGBA::zeroed(); SCREEN_SPACE as usize];

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
		locked_pointer != 0,
		abs_x, abs_y,
		delta_x, delta_y,
		scale,
		mouse_down != 0, mouse_up != 0,
		&mut p
	);
}

#[no_mangle]
pub unsafe extern fn i() {
	STATE.init();
}