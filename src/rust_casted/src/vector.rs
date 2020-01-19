use core::ops::*;
use tiny_lib::util::QuickInverseSQRT;

pub trait Vector : Add + Sub + Mul<f32> + AddAssign + SubAssign + MulAssign<f32> + Copy + Clone {
	const ZERO:Self;
	fn dot_product(v1:&Self, v2:&Self) -> f32;
	fn len(&self) -> f32;
	fn inv_len(&self) -> f32;
}

macro_rules! impl_vec {
	(
    $($vec:ident; ( $($var:ident),+ ));+
  ) => {
  	$(
			#[derive(Copy, Clone)]
			pub struct $vec {
				$(pub $var: f32),+
			}
			impl $vec {
				pub const fn new($($var: f32),+) -> $vec {
					$vec {
						$($var),+
					}
				}
			}
			impl Vector for $vec {
				const ZERO:$vec = $vec{
					$($var: 0f32),+
				};
				fn dot_product(v1:&$vec, v2:&$vec) -> f32 {
					0f32 $(
						+ (v1.$var * v2.$var)
					)+
				}
				fn len(&self) -> f32 {
					(0f32 $(
						+ (self.$var * self.$var)
					)+).sqrt()
				}
				fn inv_len(&self) -> f32 {
					(0f32 $(
						+ (self.$var * self.$var)
					)+).quick_inverse_sqrt()
				}
			}
			impl Add for $vec {
				type Output = $vec;
				#[inline]
				fn add(self, rhs: $vec) -> $vec {
					$vec {
						$($var: self.$var + rhs.$var),+
					}
				}
			}
			impl Sub for $vec {
				type Output = $vec;
				#[inline]
				fn sub(self, rhs: $vec) -> $vec {
					$vec {
						$($var: self.$var - rhs.$var),+
					}
				}
			}
			impl Mul<f32> for $vec {
				type Output = $vec;
				#[inline]
				fn mul(self, rhs: f32) -> $vec {
					$vec {
						$($var: self.$var * rhs),+
					}
				}
			}
			impl AddAssign for $vec {
				#[inline]
				fn add_assign(&mut self, rhs: $vec) {
					$(self.$var += rhs.$var;)+
				}
			}
			impl SubAssign for $vec {
				#[inline]
				fn sub_assign(&mut self, rhs: $vec) {
					$(self.$var -= rhs.$var;)+
				}
			}
			impl MulAssign<f32> for $vec {
				#[inline]
				fn mul_assign(&mut self, rhs: f32) {
					$(self.$var *= rhs;)+
				}
			}
    )+
  }
}
impl_vec!(
	Vector1D; (x);
	Vector2D; (x,y);
	Vector3D; (x,y,z);
	VecUVW; (u,v,w);
	Vec3d; (x,y,z,w)
);

impl Vec3d {
	pub const fn xyz1(x:f32, y:f32, z:f32) -> Vec3d {
		Vec3d { x, y, z, w: 1f32 }
	}
}