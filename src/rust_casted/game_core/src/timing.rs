use core::num::NonZeroU32;

const TARGET_FPS:u8 = 100;

// const MAX_TICKS_PER_FRAME:u8 = 4;
// const MAX_TICKS_PER_FRAME:u8 = 8;
const MAX_TICKS_PER_FRAME:u8 = 16; // TODO:
const TARGET_SLEEP:u8 = ((1000f32 / TARGET_FPS as f32)+0.9f32) as u8;

const MAX_DELTA_OVERFLOW_DISTANCE:i32 = -100;

#[derive(Copy, Clone)]
pub struct Time {
	tick_number:u32,
	ticks:NonZeroU32,
}
impl Time {
	#[inline(always)]
	pub fn tick_number(&self) -> u32 { self.tick_number }
	#[inline(always)]
	pub fn ticks(&self) -> NonZeroU32 { self.ticks }
}
pub struct Timing {
	tick_number:u32,
	last_tick_time_stamp:i32,
}
impl Timing {
	pub const fn new() -> Timing {
		Timing {
			tick_number: 0,
			last_tick_time_stamp: core::i32::MIN,
		}
	}
	pub fn timing_loop(&mut self, time_stamp:i32) -> Option<Time> {
		let mut delta;

		if self.last_tick_time_stamp == core::i32::MIN {
			self.last_tick_time_stamp = time_stamp;
			delta = TARGET_SLEEP as i32;
		} else {
			delta = time_stamp - self.last_tick_time_stamp;
			if delta < MAX_DELTA_OVERFLOW_DISTANCE {
				delta = (MAX_TICKS_PER_FRAME * TARGET_SLEEP) as i32;
			}
		}

		if delta <= 0 {
			return None;
		}

		let mut ticks = delta as u32 / TARGET_SLEEP as u32;

		if ticks > MAX_TICKS_PER_FRAME as u32 {
			ticks = MAX_TICKS_PER_FRAME as u32;
			self.last_tick_time_stamp = time_stamp;
		} else {
			self.last_tick_time_stamp += ticks as i32 * TARGET_SLEEP as i32;
		}

		let ticks = match NonZeroU32::new(ticks) {
			None => return None, // ticks = 0
			Some(v) => v,
		};

		self.tick_number += ticks.get();

		return Some(Time {
			ticks,
			tick_number: self.tick_number,
		});
	}
}