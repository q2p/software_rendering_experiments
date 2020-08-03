use crate::RGBA;
use crate::controls::*;
use tiny_lib::trig::PI;

const TEXTURES_AMOUNT:usize = 16;

const LEVEL_SIZE:usize = 64;

pub struct Shader1 {
	M:[i32;32767],
	
	canvas:[RGBA;w*h],
	blocks:[u8;LEVEL_SIZE*LEVEL_SIZE*LEVEL_SIZE],
	textures:[[RGBA; 16*16*3];TEXTURES_AMOUNT],

	mouse_x:i32, // M[2]
	mouse_y:i32, // M[3]
	lmb:bool,
	rmb:bool,
	vk_space:bool,
	vk_u:bool,
	vk_d:bool,
	vk_l:bool,
	vk_r:bool,

	f1:f32,
	f2:f32,
	f3:f32,
	f4:f32,
	f5:f32,
	f6:f32,
	last_frame_ms:u64, // l
	i4:i32,
	i5:i32,
	rot_x:f32, // f7
	rot_y:f32, // f8
}

pub const MS_PER_FRAME:u8 = 10;

pub const w:usize = 214;
pub const h:usize = 120;
pub const w_half:usize = w/2;
pub const h_half:usize = h/2;

impl Shader1 {
	pub const fn new() -> Shader1 {
		Shader1 {
			mouse_x:0,
			mouse_y:0,
			lmb: false, // M[0]
			rmb: false, // M[1]
			vk_space: false,
			vk_u: false,
			vk_d: false,
			vk_l: false,
			vk_r: false,
			M: [0;32767],
			// int[] arrayOfInt1 = ((DataBufferInt)localBufferedImage.getRaster().getDataBuffer()).getData();
			canvas:[RGBA::zeroed();w*h], // 	TYPE_INT_RGB
			// int[] arrayOfInt2 = new int[262144];
			blocks:[0;LEVEL_SIZE*LEVEL_SIZE*LEVEL_SIZE],
			// int[] arrayOfInt3 = new int[12288];
			textures:[[RGBA::zeroed();16*16*3];TEXTURES_AMOUNT],
			f1:0.0,
			f2:0.0,
			f3:0.0,
			f4:0.0,
			f5:0.0,
			f6:0.0,
			last_frame_ms:0,
			i4:0,
			i5:0,
			rot_x:0.0,
			rot_y:0.0,
		}
	}

