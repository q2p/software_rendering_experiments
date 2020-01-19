pub struct Controls {
	is_pointer_locked: bool,
	m_rounded_x: u16,
	m_rounded_y: u16,
	m_prec_x: f32,
	m_prec_y: f32,
	pub lmb:Key,
	pub arrow_up:Key,
	pub arrow_down:Key,
	pub arrow_left:Key,
	pub arrow_right:Key,
	pub space:Key,

	pub a:Key,
	pub c:Key,
}

pub struct Key {
	is_pressed:bool,
}
impl Key {
	#[inline(always)]
	const fn new() -> Key { Key{ is_pressed: false } }
	#[inline(always)]
	fn update(&mut self, is_pressed:bool) { self.is_pressed = is_pressed; }
	#[inline(always)]
	pub fn is_pressed(&self) -> bool { self.is_pressed }
}

impl Controls {
	pub const fn new(screen_width:u16, screen_height:u16) -> Controls {
		Controls {
			is_pointer_locked: false,

			m_rounded_x: screen_width / 2,
			m_rounded_y: screen_height / 2,
			m_prec_x: screen_width as f32 / 2f32,
			m_prec_y: screen_height as f32 / 2f32,

			lmb:         Key::new(),
			arrow_up:    Key::new(),
			arrow_down:  Key::new(),
			arrow_left:  Key::new(),
			arrow_right: Key::new(),
			space:       Key::new(),
			a:           Key::new(),
			c:           Key::new(),
		}
	}

	pub fn input_loop(
		&mut self,
		locked_pointer:bool,
		abs_x:f32,
		abs_y:f32,
		delta_x:f32,
		delta_y:f32,
		scale:f32,
		mouse_down:bool,
		mouse_up:bool,

		vku: bool,
		vkd: bool,
		vkl: bool,
		vkr: bool,
		space:bool,

		screen_width:u16, screen_height:u16,
	) {
		if !self.is_pointer_locked || !locked_pointer {
			self.m_prec_x = abs_x / scale;
			self.m_prec_y = abs_y / scale;
		}
		self.is_pointer_locked = locked_pointer;
		if self.is_pointer_locked {
			self.m_prec_x += delta_x / scale;
			self.m_prec_y += delta_y / scale;
		}
		//m_prec_x = m_prec_x.clamp(0f32, SCREEN_WIDTH  as f32);
		//m_prec_y = m_prec_y.clamp(0f32, SCREEN_HEIGHT as f32);
		self.m_rounded_x = core::cmp::min((self.m_prec_x+0.5f32) as u16, screen_width  - 1);
		self.m_rounded_y = core::cmp::min((self.m_prec_y+0.5f32) as u16, screen_height - 1);

		self.lmb        .update(mouse_down);
		self.arrow_up   .update(vku);
		self.arrow_down .update(vkd);
		self.arrow_left .update(vkl);
		self.arrow_right.update(vkr);
		self.space      .update(space);
	}

	// Cursor
	#[inline(always)] pub const fn is_pointer_locked(&self) -> bool { self.is_pointer_locked }
	#[inline(always)] pub const fn pointer_x        (&self) -> u16 { self.m_rounded_x }
	#[inline(always)] pub const fn pointer_y        (&self) -> u16 { self.m_rounded_y }
	#[inline(always)] pub const fn pointer_precise_x(&self) -> f32 { self.m_prec_x    }
	#[inline(always)] pub const fn pointer_precise_y(&self) -> f32 { self.m_prec_y    }
}