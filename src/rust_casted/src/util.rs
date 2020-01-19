use core::num::NonZeroI8;
use core::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg};

pub fn Q_rsqrt(number:f32) -> f32 {
	const threehalfs:f32 = 1.5;

	let x2 = number * 0.5;
	let mut y = number;
	let mut i = y.to_bits(); // evil floating point bit level hacking
	i = 0x5f3759df - ( i >> 1 ); // what the fuck?
	y = f32::from_bits(i);
	y = y * ( threehalfs - ( x2 * y * y ) ); // 1st iteration
	//	y  = y * ( threehalfs - ( x2 * y * y ) ); // 2nd iteration, this can be removed

	return y;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct NonZeroSignum(NonZeroI8);

impl NonZeroSignum {
	// Unsafe, because the argument might be 0. But we are only using 1 and -1.
	// When const_fn will become stable, we can just const `.unwrap()` the result and not use unsafe.
	pub const POS:NonZeroSignum = NonZeroSignum(unsafe { NonZeroI8::new_unchecked( 1) });
	pub const NEG:NonZeroSignum = NonZeroSignum(unsafe { NonZeroI8::new_unchecked(-1) });

	#[inline(always)]
	pub fn is_positive(&self) -> bool {
		self.0.get() > 0
	}
	#[inline(always)]
	pub fn is_negative(&self) -> bool {
		self.0.get() < 0
	}
}
impl Neg for NonZeroSignum {
	type Output = NonZeroSignum;
	fn neg(self) -> NonZeroSignum {
		// Unsafe, because `self.0` might be 0. But it's guaranteed not to be 0, because we are using NonZero struct.
		NonZeroSignum(unsafe { NonZeroI8::new_unchecked(-self.0.get()) })
	}
}

// TODO: tests
macro_rules! impl_signum_signed {
	($($t:ty),*) => {
		$(
			impl Into<$t> for NonZeroSignum {
				#[inline(always)]
				fn into(self) -> $t { self.0.get() as $t }
			}
			impl Mul<$t> for NonZeroSignum {
				type Output = $t;
				#[inline(always)]
				fn mul(self, rhs: $t) -> $t { Into::<$t>::into(self) * rhs }
			}
			impl Mul<NonZeroSignum> for $t {
				type Output = $t;
				#[inline(always)]
				fn mul(self, rhs: NonZeroSignum) -> $t { self * Into::<$t>::into(rhs) }
			}
			impl MulAssign<NonZeroSignum> for $t {
				#[inline(always)]
				fn mul_assign(&mut self, rhs: NonZeroSignum) { *self *= Into::<$t>::into(rhs) }
			}
			impl Div<NonZeroSignum> for $t {
				type Output = $t;
				#[inline(always)]
				fn div(self, rhs: NonZeroSignum) -> $t { self * Into::<$t>::into(rhs) }
			}
			impl DivAssign<NonZeroSignum> for $t {
				#[inline(always)]
				fn div_assign(&mut self, rhs: NonZeroSignum) { *self *= Into::<$t>::into(rhs) }
			}
			impl Add<$t> for NonZeroSignum {
				type Output = $t;
				#[inline(always)]
				fn add(self, rhs: $t) -> $t { Into::<$t>::into(self) + rhs }
			}
			impl Add<NonZeroSignum> for $t {
				type Output = $t;
				#[inline(always)]
				fn add(self, rhs: NonZeroSignum) -> $t { self + Into::<$t>::into(rhs) }
			}
			impl Sub<NonZeroSignum> for $t {
				type Output = $t;
				#[inline(always)]
				fn sub(self, rhs: NonZeroSignum) -> $t { self - Into::<$t>::into(rhs) }
			}
			impl_assign!($t);
		)*
	}
}
macro_rules! impl_signum_unsigned {
	($real:ty, $temp:ty) => {
		impl Add<NonZeroSignum> for $real {
			type Output = $real;
			#[inline(always)]
			fn add(self, rhs: NonZeroSignum) -> $real { (self as $temp + Into::<$temp>::into(rhs)) as $real }
		}
		impl Add<$real> for NonZeroSignum {
			type Output = $real;
			#[inline(always)]
			fn add(self, rhs: $real) -> $real { rhs + self }
		}
		impl Sub<NonZeroSignum> for $real {
			type Output = $real;
			#[inline(always)]
			fn sub(self, rhs: NonZeroSignum) -> $real { (self as $temp - Into::<$temp>::into(rhs)) as $real }
		}
		impl_assign!($real);
	};
}
macro_rules! impl_assign {
  ($t:ty) => {
		impl AddAssign<NonZeroSignum> for $t {
			#[inline(always)]
			fn add_assign(&mut self, rhs: NonZeroSignum) { *self = *self + rhs }
		}
		impl SubAssign<NonZeroSignum> for $t {
			#[inline(always)]
			fn sub_assign(&mut self, rhs: NonZeroSignum) { *self = *self - rhs }
		}
  }
}
impl_signum_signed!(i8,i16,i32,i64);
impl_signum_unsigned!(u8 , i8 );
impl_signum_unsigned!(u16, i16);
impl_signum_unsigned!(u32, i32);
impl_signum_unsigned!(u64, i64);

pub trait Vector : Add + Sub + Mul<f32> + AddAssign + SubAssign + MulAssign<f32> + Copy + Clone {
	const ZERO:Self;
	fn dot_product(v1:&Self, v2:&Self) -> f32;
	fn len(&self) -> f32;
	fn rev_len(&self) -> f32;
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
				fn rev_len(&self) -> f32 {
					Q_rsqrt(
						(0f32 $(
							+ (self.$var * self.$var)
						)+)
					)
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
	VecUVW; (u,v,w);
	Vec3d; (x,y,z,w)
);

impl Vec3d {
	pub const fn xyz1(x:f32, y:f32, z:f32,) -> Vec3d {
		Vec3d { x, y, z, w: 1f32 }
	}
}