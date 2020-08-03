use minifb::{Key, Window, WindowOptions, Scale, ScaleMode, MouseMode, MouseButton, KeyRepeat};
use std::time::Instant;

use game_core::*;

static mut STATE:DesktopState = DesktopState::new();

struct DesktopState {
	game:State,
	rgba:[RGBA; SCREEN_SPACE as usize],
	buffer:[u32; SCREEN_SPACE as usize],
}

impl DesktopState {
	pub const fn new() -> DesktopState {
		DesktopState {
			game: State::new(),
			rgba: [RGBA::zeroed(); SCREEN_SPACE as usize],
			buffer: [0; SCREEN_SPACE as usize],
		}
	}
	pub fn init(&mut self) {
		self.game.init();
	}
}

fn main() {
	let state = unsafe { &mut STATE };
	state.init();

	let mut window = Box::new(
		Window::new(
			"Rust Window", SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize,
			WindowOptions {
				borderless: false,
				title: true,
				resize: false,
				scale: Scale::X1,
				scale_mode: ScaleMode::UpperLeft,
			}
		).unwrap()
	);

	window.limit_update_rate(None);

	let earliest_time = Instant::now();

	let mut mouse_x = 0.0;
	let mut mouse_y = 0.0;
	let mut dx;
	let mut dy;

	loop {
		if !window.is_open() || window.is_key_down(Key::Escape) {
			break;
		}

		let before_tick = Instant::now();
		let time_stamp = before_tick.duration_since(earliest_time).as_millis() as u32;
		let time_stamp = (core::i32::MIN as i64 + time_stamp as i64) as i32;

		let vku = window.is_key_down(Key::W);
		let vkd = window.is_key_down(Key::S);
		let vkl = window.is_key_down(Key::A);
		let vkr = window.is_key_down(Key::D);
		let vk_space = window.is_key_pressed(Key::Space, KeyRepeat::No);

		let lmb = window.get_mouse_down(MouseButton::Left);
		let rmb = window.get_mouse_down(MouseButton::Right);

		if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
			dx = mx - mouse_x;
			dy = my - mouse_y;
			mouse_x = mx;
			mouse_y = my;
		} else {
			dx = 0.0;
			dy = 0.0;
		}

		state.game.tick(
			time_stamp, false,
			mouse_x, mouse_y, dx, dy,
			1f32,
			lmb, rmb,
			vku, vkd, vkl, vkr, vk_space,
			&mut state.rgba
		);

		let before_rgb = Instant::now();

		convert_to_rgb(&state.rgba, &mut state.buffer);

		let before_window = Instant::now();

		window.update_with_buffer(&state.buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize).unwrap();

		let now = Instant::now();

		println!("internal: {:>4}ms, rgb: {:>4}ms, window: {:>4}ms, total: {:>4}ms",
			before_rgb   .duration_since(before_tick  ).as_millis(),
			before_window.duration_since(before_rgb   ).as_millis(),
			now          .duration_since(before_window).as_millis(),
			now          .duration_since(before_tick  ).as_millis(),
		);
	}
}

#[inline(always)]
fn convert_to_rgb(from:&[RGBA], to:&mut [u32]) {
	for i in 0..SCREEN_SPACE {
		debug_assert!(from.get(i as usize).is_some());
		debug_assert!(to  .get(i as usize).is_some());

		unsafe {
			*to.get_unchecked_mut(i as usize) = from.get_unchecked(i as usize).to_rgb32();
		}
	}
}