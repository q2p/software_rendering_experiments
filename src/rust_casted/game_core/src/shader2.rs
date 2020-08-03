use crate::{SCREEN_WIDTH, SCREEN_HEIGHT, RGBA, SCREEN_SPACE};
use tiny_lib::{matrices::*, vector::*, util::*};
use crate::controls::*;
use core::f32::consts::PI;
use core::cmp::min;
use tiny_lib::util::*;
use tiny_lib::prng::RNG;
use tiny_lib::hasher::*;

pub struct Shader1();

impl Shader1 {

/// Returns [-0.5, 0.5]
fn noise(pos:Vec3) -> f32 {
	// Determine grid cell coordinates
	let x0 = pos.x.floor() as i32;
	let y0 = pos.y.floor() as i32;
	let z0 = pos.z.floor() as i32;

	let xt = pos.x - x0 as f32;
	let yt = pos.y - y0 as f32;
	let zt = pos.z - z0 as f32;
	
	let x1 = x0 + 1;
	let y1 = y0 + 1;
	let z1 = z0 + 1;

	#[inline(always)]
	fn pr(x:i32, y:i32, z:i32) -> f32 {
		let pos = (x as u64) << 32 ^ (y as u64) << 21 ^ (z as u64);
		// let v = RNG::new(a).get_f32() - 0.5;
		let v = (Hasher::hash_u64(0, pos) as f32) / (u32::MAX as f32) - 0.5;
		return v;
	}

	let _000 = pr(x0, y0, z0);
	let _001 = pr(x0, y0, z1);
	let _010 = pr(x0, y1, z0);
	let _011 = pr(x0, y1, z1);
	let _100 = pr(x1, y0, z0);
	let _101 = pr(x1, y0, z1);
	let _110 = pr(x1, y1, z0);
	let _111 = pr(x1, y1, z1);

	let _00t = f32::lerp(_000, _001, zt);
	let _01t = f32::lerp(_010, _011, zt);
	let _10t = f32::lerp(_100, _101, zt);
	let _11t = f32::lerp(_110, _111, zt);

	let _0tt = f32::lerp(_00t, _01t, yt);
	let _1tt = f32::lerp(_10t, _11t, yt);

	let _ttt = f32::lerp(_0tt, _1tt, xt);

	return _ttt;
}

	pub fn render(&mut self, image:&mut [RGBA], tick_number:u32, controls:&Controls) {
		let time = tick_number as f32 / 100f32;
		let rotation = rotation_matrix_z(time.sin()) * rotation_matrix_x(time * 0.5);

		for y in 0..SCREEN_HEIGHT {
			for x in 0..SCREEN_WIDTH {
				let mut uv = Vec2 {
					x: (                 x      as f32 + 0.5) / SCREEN_WIDTH  as f32 - 0.5, 
					y: ((SCREEN_HEIGHT - y - 1) as f32 + 0.5) / SCREEN_HEIGHT as f32 - 0.5
				} * 2.0;
						
				// let mut c = RGBA::zeroed();

				let light = Vec3::new(-1.0,1.0,1.0).normalize();

				let mut color = Vec3::ZERO;
		
							
				let len_sq = uv.x.sq() + uv.y.sq();
				
				
				let h = (1.0 - len_sq).sqrt();
		
		
				if h.is_finite() {
					let mut map = Vec3::new(uv.x, uv.y, h);
		
					let mut mask = light.dot(&map).max(0.0);
		
					mask = 1.0 - (1.0 - mask).sq();
		
					map = &map * &rotation;
					map = 0.5 * (map + Vec3::ONE);
		
					const zoom:f32 = 8.0;
		
					map += Vec3::new(0.0,0.0,time);
					let h =
										Self::noise(map * zoom      ) +
						0.500 * Self::noise(map * zoom * 2.0) +
						0.250 * Self::noise(map * zoom * 4.0) +
						0.125 * Self::noise(map * zoom * 8.0)
					;
		
					let h = f32::linearstep(0.05, 0.5, h);
		
					color = gradient(h, &[
						(0.00, Vec3::new(0.110, 0.112, 0.510)), // Deep water
						(0.13, Vec3::new(0.000, 0.573, 0.969)), // Water
						(0.16, Vec3::new(1.000, 0.996, 0.600)), // Sand
						(0.18, Vec3::new(0.000, 0.788, 0.427)), // Grass
						(0.50, Vec3::new(0.250, 0.581, 0.027)), // Grass Dark
						(0.70, Vec3::new(0.537, 0.357, 0.298)), // Mountain
						(1.00, Vec3::new(0.996, 0.996, 0.996)), // Snow
					]);
		
					color *= mask;
				}
		
				let n = (unsafe { tiny_lib::prng::GLOBAL_PRNG.get_f32() } - 0.5) * 0.1;
		
				color.x += n;
				color.y += n;
				color.z += n;

				let c = RGBA {
					r: (color.x * 255.0).round() as u8,
					g: (color.y * 255.0).round() as u8,
					b: (color.z * 255.0).round() as u8,
					a: 255
				};

				image[y as usize*SCREEN_WIDTH as usize + x as usize] = c;
			}
		}
	}
}

#[inline(always)]
pub fn gradient(t:f32, points:&[(f32, Vec3)]) -> Vec3 {
	debug_assert!(!points.is_empty());
	unsafe {
		if t < points.get_unchecked(0).0 {
			return points.get_unchecked(0).1;
		}
		for i in 1..points.len() {
			let p1 = points.get_unchecked(i  ).0;
			if t < p1 {
				let p0 = points.get_unchecked(i-1).0;
				let c0 = points.get_unchecked(i-1).1;
				let c1 = points.get_unchecked(i  ).1;
				let d = p1 - p0;
				let t = t - p0;
				let t = t / d;
				return Vec3::lerp(c0, c1, t)
			}
		}
		return points.get_unchecked(points.len() - 1).1;
	}
}