	// public void run() {
	pub fn init(&mut self) {
		// Random localRandom = new Random();
		let localRandom = unsafe { &mut tiny_lib::prng::GLOBAL_PRNG };
		// BufferedImage localBufferedImage = new BufferedImage(214, 120, 1);
		// localRandom.setSeed(18295169L);
		// NOOP
		// for (int i = 0; i < 262144; i++)
		for i in 0..262144 {
			// arrayOfInt2[i] = (i / 64 % 64 > 32 + localRandom.nextInt(8)) ? (localRandom.nextInt(8) + 1) : 0; 
			self.blocks[i as usize] = if i / 64 % 64 > 32 + (localRandom.get_u32() % 8) { localRandom.get_u32() as u8 % 8 + 1 } else { 0 };
		}
		// for (int j = 1; j < 16; j++) {
		for texture in 1..TEXTURES_AMOUNT {
			// int k = 255 - localRandom.nextInt(96);
			let mut k = 255 - (localRandom.get_u32() as i32 % 96);
			// for (int m = 0; m < 48; m++) {
			for m in 0..48 {
				// for (int n = 0; n < 16; n++) {
				for n in 0..16 {
					// int i1 = 9858122;
					let mut i1 = 9858122;
					// if (j == 4)
					if texture == 4 {
						// i1 = 8355711; 
						i1 = 8355711;
					}
					// if (j != 4 || localRandom.nextInt(3) == 0)
					if texture != 4 || localRandom.get_u32() % 3 == 0 {
						// k = 255 - localRandom.nextInt(96); 
						k = 255 - (localRandom.get_u32() % 96) as i32;
					}
					// if (j == 1 && m < (n * n * 3 + n * 81 >> 2 & 0x3) + 18) {
					if texture == 1 && m < (n * n * 3 * n * 81 >> 2 & 0x3) + 18 {
						// i1 = 6990400;
						i1 = 6990400;
					// } else if (j == 1 && m < (n * n * 3 + n * 81 >> 2 & 0x3) + 19) {
					} else if texture == 1 && m < (n * n * 3 + n * 81 >> 2 & 0x3) + 19 {
						// k = k * 2 / 3;
						k = k * 2 / 3;
					// } 
					}
					// if (j == 7) {
					if texture == 7 {
						// i1 = 6771249;
						i1 = 6771249;
						// if (n > 0 && n < 15 && ((m > 0 && m < 15) || (m > 32 && m < 47))) {
						if n > 0 && n < 15 && ((m > 0 && m < 15) || (m > 32 && m < 47)) {
							// i1 = 12359778;
							i1 = 12359778;
							// int i6 = n - 7;
							let mut i6 = n - 7;
							// int i7 = (m & 0xF) - 7;
							let mut i7 = (m & 0xF) - 7;
							// if (i6 < 0)
							if i6 < 0 {
								// i6 = 1 - i6; 
								i6 = 1 - i6; 
							}
							// if (i7 < 0)
							if i7 < 0 {
								// i7 = 1 - i7; 
								i7 = 1 - i7; 
							}
							// if (i7 > i6)
							if i7 > i6 {
								// i6 = i7; 
								i6 = i7; 
							}
							// k = 196 - localRandom.nextInt(32) + i6 % 3 * 32;
							k = 196 - (localRandom.get_u32() as i32 % 32) + i6 % 3 * 32;
						// } else if (localRandom.nextInt(2) == 0) {
						} else if localRandom.get_u32() as i32 % 2 == 0 {
							// k = k * (150 - (n & 0x1) * 100) / 100;
							k = k * (150 - (n & 0x1) * 100) / 100;
						// } 
						}
					// } 
					}
					// if (j == 5) {
					if texture == 5 {
						// i1 = 11876885;
						i1 = 11876885;
						// if ((n + m / 4 * 4) % 8 == 0 || m % 4 == 0)
						if (n + m / 4 * 4) % 8 == 0 || m % 4 == 0 {
							// i1 = 12365733; 
							i1 = 12365733; 
						}
					// } 
					}
					// int i2 = k;
					let mut i2 = k;
					// if (m >= 32)
					if m >= 32 {
						// i2 /= 2; 
						i2 /= 2; 
					}
					// if (j == 8) {
					if texture == 8 {
						// i1 = 5298487;
						i1 = 5298487;
						// if (localRandom.nextInt(2) == 0) {
						if localRandom.get_u32() % 2 == 0 {
							// i1 = 0;
							i1 = 0;
							// i2 = 255;
							i2 = 255;
						// } 
						}
					// } 
					}
					// int i3 = (i1 >> 16 & 0xFF) * i2 / 255 << 16 | (i1 >> 8 & 0xFF) * i2 / 255 << 8 | (i1 & 0xFF) * i2 / 255;
					let i3 = RGBA {
						r: ((i1 >> 16 & 0xFF) * i2 / 255) as u8,
						g: ((i1 >>  8 & 0xFF) * i2 / 255) as u8,
						b: ((i1       & 0xFF) * i2 / 255) as u8,
						a: 0,
					};
					// arrayOfInt3[n + m * 16 + j * 256 * 3] = i3;
					self.textures[texture][(n + m * 16) as usize] = i3;
				// } 
				}
			// } 
			}
		// } 
		}
		// float f1 = 96.5F;
		self.f1 = 96.5;
		// float f2 = 65.0F;
		self.f2 = 65.0;
		// float f3 = 96.5F;
		self.f3 = 96.5;
		// float f4 = 0.0F;
		self.f4 = 0.0;
		// float f5 = 0.0F;
		self.f5 = 0.0;
		// float f6 = 0.0F;
		self.f6 = 0.0;
		// int i4 = -1;
		self.i4 = -1;
		// int i5 = 0;
		self.i5 = 0;
	}
	// while (true) {
	pub fn main_loop(&mut self, image:&mut [RGBA], tick_number:u32) {
		let time_ms = tick_number as u64 * 10;
		let rot_x_cos = self.rot_x.sin(); // f9
		let rot_x_sin = self.rot_x.cos(); // f10
		let rot_y_sin = self.rot_y.sin(); // f11
		let rot_y_cos = self.rot_y.cos(); // f12
		while time_ms - self.last_frame_ms > MS_PER_FRAME as u64 {
			self.last_frame_ms += MS_PER_FRAME as u64;

			if self.mouse_x > 0 {
				let rot_speed_x = (self.mouse_x - 428) as f32 / w as f32 * 2.0; // f16
				let rot_speed_y = (self.mouse_y - 240) as f32 / h as f32 * 2.0; // f17
				let rot_speed_hypot = (rot_speed_x * rot_speed_x + rot_speed_y * rot_speed_y).sqrt() - 1.2f32; // f15
				
				if rot_speed_hypot > 0.0 {
					self.rot_x += rot_speed_x * rot_speed_hypot / 400.0;
					self.rot_y -= rot_speed_y * rot_speed_hypot / 400.0;
					self.rot_y = self.rot_y.clamp(-PI/2.0, PI/2.0);
				}
			}
			let mut speed_forward = 0.0;
			let mut speed_right   = 0.0;
			const WALKING_SPEED:f32 = 0.02;
			// f14 += (this.M[119] - this.M[115]) * 0.02F;
			// f13 += (this.M[100] - this.M[97]) * 0.02F;
			if self.vk_u { speed_forward += WALKING_SPEED };
			if self.vk_d { speed_forward -= WALKING_SPEED };
			if self.vk_l { speed_right   -= WALKING_SPEED };
			if self.vk_r { speed_right   += WALKING_SPEED };

			// f4 *= 0.5F;
			// f5 *= 0.99F;
			// f6 *= 0.5F;
			// f4 += f9 * f14 + f10 * f13;
			// f6 += f10 * f14 - f9 * f13;
			// f5 += 0.003F;
			self.f4 *= 0.5;
			self.f5 = self.f5 * 0.99 + 0.003;
			self.f6 *= 0.5;
			self.f4 += rot_x_cos * speed_forward + rot_x_sin * speed_right;
			self.f6 += rot_x_sin * speed_forward - rot_x_cos * speed_right;

			// int m;
			// label208: for (m = 0; m < 3; m++) {
		  'label208: for m in 0..3 {
				// float f16 = f1 + f4 * ((m + 0) % 3 / 2);
				// float f17 = f2 + f5 * ((m + 1) % 3 / 2);
				// float f19 = f3 + f6 * ((m + 2) % 3 / 2);
				let f16 = self.f1 + self.f4 * ((m + 0) % 3 / 2) as f32;
				let f17 = self.f2 + self.f5 * ((m + 1) % 3 / 2) as f32;
				let f19 = self.f3 + self.f6 * ((m + 2) % 3 / 2) as f32;
				// for (int i12 = 0; i12 < 12; i12++) {
				for i12 in 0..12 {
					//   int i13 = (int)(f16 + (i12 >> 0 & 0x1) * 0.6F - 0.3F) - 64;
					let i13 = (f16 + (i12 >> 0 & 0x1) as f32 * 0.6 - 0.3) as i32 - 64;
					//   int i14 = (int)(f17 + ((i12 >> 2) - 1) * 0.8F + 0.65F) - 64;
					let i14 = (f17 + ((i12 >> 2) - 1) as f32 * 0.8 + 0.65) as i32  - 64;
					//   int i15 = (int)(f19 + (i12 >> 1 & 0x1) * 0.6F - 0.3F) - 64;
					let i15 = (f19 + (i12 >> 1 & 0x1) as f32 * 0.6 - 0.3) as i32  - 64;
					//   if (i13 < 0 || i14 < 0 || i15 < 0 || i13 >= 64 || i14 >= 64 || i15 >= 64 || arrayOfInt2[i13 + i14 * 64 + i15 * 4096] > 0) {
					if i13 < 0 || i14 < 0 || i15 < 0 || i13 >= LEVEL_SIZE as i32 || i14 >= LEVEL_SIZE as i32 || i15 >= LEVEL_SIZE as i32 || self.blocks[(i13 + i14 * LEVEL_SIZE as i32 + i15 * LEVEL_SIZE as i32 * LEVEL_SIZE as i32) as usize] > 0 {
						// if (m != 1)
						if m != 1 {
							// break label208; 
							break 'label208; 
						}
						// if (this.M[32] > 0 && f5 > 0.0F) {
						if self.vk_space && self.f5 > 0.0 {
							// this.M[32] = 0;
							self.vk_space = false;
							// f5 = -0.1F;
							self.f5 = -0.1;
							// break label208;
							break 'label208;
						// } 
						} 
						// f5 = 0.0F;
						self.f5 = 0.0;
						// break label208;
						break 'label208;
					// } 
					} 
				// } 
				}
				// f1 = f16;
				self.f1 = f16;
				// f2 = f17;
				self.f2 = f17;
				// f3 = f19;
				self.f3 = f19;
			// } 
			} 
		// } 
		} 
		// int i6 = 0;
		let mut i6 = 0;
		// int i7 = 0;
		let mut i7 = 0;
		// if (this.M[1] > 0 && i4 > 0) {
			// arrayOfInt2[i4] = 0;
			// this.M[1] = 0;
		// } 
		// if (this.M[0] > 0 && i4 > 0) {
			// arrayOfInt2[i4 + i5] = 1;
			// this.M[0] = 0;
		// } 
		if self.rmb && self.i4 > 0 {
		  self.rmb = false;
		  self.blocks[self.i4 as usize] = 0;
		} 
		if self.lmb && self.i4 > 0 {
		  self.lmb = false;
		  self.blocks[(self.i4 + self.i5) as usize] = 1;
		} 

		// for (int k = 0; k < 12; k++) {
		for k in 0..12 {
			// int m = (int)(f1 + (k >> 0 & 0x1) * 0.6F - 0.3F) - 64;
			let m = (self.f1 + (k >> 0 & 0x1) as f32 * 0.6 - 0.3) as i32 - 64;
			// int i10 = (int)(f2 + ((k >> 2) - 1) * 0.8F + 0.65F) - 64;
			let i10 = (self.f2 + ((k >> 2) - 1) as f32 * 0.8 + 0.65) as i32 - 64;
			// int i11 = (int)(f3 + (k >> 1 & 0x1) * 0.6F - 0.3F) - 64;
			let i11 = (self.f3 + (k >> 1 & 0x1) as f32 * 0.6 - 0.3) as i32 - 64;
			// if (m >= 0 && i10 >= 0 && i11 >= 0 && m < 64 && i10 < 64 && i11 < 64)
			if m >= 0 && i10 >= 0 && i11 >= 0 && m < 64 && i10 < 64 && i11 < 64 {
				// arrayOfInt2[m + i10 * 64 + i11 * 4096] = 0; 
				self.blocks[(m + i10 * 64 + i11 * 4096) as usize] = 0; 
			}	
		// } 
		} 


		const c_90:f32 = 90.0;
		const c_1:f32 = 1.0; // f21
		const PSEUDO_FOV:f32 = PI;
		const VIEW_DISTANCE:f32 = 20.0;

		// float i8 = -1.0F;
		let mut _i8:f32 = -1.0;
		for cx in 0..w as i32 { // i9
		  for cy in 0..h as i32 { // i11
				// float f20 = (i11 - 60) / 90.0F;
				// float f18 = (i9 - 107) / 90.0F;
				let ray_y = (cy - h_half as i32) as f32 / w as f32 * PSEUDO_FOV;
				let ray_x = (cx - w_half as i32) as f32 / w as f32 * PSEUDO_FOV;
				// float f22 = f21 * f12 + f20 * f11;
				// float f23 = f20 * f12 - f21 * f11;
				// float f24 = f18 * f10 + f22 * f9;
				// float f25 = f22 * f10 - f18 * f9;
				let f22 = ray_y * rot_y_sin + c_1 * rot_y_cos;
				let f23 = ray_y * rot_y_cos - c_1 * rot_y_sin;
				let f24 = f22 * rot_x_cos + ray_x * rot_x_sin;
				let f25 = f22 * rot_x_sin - ray_x * rot_x_cos;

		    let mut color = RGBA::zeroed(); // i16
		    let mut brightness = 255; // i17
		    let mut distance = VIEW_DISTANCE; // d
				// float f26 = 5.0F;
		    let mut f26 = 5.0;

				// for (i18 = 0; i18 < 3; i18++) {
				for i18 in 0..3 {
					// float f27 = f24;
					let mut f27 = f24;
					// if (i18 == 1)
					if i18 == 1 {
						// f27 = f23; 
						f27 = f23; 
					}
					// if (i18 == 2)
					if i18 == 2 {
					// f27 = f25; 
						f27 = f25; 
					}

					// float f28 = 1.0F / ((f27 < 0.0F) ? -f27 : f27);
					let f28 = 1.0 / if f27 < 0.0 { -f27 } else { f27 };
					// float f29 = f24 * f28;
					let f29 = f24 * f28;
					// float f30 = f23 * f28;
					let f30 = f23 * f28;
					// float f31 = f25 * f28;
					let f31 = f25 * f28;
					// float f32 = f1 - (int)f1;
					let mut _f32 = self.f1 - (self.f1 as i32) as f32;

					// if (i18 == 1)
					if i18 == 1 {
						// f32 = f2 - (int)f2; 
						_f32 = self.f2 - (self.f2 as i32) as f32;
					}
					// if (i18 == 2)
					if i18 == 2 {
						// f32 = f3 - (int)f3; 
						_f32 = self.f3 - (self.f3 as i32) as f32;
					}
					// if (f27 > 0.0F)
					if f27 > 0.0 {
						// f32 = 1.0F - f32; 
						_f32 = 1.0 - _f32;
					}
					// float f33 = f28 * f32;
		      let mut f33 = f28 * _f32;
					// float f34 = f1 + f29 * f32;
		      let mut f34 = self.f1 + f29 * _f32;
					// float f35 = f2 + f30 * f32;
		      let mut f35 = self.f2 + f30 * _f32;
					// float f36 = f3 + f31 * f32;
		      let mut f36 = self.f3 + f31 * _f32;

					// if (f27 < 0.0F) {
		      if f27 < 0.0 {
						// if (i18 == 0)
		      	if i18 == 0 {
							// f34--; 
							f34 -= 1.0; 
						}
						// if (i18 == 1)
		        if i18 == 1 {
							// f35--; 
							f35 -= 1.0; 
						}
						// if (i18 == 2)
						if i18 == 2 {
							// f36--; 
							f36 -= 1.0;
						}
					// } 
		      } 

					// while (f33 < d) {
					while f33 < distance {
						// int i21 = (int)f34 - 64;
						let i21 = f34 as i32 - 64;
						// int i22 = (int)f35 - 64;
						let i22 = f35 as i32 - 64;
						// int i23 = (int)f36 - 64;
						let i23 = f36 as i32 - 64;

						// if (i21 < 0 || i22 < 0 || i23 < 0 || i21 >= 64 || i22 >= 64 || i23 >= 64)
						//   break; 

		        if i21 < 0 || i22 < 0 || i23 < 0 || i21 >= 64 || i22 >= 64 || i23 >= 64 {
							break; 
						}

						// int i24 = i21 + i22 * 64 + i23 * 4096;
						let i24 = i21 + i22 * 64 + i23 * 4096;
						// int i25 = arrayOfInt2[i24];
						let block = self.blocks[i24 as usize]; // i25
						// if (i25 > 0) {
						if block != 0 {
							// i6 = (int)((f34 + f36) * 16.0F) & 0xF;
		          i6 = ((f34 + f36) * 16.0) as i32 & 0xF;
							// i7 = ((int)(f35 * 16.0F) & 0xF) + 16;
		          i7 = ((f35 * 16.0) as i32 & 0xF) + 16;
							// if (i18 == 1) {
		          if i18 == 1 {
								// i6 = (int)(f34 * 16.0F) & 0xF;
		            i6 = (f34 * 16.0) as i32 & 0xF;
								// i7 = (int)(f36 * 16.0F) & 0xF;
		            i7 = (f36 * 16.0) as i32 & 0xF;
								// if (f30 < 0.0F)
		            if f30 < 0.0 {
									// i7 += 32; 
		              i7 += 32; 
								}
							// } 
		          } 

		          let mut possible_color = RGBA { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF }; // i26
							// if (i24 != i4 || (i6 > 0 && i7 % 16 > 0 && i6 < 15 && i7 % 16 < 15))
							//   i26 = arrayOfInt3[i6 + i7 * 16 + i25 * 256 * 3]; 
		          if i24 != self.i4 || (i6 > 0 && i7 % 16 > 0 && i6 < 15 && i7 % 16 < 15) {
								possible_color = self.textures[block as usize][(i6 + i7 * 16) as usize]; 
							}
							// if (f33 < f26 && i9 == this.M[2] / 4 && i11 == this.M[3] / 4) {
		          if f33 < f26 && cx == self.mouse_x / 4 && cy == self.mouse_y / 4 {
								// i8 = i24;
								_i8 = i24 as f32;
								// i5 = 1;
		            self.i5 = 1;
								// if (f27 > 0.0F)
								if f27 > 0.0 {
									// i5 = -1; 
									self.i5 = -1; 
								}
								// i5 <<= 6 * i18;
		            self.i5 <<= 6 * i18;
								// f26 = f33;
		          	f26 = f33;
							// } 
		          } 

							// if (i26 > 0) {
							if !possible_color.is_zero() {
								// i16 = i26;
		          	color = possible_color;
								// i17 = 255 - (int)(f33 / 20.0F * 255.0F);
								// i17 = i17 * (255 - (i18 + 2) % 3 * 50) / 255;
		          	brightness = 255 - (f33 / VIEW_DISTANCE * 255.0) as i32;
		          	brightness = brightness * (255 - (i18 + 2) % 3 * 50) / 255;
								// d = f33;
		          	distance = f33;
							// } 
		          } 
						// } 
						} 

						// f34 += f29;
						f34 += f29;
						// f35 += f30;
						f35 += f30;
						// f36 += f31;
						f36 += f31;
						// f33 += f28;
						f33 += f28;
					// } 
					} 
				// } 
				} 
		
				let pixel = &mut self.canvas[(cy * w as i32 + cx) as usize];
				pixel.r = (color.r as i32 * brightness / 255) as u8; // i18
				pixel.g = (color.g as i32 * brightness / 255) as u8; // i19
				pixel.b = (color.b as i32 * brightness / 255) as u8; // i20
			}
		}
		
		// i4 = (int)i8;
		self.i4 = _i8 as i32;

		// Thread.sleep(2L);
		// if (!isActive())
		//   return; 
		// NOOP

		// getGraphics().drawImage(localBufferedImage, 0, 0, 856, 480, null);
		for y in 0..h {
			for x in 0..w {
				unsafe { 
					let l = *self.canvas.get_unchecked( y * w + x);
					for ry in 0..4 {
						for rx in 0..4 {
							let ty = y * 4 + ry;
							let tx = x * 4 + rx;
							let tw = w * 4;
							*image.get_unchecked_mut((ty * tw) + tx) = l;
						}
					}
				}
			}
		}

		// }
	}

	// public boolean handleEvent(Event paramEvent) {
	pub fn handle_event(&mut self, controls:&Controls) {
		self.lmb      = controls.lmb        .is_pressed();
		self.rmb      = controls.rmb        .is_pressed();
		self.vk_space = controls.space      .is_pressed();
		self.vk_u     = controls.arrow_up   .is_pressed();
		self.vk_d     = controls.arrow_down .is_pressed();
		self.vk_l     = controls.arrow_left .is_pressed();
		self.vk_r     = controls.arrow_right.is_pressed();

		let px = controls.pointer_precise_x() as i32;
		let py = controls.pointer_precise_y() as i32;

		if px < 0 || py < 0 || px >= w as i32 * 4 || py >= h as i32 * 4 {
			self.mouse_x = 0;
		} else {
			self.mouse_x = px as i32;
			self.mouse_y = py as i32;
		}

		// int i = 0;
		// switch (paramEvent.id) {
		//       case 401:
		//         i = 1;
		//       case 402:
		//         this.M[paramEvent.key] = i;
		//         break;
		//       case 501:
		//         i = 1;
		//         this.M[2] = paramEvent.x;
		//         this.M[3] = paramEvent.y;
		//       case 502:
		//         if ((paramEvent.modifiers & 0x4) > 0) {
		//           this.M[1] = i;
		//           break;
		//         } 
		//         this.M[0] = i;
		//         break;
		//       case 503:
		//       case 506:
		//         this.M[2] = paramEvent.x;
		//         this.M[3] = paramEvent.y;
		//         break;
		//       case 505:
		//         this.M[2] = 0;
		//         break;
		// } 
		// return true;
	// }
	}

	pub fn render(&mut self, image:&mut [RGBA], tick_number:u32, controls:&Controls) {
		self.handle_event(controls);
		self.main_loop(image, tick_number);
	}
}