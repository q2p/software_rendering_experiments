/*
  raycastlib (RCL) - Small C header-only raycasting library for embedded and
  low performance computers, such as Arduino. Only uses integer math and stdint
  standard library.

  Check the defines below to fine-tune accuracy vs performance! Don't forget
  to compile with optimizations.

  Before including the library define RCL_PIXEL_FUNCTION to the name of the
  function (with RCL_PixelFunction signature) that will render your pixels!

  - All public (and most private) library identifiers start with RCL_.
  - Game field's bottom left corner is at [0,0].
  - X axis goes right in the ground plane.
  - Y axis goes up in the ground plane.
  - Height means the Z (vertical) coordinate.
  - Each game square is RCL_UNITS_PER_SQUARE * RCL_UNITS_PER_SQUARE points.
  - Angles are in RCL_Units, 0 means pointing right (x+) and positively rotates
    clockwise. A full angle has RCL_UNITS_PER_SQUARE RCL_Units.
  - Most things are normalized with RCL_UNITS_PER_SQUARE (sin, cos, vector
    unit length, texture coordinates etc.).
  - Screen coordinates are normal: [0,0] = top left, x goes right, y goes down.

  The library is meant to be used in not so huge programs that use single
  translation unit and so includes both declarations and implementation at once.
  If you for some reason use multiple translation units (which include the
  library), you'll have to handle this yourself (e.g. create a wrapper, manually
  split the library into .c and .h etc.).

  author: Miloslav "drummyfish" Ciz
  license: CC0 1.0
  version: 0.901
*/

// #include <stdint.h>

const HITS_ARRAY_LIMIT:u8 = 16;

const CAMERA_RESOLUTION_X:u16 = 320; // 20
const CAMERA_RESOLUTION_Y:u16 = 200; // 15
const _RCL_camResYLimit:u16 = CAMERA_RESOLUTION_Y - 1;
const halfResY:u16 = CAMERA_RESOLUTION_Y / 2;

use crate::profile;
use std::fmt::{Display, Formatter, Error};
use tiny_lib::util::NonZeroSignum;
use crate::rcl_switch::*;
use crate::rcl_general::RCL_General;

const RCL_RAYCAST_TINY:bool = false; // Turns on super efficient version of this library. Only use if neccesarry, looks ugly. Also not done yet.

// Smallest spatial unit, there is RCL_UNITS_PER_SQUARE units in a square's length. This effectively serves the purpose of a fixed-point arithmetic.
pub(crate) type RCL_Unit = i32; // i16 if not RCL_RAYCAST_TINY
type RCL_Unit_unsigned = u32; // u16 if not RCL_RAYCAST_TINY
const RCL_INFINITY:RCL_Unit = if RCL_RAYCAST_TINY { 2000000000 } else { 30000 };
pub(crate) const RCL_UNITS_PER_SQUARE:RCL_Unit = if RCL_RAYCAST_TINY { 1024 } else { 32 }; // Number of RCL_Units in a side of a spatial square.
const RCL_USE_DIST_APPROX:u8 =
	if RCL_RAYCAST_TINY {
		0 // What distance approximation to use:
		// 0: none (compute full Euclidean distance)
		// 1: accurate approximation
		// 2: octagonal approximation (LQ)
	} else {
		2
	}
;

pub const RCL_COMPUTE_WALL_TEXCOORDS:bool = true;

const RCL_COMPUTE_FLOOR_TEXCOORDS:bool = false;

pub const RCL_FLOOR_TEXCOORDS_HEIGHT:RCL_Unit = 0;
// If RCL_COMPUTE_FLOOR_TEXCOORDS == 1, this says for what height level the texture coords will be computed for (for simplicity/performance only one level is allowed).

const RCL_USE_COS_LUT:u8 = 0; // type of look up table for cos function: 0: none (compute) 1: 64 items 2: 128 items

const RCL_RECTILINEAR:bool = true; // Whether to use rectilinear perspective (normally used), or curvilinear perspective (fish eye).

const RCL_TEXTURE_VERTICAL_STRETCH:u8 = 1; // Whether textures should be stretched to wall height (possibly slightly slower if on).

const RCL_COMPUTE_FLOOR_DEPTH:bool = true; // Whether depth should be computed for floor pixels - turns this off if not needed.

const  RCL_COMPUTE_CEILING_DEPTH:bool = true; // As RCL_COMPUTE_FLOOR_DEPTH but for ceiling.

const RCL_ROLL_TEXTURE_COORDS:bool = true; // Says whether rolling doors should also roll the texture coordinates along (mostly desired for doors).

const RCL_VERTICAL_FOV:RCL_Unit = RCL_UNITS_PER_SQUARE / 2;
const RCL_HORIZONTAL_FOV:RCL_Unit = RCL_UNITS_PER_SQUARE / 4;
const RCL_HORIZONTAL_FOV_HALF:RCL_Unit = RCL_HORIZONTAL_FOV / 2;

const RCL_CAMERA_COLL_RADIUS:RCL_Unit = RCL_UNITS_PER_SQUARE / 4;
const RCL_CAMERA_COLL_HEIGHT_BELOW:RCL_Unit = RCL_UNITS_PER_SQUARE;
const RCL_CAMERA_COLL_HEIGHT_ABOVE:RCL_Unit = RCL_UNITS_PER_SQUARE / 3;
const RCL_CAMERA_COLL_STEP_HEIGHT:RCL_Unit = RCL_UNITS_PER_SQUARE / 2;

// This says scaling of fixed poit vertical texture coord computation. This should be power of two! Higher number can look more accurate but may cause overflow. */
const RCL_TEXTURE_INTERPOLATION_SCALE:RCL_Unit = 1024;

// What depth the horizon has (the floor depth is only approximated with the help of this constant).
const RCL_HORIZON_DEPTH:RCL_Unit = 11 * RCL_UNITS_PER_SQUARE;

const RCL_VERTICAL_DEPTH_MULTIPLY:RCL_Unit = 2; // Defines a multiplier of height difference when approximating floor/ceil depth.

/// To prevent zero divisions.
// TODO: NonZeroU8 and so on
fn RCL_nonZero(v:RCL_Unit) -> RCL_Unit {
	if v == 0 {
		1
	} else {
		v
	}
}
fn RCL_zeroClamp(x:RCL_Unit) -> RCL_Unit {
	if x >= 0 {
		x
	} else {
		0
	}
}
fn RCL_likely(cond:bool) -> bool {
	unsafe { core::intrinsics::likely(cond) }
}
fn RCL_unlikely(cond:bool) -> bool {
	unsafe { core::intrinsics::unlikely(cond) }
}

/// Position in 2D space.
#[derive(Copy, Clone)]
pub struct RCL_Vector2D {
  pub x:RCL_Unit,
  pub y:RCL_Unit,
}
impl RCL_Vector2D {
	pub const ZERO:RCL_Vector2D = RCL_Vector2D { x: 0, y: 0 };

	fn len(self) -> RCL_Unit {
		unsafe { profile::RCL_len.call(); }

		return RCL_Vector2D::dist(RCL_Vector2D::ZERO, self);
	}

	/// Normalizes given vector to have RCL_UNITS_PER_SQUARE length.
	fn normalize(self) -> RCL_Vector2D {
		unsafe { profile::RCL_normalize.call(); }

		let l = RCL_nonZero(self.len());

		return RCL_Vector2D {
			x: (self.x * RCL_UNITS_PER_SQUARE) / l,
			y: (self.y * RCL_UNITS_PER_SQUARE) / l,
		};
	}

	/// Computes a cos of an angle between two vectors.
	fn angleCos(mut v1:RCL_Vector2D, mut v2:RCL_Vector2D) -> RCL_Unit {
		unsafe { profile::RCL_vectorsAngleCos.call(); }

		v1 = v1.normalize();
		v2 = v2.normalize();

		return (v1.x * v2.x + v1.y * v2.y) / RCL_UNITS_PER_SQUARE;
	}

	fn dist(p1:RCL_Vector2D, p2:RCL_Vector2D) -> RCL_Unit {
		unsafe { profile::RCL_dist.call(); }

		let mut dx:RCL_Unit = p2.x - p1.x;
		let mut dy:RCL_Unit = p2.y - p1.y;

		if RCL_USE_DIST_APPROX == 2 {
			// octagonal approximation

			dx = RCL_absVal(dx);
			dy = RCL_absVal(dy);

			return if dy > dx {
				dx / 2 + dy
			} else {
				dy / 2 + dx
			};
		} else if RCL_USE_DIST_APPROX == 1 {
			// more accurate approximation

			// dx = ((dx < 0) * 2 - 1) * dx;
			// dy = ((dy < 0) * 2 - 1) * dy;
			// dx = if dx < 0 { 1 } else { -1 } * dx;
			// dy = if dy < 0 { 1 } else { -1 } * dy;
			if dx > 0 { dx *= -1; }
			if dy > 0 { dy *= -1; }

			let a;
			let b;
			if dx < dy {
				a = dy;
				b = dx;
			} else {
				a = dx;
				b = dy;
			}

			let mut result = a + (44 * b) / 102;

			if a < (b << 4) {
				result -= (5 * a) / 128;
			}

			return result;
		} else {
			dx = dx * dx;
			dy = dy * dy;

			return RCL_sqrtInt(dx + dy) as RCL_Unit;
		}
	}
}
impl Display for RCL_Vector2D {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		write!(f, "[{},{}]", self.x, self.y)
	}
}

#[derive(Copy, Clone)]
struct RCL_Ray {
  start:RCL_Vector2D,
  direction:RCL_Vector2D,
}
impl Display for RCL_Ray {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		write!(f, "ray: start: {}  dir: {}", self.start, self.direction)
	}
}

#[derive(Copy, Clone)]
pub struct RCL_HitResult {
	/// Distance to the hit position, or -1 if no collision happened. If RCL_RECTILINEAR != 0, then the distance is perpendicular to the projection plane (fish eye correction), otherwise it is the straight distance to the ray start position.
  distance:RCL_Unit,
	/// Direction of hit. The convention for angle units is explained above.
  pub direction:u8,
	/// Normalized (0 to RCL_UNITS_PER_SQUARE - 1) texture coordinate (horizontal).
  textureCoord:RCL_Unit,
	/// Collided square coordinates.
  square:RCL_Vector2D,
	/// Exact collision position in RCL_Units.
  position:RCL_Vector2D,
	/// Value returned by array function (most often this will be the floor height).
  arrayValue:RCL_Unit,
	/// Integer identifying type of square (number returned by type function, e.g. texture index).
	pub type_:RCL_Unit,
	/// Holds value of door roll.
  doorRoll:RCL_Unit,
}

impl RCL_HitResult {
	fn zeroed() -> RCL_HitResult {
		RCL_HitResult {
			distance: 0,
			direction: 0,
			textureCoord: 0,
			square: RCL_Vector2D::ZERO,
			position: RCL_Vector2D::ZERO,
			arrayValue: 0,
			type_: 0,
			doorRoll: 0
		}
	}

	/// Fills a RCL_HitResult struct with info for a hit at infinity.
	#[inline]
	fn _RCL_makeInfiniteHit(&mut self, ray:&RCL_Ray) {
		self.distance = RCL_UNITS_PER_SQUARE * RCL_UNITS_PER_SQUARE;
		// ^ horizon is at infinity, but we can't use too big infinity (RCL_INFINITY) because it would overflow in the following mult.
		self.position.x = (ray.direction.x * self.distance) / RCL_UNITS_PER_SQUARE;
		self.position.y = (ray.direction.y * self.distance) / RCL_UNITS_PER_SQUARE;

		self.direction = 0;
		self.textureCoord = 0;
		self.arrayValue = 0;
		self.doorRoll = 0;
		self.type_ = 0;
	}
}
impl Display for RCL_HitResult {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		writeln!(f, "hit:")?;
		writeln!(f, "  square: {}", self.square)?;
		writeln!(f, "  pos: {}", self.position)?;
		writeln!(f, "  dist: {}", self.distance)?;
		writeln!(f, "  dir: {}", self.direction)?;
		writeln!(f, "  texcoord: {}", self.textureCoord)
	}
}

#[derive(Clone)]
pub struct RCL_Camera {
  pub position:RCL_Vector2D,
  pub direction:RCL_Unit,
  pub resolution:RCL_Vector2D,
	// from -camera.resolution.y to +camera.resolution.y
  pub shear:i16, /// Shear offset in pixels (0 => no shear), can simulate looking up/down.
	pub height:RCL_Unit,
}

impl RCL_Camera {
	pub(crate) fn init(&mut self) {
		self.position.x = 0;
		self.position.y = 0;
		self.direction = 0;
//		self.resolution.x = 20;
//		self.resolution.y = 15;
		self.shear = 0;
		self.height = RCL_UNITS_PER_SQUARE;
	}
}

impl Display for RCL_Camera {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		writeln!(f, "camera:")?;
		writeln!(f, "  position: {}", self.position)?;
		writeln!(f, "  height: {}", self.height)?;
		writeln!(f, "  direction: {}", self.direction)?;
		writeln!(f, "  shear: {}", self.shear)
		// writeln!(f, "  resolution: {} x {}", self.resolution.x, self.resolution.y);
	}
}

/// Holds an information about a single rendered pixel (for a pixel function that works as a fragment shader).
pub struct RCL_PixelInfo {
	/// On-screen position.
  pub position:RCL_Vector2D,
	/// Whether the pixel is a wall or a floor/ceiling.
	pub isWall:bool,
	/// Whether the pixel is floor or ceiling.
	pub isFloor:bool,
	/// If the pixel belongs to horizon segment.
	pub isHorizon:bool,
	/// Corrected depth.
	pub depth:RCL_Unit,
	/// Only for wall pixels, says its height.
	pub wallHeight:RCL_Unit,
	/// World height (mostly for floor).
	pub height:RCL_Unit,
	/// Corresponding ray hit.
	pub hit:RCL_HitResult,
	/// Normalized (0 to RCL_UNITS_PER_SQUARE - 1) texture coordinates.
	pub texCoords:RCL_Vector2D,
}

impl RCL_PixelInfo {
	fn zeroed() -> RCL_PixelInfo {
		RCL_PixelInfo {
			position: RCL_Vector2D::ZERO,
			isWall: false,
			isFloor: false,
			isHorizon: false,
			depth: 0,
			wallHeight: 0,
			height: 0,
			hit: RCL_HitResult::zeroed(),
			texCoords: RCL_Vector2D::ZERO,
		}
	}
}

impl Display for RCL_PixelInfo {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		writeln!(f, "pixel:")?;
		writeln!(f, "  position: {}", self.position)?;
		writeln!(f, "  texCoord: {}", self.texCoords)?;
		writeln!(f, "  depth: {}", self.depth)?;
		writeln!(f, "  height: {}", self.height)?;
		writeln!(f, "  wall: {}", self.isWall)?;
		writeln!(f, "  hit: {}", self.hit)
	}
}

#[derive(Copy, Clone)]
pub struct RCL_RayConstraints {
  pub maxHits:u16,
  pub maxSteps:u16,
}

impl RCL_RayConstraints {
	pub fn init(&mut self) {
		self.maxHits = 1;
		self.maxSteps = 20;
	}
}

/**
  Function used to retrieve some information about cells of the rendered scene.
  It should return a characteristic of given square as an integer (e.g. square
  height, texture index, ...) - between squares that return different numbers
  there is considered to be a collision.

  This function should be as fast as possible as it will typically be called
  very often.
*/
pub(crate) type RCL_ArrayFunction = fn(x:i16, y:i16) -> RCL_Unit;
/*
  TODO: maybe array functions should be replaced by defines of funtion names
  like with pixelFunc? Could be more efficient than function pointers.
*/

/**
  Function that renders a single pixel at the display. It is handed an info
  about the pixel it should draw.

  This function should be as fast as possible as it will typically be called
  very often.
*/
pub type RCL_PixelFunction = fn(general:&mut RCL_General, info:&RCL_PixelInfo);
type RCL_ColumnFunction = fn(&mut RCL_Renderer_Global, general:&mut RCL_General, hits:&[RCL_HitResult], x:u16, ray:RCL_Ray);

//=============================================================================
// privates

fn _RCL_UNUSED<T>(what:T) {}

const CLT_SIZE:usize =
	if RCL_USE_COS_LUT == 1 {
		if RCL_RAYCAST_TINY {
			16
		} else {
			64
		}
	} else if RCL_USE_COS_LUT == 2 {
		128
	} else {
		0
	};
const cosLUT:&'static [RCL_Unit] =
	if RCL_USE_COS_LUT == 1 {
		if RCL_RAYCAST_TINY {
			&[ 16,14,11,6,0,-6,-11,-14,-15,-14,-11,-6,0,6,11,14 ]
		} else {
			&[
				 1024, 1019, 1004, 979, 946, 903, 851, 791, 724, 649, 568, 482,  391,  297, 199, 100, 0,-100,
				- 199,- 297,- 391,-482,-568,-649,-724,-791,-851,-903,-946,-979,-1004,-1019,
				-1023,-1019,-1004,-979,-946,-903,-851,-791,-724,-649,-568,-482,- 391,- 297,
				- 199,- 100,    0, 100, 199, 297, 391, 482, 568, 649, 724, 791,  851,  903, 946, 979,1004,1019
			]
		}
	} else if RCL_USE_COS_LUT == 2 {
		&[
			1024,1022,1019,1012,1004,993,979,964,946,925,903,878,851,822,791,758,724,
			687,649,609,568,526,482,437,391,344,297,248,199,150,100,50,0,-50,-100,-150,
			-199,-248,-297,-344,-391,-437,-482,-526,-568,-609,-649,-687,-724,-758,-791,
			-822,-851,-878,-903,-925,-946,-964,-979,-993,-1004,-1012,-1019,-1022,-1023,
			-1022,-1019,-1012,-1004,-993,-979,-964,-946,-925,-903,-878,-851,-822,-791,
			-758,-724,-687,-649,-609,-568,-526,-482,-437,-391,-344,-297,-248,-199,-150,
			-100,-50,0,50,100,150,199,248,297,344,391,437,482,526,568,609,649,687,724,
			758,791,822,851,878,903,925,946,964,979,993,1004,1012,1019,1022
		]
	} else {
		&[]
	};

pub fn RCL_clamp(value:RCL_Unit, valueMin:RCL_Unit, valueMax:RCL_Unit) -> RCL_Unit {
	unsafe { profile::RCL_clamp.call(); }

	debug_assert!(valueMin <= valueMax);

	if value >= valueMin {
		if value <= valueMax {
			value
		} else {
			valueMax
		}
	} else {
		valueMin
	}
}

#[inline]
pub fn RCL_absVal(value:RCL_Unit) -> RCL_Unit {
	unsafe { profile::RCL_absVal.call(); }
	// return value * (((value >= 0) << 1) - 1);
	// TODO:
	return value.abs()
}

/// Like mod, but behaves differently for negative values.
#[inline]
pub fn RCL_wrap(value:RCL_Unit, mod_:RCL_Unit) -> RCL_Unit {
	unsafe { profile::RCL_wrap.call(); }
	let cmp:RCL_Unit = if value < 0 { 1 } else { 0 };
	return cmp * mod_ + (value % mod_) - cmp;
}

/// Performs division, rounding down, NOT towards zero.
#[inline]
fn RCL_divRoundDown(value:RCL_Unit, divisor:RCL_Unit) -> RCL_Unit{
	unsafe { profile::RCL_divRoundDown.call(); }

	return value / divisor - (if value >= 0 { 0 } else { 1 });
}

/// Bhaskara's cosine approximation formula
// #define
fn trigHelper(x:RCL_Unit) -> RCL_Unit { // TODO: X's type is yet unknown
	(RCL_UNITS_PER_SQUARE *
		(RCL_UNITS_PER_SQUARE / 2 * RCL_UNITS_PER_SQUARE / 2 - 4 * x * x) /
		(RCL_UNITS_PER_SQUARE / 2 * RCL_UNITS_PER_SQUARE / 2 +     x * x))
}

/**
Cos function.

@param  input to cos in RCL_Units (RCL_UNITS_PER_SQUARE = 2 * pi = 360 degrees)
@return RCL_normalized output in RCL_Units (from -RCL_UNITS_PER_SQUARE to
				RCL_UNITS_PER_SQUARE)
*/
fn RCL_cosInt(input:RCL_Unit) -> RCL_Unit {
	unsafe { profile::RCL_cosInt.call(); }

	let input = RCL_wrap(input,RCL_UNITS_PER_SQUARE);

	if RCL_USE_COS_LUT == 1 {
		return if RCL_RAYCAST_TINY {
			cosLUT[input as usize]
		} else {
			cosLUT[(input / 16) as usize]
		}
	} else if RCL_USE_COS_LUT == 2 {
		return cosLUT[(input / 8) as usize];
	} else {
		if input < RCL_UNITS_PER_SQUARE / 4 {
			return trigHelper(input);
		} else if input < RCL_UNITS_PER_SQUARE / 2 {
			return -1 * trigHelper(RCL_UNITS_PER_SQUARE / 2 - input);
		} else if input < 3 * RCL_UNITS_PER_SQUARE / 4 {
			return -1 * trigHelper(input - RCL_UNITS_PER_SQUARE / 2);
		} else {
			return trigHelper(RCL_UNITS_PER_SQUARE - input);
		}
	}
}

// #undef trigHelper

fn RCL_sinInt(input:RCL_Unit) -> RCL_Unit {
	return RCL_cosInt(input - RCL_UNITS_PER_SQUARE / 4);
}

pub fn RCL_angleToDirection(angle:RCL_Unit) -> RCL_Vector2D {
	unsafe { profile::RCL_angleToDirection.call() };

	return RCL_Vector2D {
		x:      RCL_cosInt(angle),
		y: -1 * RCL_sinInt(angle),
	};
}

fn RCL_sqrtInt(value:RCL_Unit) -> RCL_Unit_unsigned {
	unsafe { profile::RCL_sqrtInt.call() };

	let mut result:RCL_Unit_unsigned = 0;
	let mut a:RCL_Unit_unsigned = value as RCL_Unit_unsigned;
	let mut b:RCL_Unit_unsigned = 1 << if RCL_RAYCAST_TINY { 14 } else { 30 };

	while b > a {
		b >>= 2;
	}

	while b != 0 {
		if a >= result + b {
			a -= result + b;
			result = result +  2 * b;
		}

		b >>= 2;
		result >>= 1;
	}

	return result;
}

#[inline]
fn RCL_pointIsLeftOfRay(point:RCL_Vector2D, ray:RCL_Ray) -> bool {
	unsafe { profile::RCL_pointIsLeftOfRay.call(); }

	let dX = point.x - ray.start.x;
	let dY = point.y - ray.start.y;
	return (ray.direction.x * dY - ray.direction.y * dX) > 0;
	// ^ Z component of cross-product
}


///  Converts an angle in whole degrees to an angle in RCL_Units that this library uses.
fn RCL_degreesToUnitsAngle(degrees:i16) -> RCL_Unit {
	return (degrees as RCL_Unit * RCL_UNITS_PER_SQUARE) / 360;
}

/// Computes the change in size of an object due to perspective.
pub fn RCL_perspectiveScale(originalSize:RCL_Unit, distance:RCL_Unit) -> RCL_Unit {
	unsafe { profile::RCL_perspectiveScale.call(); }

	return if distance != 0 {
		(originalSize * RCL_UNITS_PER_SQUARE) / ((RCL_VERTICAL_FOV * 2 * distance) / RCL_UNITS_PER_SQUARE)
	} else {
		0
	}
}

fn RCL_perspectiveScaleInverse(originalSize:RCL_Unit, scaledSize:RCL_Unit) -> RCL_Unit {
	if scaledSize != 0 {
		(originalSize * RCL_UNITS_PER_SQUARE + RCL_UNITS_PER_SQUARE / 2) /
			// ^ take the middle
			((RCL_VERTICAL_FOV * 2 * scaledSize) / RCL_UNITS_PER_SQUARE)
	} else {
		RCL_INFINITY
	}
}

// Maps a single point in the world to the screen (2D position + depth).
pub fn RCL_mapToScreen(worldPosition:RCL_Vector2D, height:RCL_Unit, camera:&RCL_Camera) -> RCL_PixelInfo {
	let mut result:RCL_PixelInfo = RCL_PixelInfo {
		position: RCL_Vector2D::ZERO,
		isWall: false,
		isFloor: false,
		isHorizon: false,
		depth: 0,
		wallHeight: 0,
		height: 0,
		hit: RCL_HitResult::zeroed(),
		texCoords: RCL_Vector2D::ZERO,
	};

	let mut toPoint:RCL_Vector2D = RCL_Vector2D {
		x: worldPosition.x - camera.position.x,
		y: worldPosition.y - camera.position.y
	};

	let middleColumn = CAMERA_RESOLUTION_X / 2;

	// rotate the point

	let cos = RCL_cosInt(camera.direction);
	let sin = RCL_sinInt(camera.direction);

	let tmp = toPoint.x;

	toPoint.x = (toPoint.x * cos - toPoint.y * sin) / RCL_UNITS_PER_SQUARE;
	toPoint.y = (tmp * sin + toPoint.y * cos) / RCL_UNITS_PER_SQUARE;

	result.depth = toPoint.x;

	result.position.x = middleColumn as RCL_Unit + (-1 * toPoint.y * middleColumn as RCL_Unit) / RCL_nonZero(result.depth);

	result.position.y = halfResY as RCL_Unit - (CAMERA_RESOLUTION_Y as RCL_Unit * RCL_perspectiveScale(height - camera.height,result.depth)) / RCL_UNITS_PER_SQUARE + camera.shear as RCL_Unit;

	return result;
}

// global helper variables, for precomputing stuff etc.
pub struct RCL_Renderer_Global {
	_RCL_camera:RCL_Camera,
	_RCL_horizontalDepthStep:RCL_Unit,
	_RCL_startFloorHeight:RCL_Unit,
	_RCL_startCeil_Height:RCL_Unit,
	_RCL_middleRow:i16,
	_RCL_fHorizontalDepthStart:i16,
	_RCL_cHorizontalDepthStart:i16,
	_RCL_cameraHeightScreen:i16,
	_RCL_rollFunction:Option<RCL_ArrayFunction>, // says door rolling
	_RCL_floorPixelDistances:Option<[RCL_Unit; CAMERA_RESOLUTION_Y as usize]>,
}

impl RCL_Renderer_Global {
	pub const fn new() -> RCL_Renderer_Global {
		RCL_Renderer_Global {
			_RCL_camera: RCL_Camera {
				position: RCL_Vector2D::ZERO,
				direction: 0,
				resolution: RCL_Vector2D::ZERO,
				shear: 0,
				height: 0,
			},
			_RCL_horizontalDepthStep: 0,
			_RCL_startFloorHeight: 0,
			_RCL_startCeil_Height: 0,
			_RCL_middleRow: 0,
			_RCL_fHorizontalDepthStart: 0,
			_RCL_cHorizontalDepthStart: 0,
			_RCL_cameraHeightScreen: 0,
			_RCL_rollFunction: None,
			_RCL_floorPixelDistances: None,
		}
	}

	/**
		Casts a single ray and returns a list of collisions.

		@param ray ray to be cast, if RCL_RECTILINEAR != 0 then the computed hit
					 distance is divided by the ray direction vector length (to correct
					 the fish eye effect)
		@param arrayFunc function that will be used to determine collisions (hits)
					 with the ray (squares for which this function returns different values
					 are considered to have a collision between them), this will typically
					 be a function returning floor height
		@param typeFunc optional (can be 0) function - if provided, it will be used
					 to mark the hit result with the number returned by this function
					 (it can be e.g. a texture index)
		@param hitResults array in which the hit results will be stored (has to be
					 preallocated with at space for at least as many hit results as
					 maxHits specified with the constraints parameter)
		@param hitResultsLen in this variable the number of hit results will be
					 returned
		@param constraints specifies constraints for the ray cast
	*/
	fn RCL_castRayMultiHit(
		&mut self,
		ray:RCL_Ray,
		arrayFunc:RCL_ArrayFunction,
		typeFunc:Option<RCL_ArrayFunction>,
		hitResults:&mut [RCL_HitResult],
		hitResultsLen:&mut u8,
		constraints:RCL_RayConstraints
	) {
		unsafe { profile::RCL_castRayMultiHit.call(); }

		assert!(!hitResults.is_empty()); // Should prevent runtime checking

		let currentPos = ray.start;
		let mut currentSquare = RCL_Vector2D {
			x: RCL_divRoundDown(ray.start.x,RCL_UNITS_PER_SQUARE),
			y: RCL_divRoundDown(ray.start.y,RCL_UNITS_PER_SQUARE),
		};

		*hitResultsLen = 0;

		let mut squareType:RCL_Unit = arrayFunc(currentSquare.x as i16, currentSquare.y as i16);

		// DDA variables
		let mut nextSideDist = RCL_Vector2D::ZERO; // dist. from start to the next side in given axis
		let mut step = RCL_Vector2D::ZERO; // -1 or 1 for each axis
		let mut stepHorizontal:bool = false; // whether the last step was hor. or vert.

		let dirVecLengthNorm = ray.direction.len() * RCL_UNITS_PER_SQUARE;

		let delta = RCL_Vector2D {
			x: RCL_absVal(dirVecLengthNorm / RCL_nonZero(ray.direction.x)),
			y: RCL_absVal(dirVecLengthNorm / RCL_nonZero(ray.direction.y)),
		};

		// init DDA

		if ray.direction.x < 0 {
			step.x = -1;
			nextSideDist.x = (RCL_wrap(ray.start.x,RCL_UNITS_PER_SQUARE) * delta.x) / RCL_UNITS_PER_SQUARE;
		} else {
			step.x = 1;
			nextSideDist.x = ((RCL_wrap(RCL_UNITS_PER_SQUARE - ray.start.x,RCL_UNITS_PER_SQUARE)) * delta.x) / RCL_UNITS_PER_SQUARE;
		}

		if ray.direction.y < 0 {
			step.y = -1;
			nextSideDist.y = (RCL_wrap(ray.start.y,RCL_UNITS_PER_SQUARE) * delta.y) / RCL_UNITS_PER_SQUARE;
		} else {
			step.y = 1;
			nextSideDist.y = ((RCL_wrap(RCL_UNITS_PER_SQUARE - ray.start.y,RCL_UNITS_PER_SQUARE)) * delta.y) / RCL_UNITS_PER_SQUARE;
		}

		// DDA loop

		const RECIP_SCALE:RCL_Unit = 65536;

		let rayDirXRecip = RECIP_SCALE / RCL_nonZero(ray.direction.x);
		let rayDirYRecip = RECIP_SCALE / RCL_nonZero(ray.direction.y);
		// ^ we precompute reciprocals to avoid divisions in the loop

		for i in 0..constraints.maxSteps {
			let currentType:RCL_Unit = arrayFunc(currentSquare.x as i16, currentSquare.y as i16);

			if RCL_unlikely(currentType != squareType) {
				// collision

				let mut h = RCL_HitResult::zeroed();

				h.arrayValue = currentType;
				h.doorRoll = 0;
				h.position = currentPos;
				h.square   = currentSquare;

				if stepHorizontal {
					h.position.x = currentSquare.x * RCL_UNITS_PER_SQUARE;
					h.direction = 3;

					if step.x == -1 {
						h.direction = 1;
						h.position.x += RCL_UNITS_PER_SQUARE;
					}

					let diff = h.position.x - ray.start.x;

					// avoid division by multiplying with reciprocal
					h.position.y = ray.start.y + ((ray.direction.y * diff) * rayDirXRecip) / RECIP_SCALE;

					if RCL_RECTILINEAR {
						// Here we compute the fish eye corrected distance (perpendicular to
						// the projection plane) as the Euclidean distance divided by the length
						// of the ray direction vector. This can be computed without actually
						// computing Euclidean distances as a hypothenuse A (distance) divided
						// by hypothenuse B (length) is equal to leg A (distance along one axis)
						// divided by leg B (length along the same axis).

						h.distance = (((h.position.x - ray.start.x) / 4) * RCL_UNITS_PER_SQUARE * rayDirXRecip) / (RECIP_SCALE / 4); // "/ 4" is here to prevent overflow
					}
				} else {
					h.position.y = currentSquare.y * RCL_UNITS_PER_SQUARE;
					h.direction = 2;

					if step.y == -1 {
						h.direction = 0;
						h.position.y += RCL_UNITS_PER_SQUARE;
					}

					let diff = h.position.y - ray.start.y;

					h.position.x = ray.start.x + ((ray.direction.x * diff) * rayDirYRecip) / RECIP_SCALE;

					if RCL_RECTILINEAR {
						h.distance = (((h.position.y - ray.start.y) / 4) * RCL_UNITS_PER_SQUARE * rayDirYRecip) / (RECIP_SCALE / 4); // "^ / 4" is here to prevent overflow
					}
				}

				if !RCL_RECTILINEAR {
					h.distance = RCL_Vector2D::dist(h.position, ray.start);
				}

				if let Some(typeFunc) = typeFunc {
					h.type_ = typeFunc(currentSquare.x as i16, currentSquare.y as i16);
				}

				if RCL_COMPUTE_WALL_TEXCOORDS {
					h.textureCoord = match h.direction {
						0 => RCL_wrap(-1 * h.position.x,RCL_UNITS_PER_SQUARE),
						1 => RCL_wrap(h.position.y,RCL_UNITS_PER_SQUARE),
						2 => RCL_wrap(h.position.x,RCL_UNITS_PER_SQUARE),
						3 => RCL_wrap(-1 * h.position.y,RCL_UNITS_PER_SQUARE),
						_ => 0,
					};

					if let Some(roll_function) = self._RCL_rollFunction {
						h.doorRoll = roll_function(currentSquare.x as i16, currentSquare.y as i16);

						if h.direction == 0 || h.direction == 1 {
							h.doorRoll *= -1;
						}
					}
				} else {
					h.textureCoord = 0;
				}

				hitResults[*hitResultsLen as usize] = h;

				*hitResultsLen += 1;

				squareType = currentType;

				if *hitResultsLen as usize >= hitResults.len() {
					break;
				}
			}

			// DDA step

			if nextSideDist.x < nextSideDist.y {
				nextSideDist.x += delta.x;
				currentSquare.x += step.x;
				stepHorizontal = true;
			} else {
				nextSideDist.y += delta.y;
				currentSquare.y += step.y;
				stepHorizontal = false;
			}
		}
	}

	/**
		Simple-interface function to cast a single ray.
		@return          The first collision result.
	*/
	fn RCL_castRay(&mut self, ray:RCL_Ray, arrayFunc:RCL_ArrayFunction) -> RCL_HitResult {
		unsafe { profile::RCL_castRay.call(); }

		let mut result = [RCL_HitResult::zeroed()];
		let mut RCL_len = 0;
		let c = RCL_RayConstraints {
			maxSteps: 1000,
			maxHits: 1,
		};

		self.RCL_castRayMultiHit(ray,arrayFunc,None,&mut result,&mut RCL_len,c);

		let mut result = result[0];

		if RCL_len == 0 {
			result.distance = -1;
		}

		return result;
	}

	/// Casts rays for given camera view and for each hit calls a user provided function.
	fn RCL_castRaysMultiHit(
		&mut self, general: &mut RCL_General, cam:&RCL_Camera,
		arrayFunc:RCL_ArrayFunction,
		typeFunction:Option<RCL_ArrayFunction>,
		columnFunc:RCL_ColumnFunction,
	) {
		let dir1 = RCL_angleToDirection(cam.direction - RCL_HORIZONTAL_FOV_HALF);
		let dir2 = RCL_angleToDirection(cam.direction + RCL_HORIZONTAL_FOV_HALF);

		let dX = dir2.x - dir1.x;
		let dY = dir2.y - dir1.y;

		let mut hits = [RCL_HitResult::zeroed(); HITS_ARRAY_LIMIT as usize];
		let mut hitCount:u8 = 0;

		let mut r = RCL_Ray {
			start: cam.position,
			direction: RCL_Vector2D::ZERO,
		};

		let mut currentDX:RCL_Unit = 0;
		let mut currentDY:RCL_Unit = 0;

		for i in 0..CAMERA_RESOLUTION_X {
			// Here by linearly interpolating the direction vector its length changes,
			// which in result achieves correcting the fish eye effect (computing
			// perpendicular distance).

			r.direction.x = dir1.x + currentDX / CAMERA_RESOLUTION_X as RCL_Unit;
			r.direction.y = dir1.y + currentDY / CAMERA_RESOLUTION_Y as RCL_Unit;

			self.RCL_castRayMultiHit(r, arrayFunc,typeFunction,&mut hits,&mut hitCount, general.defaultConstraints);

			columnFunc(self, general, &hits[0..hitCount as usize], i, r);

			currentDX += dX;
			currentDY += dY;
		}
	}

	/// Helper function that determines intersection with both ceiling and floor.
	fn _RCL_floorCeilFunction(x:i16, y:i16) -> RCL_Unit {
		let f = floorHeightFunction(x, y);

		match ceilingHeightFunc {
			None => f,
			Some(chf) => {
				let c = chf(x, y);

				if !RCL_RAYCAST_TINY {
					((f & 0x0000ffff) << 16) | (c & 0x0000ffff)
				} else {
					((f & 0x00ff) << 8) | (c & 0x00ff)
				}
			},
		}
	}

	fn _floorHeightNotZeroFunction(x:i16, y:i16) -> RCL_Unit {
		if floorHeightFunction(x, y) == 0 {
			0
		} else {
			RCL_nonZero(((x & 0x00FF) | ((y & 0x00FF) << 8)) as RCL_Unit)
			// ^ this makes collisions between all squares - needed for rolling doors
		}
	}

	fn RCL_adjustDistance(distance:RCL_Unit, camera:&RCL_Camera, ray:&RCL_Ray) -> RCL_Unit {
		/* FIXME/TODO: The adjusted (=orthogonal, camera-space) distance could
			 possibly be computed more efficiently by not computing Euclidean
			 distance at all, but rather compute the distance of the collision
			 point from the projection plane (line). */

		let result = (distance * RCL_Vector2D::angleCos(RCL_angleToDirection(camera.direction), ray.direction)) / RCL_UNITS_PER_SQUARE;

		return RCL_nonZero(result);
				// ^ prevent division by zero
	}

	/// Helper for drawing floor or ceiling. Returns the last drawn pixel position.
	#[inline]
	fn _RCL_drawHorizontalColumn(
		&mut self,
		general:&mut RCL_General,
		yCurrent:i16,
		yTo:RCL_Unit,
		limit1:i16,
		limit2:i16,
		verticalOffset:RCL_Unit,
		increment:NonZeroSignum,
		computeDepth:bool,
		computeCoords:bool,
		depthIncrementMultiplier:i8,
		ray:&RCL_Ray,
		pixelInfo:&mut RCL_PixelInfo
	) -> i16 {
		_RCL_UNUSED(ray);

		let mut depthIncrement:RCL_Unit = 0;
		let mut dx:RCL_Unit = 0;
		let mut dy:RCL_Unit = 0;

		pixelInfo.isWall = false;

		let limit = RCL_clamp(yTo,limit1 as RCL_Unit,limit2 as RCL_Unit) as i16;

		// TODO: this is for clamping depth to 0 so that we don't have negative depths, but we should do it more elegantly and efficiently
		let mut depth:RCL_Unit = 0;

		_RCL_UNUSED(depth);

		let doDepth;
		let doCoords;

		if computeDepth { // branch early
			doDepth = true;
			if !computeCoords {
				doCoords = false;
			} else {
				doCoords = true;
			}
		} else {
			if !computeCoords {
				doDepth = false;
				doCoords = false;
			} else {
				doDepth = true;
				doCoords = true;
			}
		}

		// for performance reasons have different version of the critical loop to be able to branch early
		if doDepth { // constant condition - compiler should optimize it out
			depth = pixelInfo.depth + RCL_absVal(verticalOffset) * RCL_VERTICAL_DEPTH_MULTIPLY;
			depthIncrement = depthIncrementMultiplier as RCL_Unit * self._RCL_horizontalDepthStep;
		}
		if doCoords { // constant condition - compiler should optimize it out
			dx = pixelInfo.hit.position.x - self._RCL_camera.position.x;
			dy = pixelInfo.hit.position.y - self._RCL_camera.position.y;
		}
		let mut i:i16 = yCurrent + increment;
		while if increment.is_negative() { i >= limit } else { i <= limit } {
			pixelInfo.position.y = i.into();
			if doDepth { // constant condition - compiler should optimize it out
				depth += depthIncrement;
				pixelInfo.depth = RCL_zeroClamp(depth);
				// ^ int comparison is fast, it is not braching! (= test instr.)
			}
			if doCoords { // constant condition - compiler should optimize it out
				let d = self._RCL_floorPixelDistances.unwrap()[i as usize]; // TODO: remove unwrap
				let d2 = RCL_nonZero(pixelInfo.hit.distance);
				pixelInfo.texCoords.x = self._RCL_camera.position.x + ((d * dx) / d2);
				pixelInfo.texCoords.y = self._RCL_camera.position.y + ((d * dy) / d2);
			}
			pixelFunc(general, pixelInfo);
			/* TODO: is efficient? */ i += increment;
		}

		return limit;
	}

	/// Helper for drawing walls. Returns the last drawn pixel position.
	#[inline]
	fn _RCL_drawWall(
		general:&mut RCL_General,
		yCurrent:i16,
		yFrom:RCL_Unit,
		yTo:RCL_Unit,
		limit1:i16,
		limit2:i16,
		mut height:RCL_Unit,
		increment:NonZeroSignum,
		pixelInfo:&mut RCL_PixelInfo
	) -> i16 {
		height = RCL_absVal(height);

		pixelInfo.isWall = true;

		let limit = RCL_clamp(yTo,limit1 as RCL_Unit,limit2 as RCL_Unit) as i16;

		let wallLength:RCL_Unit = RCL_nonZero(RCL_absVal(yTo - yFrom - 1));

		let wallPosition:RCL_Unit = RCL_absVal(yFrom - yCurrent as RCL_Unit) - increment;

		let heightScaled:RCL_Unit = height * RCL_TEXTURE_INTERPOLATION_SCALE;

		let mut coordStepScaled:RCL_Unit = if RCL_COMPUTE_WALL_TEXCOORDS {
			if RCL_TEXTURE_VERTICAL_STRETCH == 1 {
				((RCL_UNITS_PER_SQUARE * RCL_TEXTURE_INTERPOLATION_SCALE) / wallLength)
			} else {
				(heightScaled / wallLength)
			}
		} else {
			0
		};

		pixelInfo.texCoords.y = if RCL_COMPUTE_WALL_TEXCOORDS { wallPosition * coordStepScaled } else { 0 };

		if increment.is_negative() {
			coordStepScaled *= -1;
			pixelInfo.texCoords.y =
				if RCL_TEXTURE_VERTICAL_STRETCH == 1 {
					(RCL_UNITS_PER_SQUARE * RCL_TEXTURE_INTERPOLATION_SCALE) - pixelInfo.texCoords.y
				} else {
					heightScaled - pixelInfo.texCoords.y
				}
			;
		} else {
			// with floor wall, don't start under 0
			pixelInfo.texCoords.y = RCL_zeroClamp(pixelInfo.texCoords.y);
		}

		let mut textureCoordScaled:RCL_Unit = pixelInfo.texCoords.y;

		let mut i = yCurrent + increment;
		while if increment.is_negative() { i >= limit } else { i <= limit } { // TODO: is efficient?
			pixelInfo.position.y = i as RCL_Unit;

			if RCL_COMPUTE_WALL_TEXCOORDS {
				pixelInfo.texCoords.y = textureCoordScaled / RCL_TEXTURE_INTERPOLATION_SCALE;
				textureCoordScaled += coordStepScaled;
			}

			pixelFunc(general, pixelInfo);

			i += increment;
		}

		return limit;
	}

	fn _RCL_columnFunctionComplex(&mut self, general:&mut RCL_General, hits:&[RCL_HitResult], x:u16, mut ray:RCL_Ray) {
		// last written Y position, can never go backwards
		let mut fPosY = CAMERA_RESOLUTION_Y as i16;
		let mut cPosY = -1i16;

		// world coordinates (relative to camera height though)
		let mut fZ1World = self._RCL_startFloorHeight;
		let mut cZ1World = self._RCL_startCeil_Height;

		let mut p:RCL_PixelInfo = RCL_PixelInfo::zeroed();
		p.position.x = x as RCL_Unit;
		p.height = 0;
		p.wallHeight = 0;
		p.texCoords = RCL_Vector2D::ZERO;

		// we'll be simulatenously drawing the floor and the ceiling now
		for j in 0..=hits.len() {
			//              ^ "=" add extra iteration for horizon plane
			let drawingHorizon:bool = j == hits.len();

			let mut hit = RCL_HitResult::zeroed();
			let mut distance = 1;

			let mut fWallHeight = 0;
			let mut cWallHeight = 0;
			let mut fZ2World = 0;
			let mut cZ2World = 0;
			let mut fZ1Screen = 0;
			let mut cZ1Screen = 0;
			let mut fZ2Screen = 0;
			let mut cZ2Screen = 0;

			if !drawingHorizon {
				hit = hits[j as usize];
				distance = RCL_nonZero(hit.distance);
				p.hit = hit.clone();

				fWallHeight = floorHeightFunction(hit.square.x as i16, hit.square.y as i16);
				fZ2World = fWallHeight - self._RCL_camera.height;
				fZ1Screen = self._RCL_middleRow as RCL_Unit - RCL_perspectiveScale((fZ1World * CAMERA_RESOLUTION_Y as RCL_Unit) / RCL_UNITS_PER_SQUARE, distance);
				fZ2Screen = self._RCL_middleRow as RCL_Unit - RCL_perspectiveScale((fZ2World * CAMERA_RESOLUTION_Y as RCL_Unit) / RCL_UNITS_PER_SQUARE, distance);

				if let Some(chf) = ceilingHeightFunc {
					cWallHeight = chf(hit.square.x as i16, hit.square.y as i16);
					cZ2World = cWallHeight - self._RCL_camera.height;
					cZ1Screen = self._RCL_middleRow as RCL_Unit - RCL_perspectiveScale((cZ1World * CAMERA_RESOLUTION_Y as RCL_Unit) / RCL_UNITS_PER_SQUARE, distance);
					cZ2Screen = self._RCL_middleRow as RCL_Unit - RCL_perspectiveScale((cZ2World * CAMERA_RESOLUTION_Y as RCL_Unit) / RCL_UNITS_PER_SQUARE, distance);
				}
			} else {
				fZ1Screen =  self._RCL_middleRow      as RCL_Unit;
				cZ1Screen = (self._RCL_middleRow + 1) as RCL_Unit;
				p.hit._RCL_makeInfiniteHit(&ray);
			}

			let mut limit;

			p.isWall = false;
			p.isHorizon = drawingHorizon;

			// draw floor until wall
			p.isFloor = true;
			p.height = fZ1World + self._RCL_camera.height;
			p.wallHeight = 0;

			if RCL_COMPUTE_FLOOR_DEPTH {
				p.depth = (self._RCL_fHorizontalDepthStart - fPosY) as RCL_Unit * self._RCL_horizontalDepthStep;
			} else {
				p.depth = 0;
			}

			limit = self._RCL_drawHorizontalColumn(general, fPosY,fZ1Screen,cPosY + 1,
			 CAMERA_RESOLUTION_Y as i16,fZ1World,NonZeroSignum::NEG,RCL_COMPUTE_FLOOR_DEPTH,
			 // ^ purposfully allow outside screen bounds
				 RCL_COMPUTE_FLOOR_TEXCOORDS && p.height == RCL_FLOOR_TEXCOORDS_HEIGHT,
				 1, &ray, &mut p);

			if fPosY > limit {
				fPosY = limit;
			}

			if ceilingHeightFunc.is_some() || drawingHorizon {
				// draw ceiling until wall
				p.isFloor = false;
				p.height = cZ1World + self._RCL_camera.height;

				if RCL_COMPUTE_CEILING_DEPTH {
					p.depth = (cPosY - self._RCL_cHorizontalDepthStart) as RCL_Unit * self._RCL_horizontalDepthStep;
				}

				limit = self._RCL_drawHorizontalColumn(general, cPosY,cZ1Screen,
					-1,fPosY as i16 - 1,cZ1World,NonZeroSignum::POS, RCL_COMPUTE_CEILING_DEPTH, false,1, &ray,&mut p);
				// ^ purposfully allow outside screen bounds here

				if cPosY < limit {
					cPosY = limit;
				}
			}

			if !drawingHorizon { // don't draw walls for horizon plane
				p.isWall = true;
				p.depth = distance;
				p.isFloor = true;
				p.texCoords.x = hit.textureCoord;
				p.height = fZ1World + self._RCL_camera.height;
				p.wallHeight = fWallHeight;

				// draw floor wall

				if fPosY > 0 { // still pixels left?
					p.isFloor = true;

					limit = Self::_RCL_drawWall(general, fPosY,fZ1Screen,fZ2Screen,cPosY + 1,
										CAMERA_RESOLUTION_Y as i16,
										// ^ purposfully allow outside screen bounds here
										if RCL_TEXTURE_VERTICAL_STRETCH == 1 {
											RCL_UNITS_PER_SQUARE
										} else {
											fZ2World - fZ1World
										}
										,NonZeroSignum::NEG, &mut p);


					if fPosY > limit {
						fPosY = limit;
					}

					// for the next iteration
					// purposfully allow outside screen bounds here
					fZ1World = fZ2World;
				}

				// draw ceiling wall

				if ceilingHeightFunc.is_some() && cPosY < _RCL_camResYLimit as i16 { // pixels left?
					p.isFloor = false;
					p.height = cZ1World + self._RCL_camera.height;
					p.wallHeight = cWallHeight;

					limit = Self::_RCL_drawWall(general, cPosY,cZ1Screen,cZ2Screen,
										-1,fPosY - 1,
									// ^ puposfully allow outside screen bounds here
							if RCL_TEXTURE_VERTICAL_STRETCH == 1 {
											RCL_UNITS_PER_SQUARE
										} else {
											cZ1World - cZ2World
										},
										NonZeroSignum::POS,&mut p);

					if cPosY < limit {
						cPosY = limit;
					}

					// for the next iteration
					// puposfully allow outside screen bounds here
					cZ1World = cZ2World;
				}
			}
		}
	}

	fn _RCL_columnFunctionSimple(&mut self, general:&mut RCL_General, hits:&[RCL_HitResult], x:u16, ray:RCL_Ray) {
		let mut y = 0;
		let mut wallHeightScreen:RCL_Unit = 0;
		let mut wallStart:RCL_Unit = self._RCL_middleRow as RCL_Unit;
		let mut heightOffset:RCL_Unit = 0;

		let mut dist:RCL_Unit = 1;

		let mut p = RCL_PixelInfo::zeroed();
		p.position.x = x as RCL_Unit;
		p.wallHeight = RCL_UNITS_PER_SQUARE;

		if hits.len() > 0 {
			let mut hit = &hits[0];

			let mut goOn = true;

			if self._RCL_rollFunction.is_some() && RCL_COMPUTE_WALL_TEXCOORDS {
				if hit.arrayValue == 0 {
					// standing inside door square, looking out => move to the next hit

					if hits.len() > 1 {
						hit = &hits[1];
					} else {
						goOn = false;
					}
				} else {
					// normal hit, check the door roll

					let texCoordMod:RCL_Unit = hit.textureCoord % RCL_UNITS_PER_SQUARE;

					let unrolled = if hit.doorRoll >= 0 {
						hit.doorRoll > texCoordMod
					} else {
						texCoordMod > RCL_UNITS_PER_SQUARE + hit.doorRoll
					};

					if unrolled {
						goOn = false;

						if hits.len() > 1 { // should probably always be true (hit on square exit)
							if hit.direction % 2 != hits[1].direction % 2 {
								// hit on the inner side
								hit = &hits[1];
								goOn = true;
							} else if hits.len() > 2 {
								// hit on the opposite side
								hit = &hits[2];
								goOn = true;
							}
						}
					}
				}
			}

			p.hit = hit.clone();

			if goOn {
				dist = hit.distance;

				let wallHeightWorld = floorHeightFunction(hit.square.x as i16, hit.square.y as i16); // Was :i16

				wallHeightScreen = RCL_perspectiveScale((wallHeightWorld * CAMERA_RESOLUTION_Y as RCL_Unit) / RCL_UNITS_PER_SQUARE,dist);

				let RCL_normalizedWallHeight = if wallHeightWorld != 0 {
					((RCL_UNITS_PER_SQUARE * wallHeightScreen) / wallHeightWorld)
				} else {
					0
				};

				heightOffset = RCL_perspectiveScale(self._RCL_cameraHeightScreen as RCL_Unit, dist);

				wallStart = self._RCL_middleRow as RCL_Unit - wallHeightScreen + heightOffset + RCL_normalizedWallHeight;
			}
		} else {
			p.hit._RCL_makeInfiniteHit(&ray);
		}

		// draw ceiling

		p.isWall = false;
		p.isFloor = false;
		p.isHorizon = true;
		p.depth = 1;
		p.height = RCL_UNITS_PER_SQUARE;

		y = self._RCL_drawHorizontalColumn(general, -1,wallStart,-1,self._RCL_middleRow,self._RCL_camera.height,NonZeroSignum::POS, RCL_COMPUTE_CEILING_DEPTH, false, 1, &ray, &mut p);

		// draw wall

		p.isWall = true;
		p.isFloor = true;
		p.depth = dist;
		p.height = 0;

		if RCL_ROLL_TEXTURE_COORDS && RCL_COMPUTE_WALL_TEXCOORDS {
			p.hit.textureCoord -= p.hit.doorRoll;
		}

		p.texCoords.x = p.hit.textureCoord;
		p.texCoords.y = 0;

		let limit = Self::_RCL_drawWall(general, y,wallStart,wallStart + wallHeightScreen - 1, -1,_RCL_camResYLimit as i16,p.hit.arrayValue,NonZeroSignum::POS,&mut p);

		y = y.max(limit); // take max, in case no wall was drawn
		y = (y as i32).max(wallStart) as i16;

		// draw floor

		p.isWall = false;

		if RCL_COMPUTE_FLOOR_DEPTH {
			p.depth = (CAMERA_RESOLUTION_Y as RCL_Unit - y as RCL_Unit) * self._RCL_horizontalDepthStep + 1;
		}

		self._RCL_drawHorizontalColumn(general, y,_RCL_camResYLimit as RCL_Unit,-1,_RCL_camResYLimit as i16, self._RCL_camera.height,NonZeroSignum::POS,RCL_COMPUTE_FLOOR_DEPTH,RCL_COMPUTE_FLOOR_TEXCOORDS, -1,&ray,&mut p);
	}

	// Precomputes a distance from camera to the floor at each screen row into an array (must be preallocated with sufficient (CAMERA_RESOLUTION_Y) length).
	#[inline]
	fn _RCL_precomputeFloorDistances(&self, camera:&RCL_Camera, startIndex:u16) -> [RCL_Unit; CAMERA_RESOLUTION_Y as usize] {
		let mut floorPixelDistances = [0 as RCL_Unit;CAMERA_RESOLUTION_Y as usize];

		let camHeightScreenSize = (camera.height * CAMERA_RESOLUTION_Y as RCL_Unit) / RCL_UNITS_PER_SQUARE;

		for i in startIndex..CAMERA_RESOLUTION_Y {
			floorPixelDistances[i as usize] = RCL_perspectiveScaleInverse(camHeightScreenSize, RCL_absVal(i as RCL_Unit - self._RCL_middleRow as RCL_Unit));
		}

		return floorPixelDistances;
	}

	/**
		Using provided functions, renders a complete complex (multilevel) camera
		view.

		This function should render each screen pixel exactly once.

		function rendering summary:
		- performance:            slower
		- accuracy:               higher
		- wall textures:          yes
		- different wall heights: yes
		- floor/ceiling textures: no
		- floor geometry:         yes, multilevel
		- ceiling geometry:       yes (optional), multilevel
		- rolling door:           no
		- camera shearing:        yes
		- rendering order:        left-to-right, not specifically ordered vertically

		@param cam camera whose view to render
		@param floorHeightFunc function that returns floor height (in RCL_Units)
		@param ceilingHeightFunc same as floorHeightFunc but for ceiling, can also be
														 0 (no ceiling will be rendered)
		@param typeFunction function that says a type of square (e.g. its texture
											 index), can be 0 (no type in hit result)
		@param pixelFunc callback function to draw a single pixel on screen
		@param constraints constraints for each cast ray
	*/
	pub fn RCL_renderComplex(&mut self, general:&mut RCL_General, cam:RCL_Camera, typeFunction:Option<RCL_ArrayFunction>) {
		self._RCL_camera = cam.clone();

		self._RCL_middleRow = halfResY as i16 + cam.shear;

		self._RCL_fHorizontalDepthStart = self._RCL_middleRow + halfResY as i16;
		self._RCL_cHorizontalDepthStart = self._RCL_middleRow - halfResY as i16;

		self._RCL_startFloorHeight =
			floorHeightFunction(
				RCL_divRoundDown(cam.position.x,RCL_UNITS_PER_SQUARE) as i16,
				RCL_divRoundDown(cam.position.y,RCL_UNITS_PER_SQUARE) as i16
			) - cam.height;

		self._RCL_startCeil_Height =
			if let Some(chf) = ceilingHeightFunc {
				chf(
					RCL_divRoundDown(cam.position.x,RCL_UNITS_PER_SQUARE) as i16,
					RCL_divRoundDown(cam.position.y,RCL_UNITS_PER_SQUARE) as i16
				) - cam.height
			} else {
				RCL_INFINITY
			};

		self._RCL_horizontalDepthStep = RCL_HORIZON_DEPTH / CAMERA_RESOLUTION_Y as RCL_Unit;

		if RCL_COMPUTE_FLOOR_TEXCOORDS {
			let floorPixelDistances = self._RCL_precomputeFloorDistances(&cam, 0);
			self._RCL_floorPixelDistances = Some(floorPixelDistances); // pass to column function
		}

		self.RCL_castRaysMultiHit(general, &cam, Self::_RCL_floorCeilFunction, typeFunction, Self::_RCL_columnFunctionComplex);
	}


	/**
		Renders given camera view, with help of provided functions. This function is
		simpler and faster than RCL_renderComplex(...) and is meant to be rendering
		flat levels.

		function rendering summary:
		- performance:            faster
		- accuracy:               lower
		- wall textures:          yes
		- different wall heights: yes
		- floor/ceiling textures: yes (only floor, you can mirror it for ceiling)
		- floor geometry:         no (just flat floor, with depth information)
		- ceiling geometry:       no (just flat ceiling, with depth information)
		- rolling door:           yes
		- camera shearing:        no
		- rendering order:        left-to-right, top-to-bottom

		Additionally this function supports rendering rolling doors.

		This function should render each screen pixel exactly once.

		@param rollFunc function that for given square says its door roll in
					 RCL_Units (0 = no roll, RCL_UNITS_PER_SQUARE = full roll right,
					 -RCL_UNITS_PER_SQUARE = full roll left), can be zero (no rolling door,
					 rendering should also be faster as fewer intersections will be tested)
	*/
	fn RCL_renderSimple(&mut self, general:&mut RCL_General, cam:RCL_Camera, typeFunc:Option<RCL_ArrayFunction>) {
		self._RCL_camera = cam.clone();
		self._RCL_middleRow = halfResY as i16;

		self._RCL_cameraHeightScreen = (
			(CAMERA_RESOLUTION_Y as RCL_Unit * (self._RCL_camera.height - RCL_UNITS_PER_SQUARE))
			/
			RCL_UNITS_PER_SQUARE
		) as i16;

		self._RCL_horizontalDepthStep = RCL_HORIZON_DEPTH / CAMERA_RESOLUTION_Y as RCL_Unit;

		general.defaultConstraints.maxHits =
			if self._RCL_rollFunction.is_some() {
				3 // for correctly rendering rolling doors we'll need 3 hits (NOT 2)
			} else {
				1 // no door => 1 hit is enough
			};

		if RCL_COMPUTE_FLOOR_TEXCOORDS {
			// pass to column function
			self._RCL_floorPixelDistances = Some(self._RCL_precomputeFloorDistances(&cam, self._RCL_middleRow as u16));
		}

		self.RCL_castRaysMultiHit(general, &cam, Self::_floorHeightNotZeroFunction,typeFunc, Self::_RCL_columnFunctionSimple);

		if RCL_COMPUTE_FLOOR_TEXCOORDS {
			self._RCL_floorPixelDistances = None;
		}
	}

	// checks a single square for collision against the camera
	// #define
	fn collCheck(&mut self, dirCollides:&mut bool, s1:i16, s2:i16, computeHeight:bool, bottomLimit:RCL_Unit, topLimit:RCL_Unit) {
		if computeHeight {
			let height = floorHeightFunction(s1,s2);
			if height > bottomLimit {
				*dirCollides = true;
			} else if let Some(chf) = ceilingHeightFunc {
				let height = chf(s1, s2);
				if height < topLimit {
					*dirCollides = true;
				}
			}
		} else {
			*dirCollides = floorHeightFunction(s1, s2) > RCL_CAMERA_COLL_STEP_HEIGHT;
		}
	}

	// check a collision against non-diagonal square
	// #define
	#[inline]
	fn collCheckOrtho(&mut self,
		dirCollides:&mut bool, dirSquare:i16, dirSquareNew:i16, dir2Dir:i16, dir2Square:&mut i16, dir2Square2:&mut i16,
		s1:i16, s2:i16,
		x:bool,
		computeHeight:bool, bottomLimit:RCL_Unit, topLimit:RCL_Unit, corner_dir2:RCL_Unit
	) {
		if dirSquareNew != dirSquare {
			self.collCheck(dirCollides, s1, s2, computeHeight, bottomLimit, topLimit);
		}
		if !*dirCollides { // now also check for coll on the neighbouring square
			*dir2Square2 = RCL_divRoundDown(corner_dir2 - dir2Dir as RCL_Unit * RCL_CAMERA_COLL_RADIUS * 2,RCL_UNITS_PER_SQUARE) as i16;
		}
		if dir2Square2 != dir2Square {
			if x {
				self.collCheck(dirCollides, dirSquareNew, *dir2Square2, computeHeight, bottomLimit, topLimit);
			} else {
				self.collCheck(dirCollides, *dir2Square2, dirSquareNew, computeHeight, bottomLimit, topLimit);
			}
		}
	}

	/**
		Function that moves given camera and makes it collide with walls and
		potentially also floor and ceilings. It's meant to help implement player
		movement.

		@param camera camera to move
		@param planeOffset offset to move the camera in
		@param heightOffset height offset to move the camera in
		@param floorHeightFunc function used to retrieve the floor height
		@param ceilingHeightFunc function for retrieving ceiling height, can be 0
														 (camera won't collide with ceiling)
		@param computeHeight whether to compute height - if false (0), floor and
												 ceiling functions won't be used and the camera will
												 only collide horizontally with walls (good for simpler
												 game, also faster)
		@param force if true, forces to recompute collision even if position doesn't
								 change
	*/
	pub fn RCL_moveCameraWithCollision(&mut self,
																 camera:&mut RCL_Camera,
																 planeOffset:RCL_Vector2D, heightOffset:RCL_Unit,
																 computeHeight:bool, force:bool
	) {
		let movesInPlane = planeOffset.x != 0 || planeOffset.y != 0;
		let xSquareNew:i16;
		let ySquareNew:i16;

		if movesInPlane || force {
			let xDir:i16 = if planeOffset.x > 0 { 1 } else { -1 };
			let yDir:i16 = if planeOffset.y > 0 { 1 } else { -1 };

			// BBox corner in the movement direction
			let mut corner:RCL_Vector2D = RCL_Vector2D {
				x: camera.position.x + xDir as RCL_Unit * RCL_CAMERA_COLL_RADIUS,
				y: camera.position.y + yDir as RCL_Unit * RCL_CAMERA_COLL_RADIUS,
			};

			let mut xSquare:i16 = RCL_divRoundDown(corner.x,RCL_UNITS_PER_SQUARE) as i16;
			let mut ySquare:i16 = RCL_divRoundDown(corner.y,RCL_UNITS_PER_SQUARE) as i16;

			let mut cornerNew = RCL_Vector2D {
				x: corner.x + planeOffset.x,
				y: corner.y + planeOffset.y,
			};

			xSquareNew = RCL_divRoundDown(cornerNew.x,RCL_UNITS_PER_SQUARE) as i16;
			ySquareNew = RCL_divRoundDown(cornerNew.y,RCL_UNITS_PER_SQUARE) as i16;

			let bottomLimit;
			let topLimit;

			if computeHeight {
				bottomLimit = camera.height - RCL_CAMERA_COLL_HEIGHT_BELOW + RCL_CAMERA_COLL_STEP_HEIGHT;
				topLimit = camera.height + RCL_CAMERA_COLL_HEIGHT_ABOVE;
			} else {
				// TODO: personal, check linter. will it suggest to replace "-1 *" with "-"?
				bottomLimit = -1 * RCL_INFINITY;
				topLimit = RCL_INFINITY;
			}

			let mut xCollides = false;
			let mut ySquare2:i16 = 0;
			// xy
			let ys = ySquare;
			self.collCheckOrtho(&mut xCollides,xSquare,xSquareNew, yDir, &mut ySquare, &mut ySquare2,xSquareNew,ys,true, computeHeight, bottomLimit, topLimit, corner.y);

			let mut yCollides = false;
			let mut xSquare2:i16 = 0;
			// yx
			let xs = xSquare;
			self.collCheckOrtho(&mut yCollides,ySquare, ySquareNew, xDir, &mut xSquare, &mut xSquare2,xs,ySquareNew,false, computeHeight, bottomLimit, topLimit, corner.x);

			// #define
			fn collHandle(dirCollides:bool, dirSquare:i16, dirDir:i16, cornerNewDir:&mut RCL_Unit) {
				if dirCollides {
					*cornerNewDir =
						dirSquare as RCL_Unit * RCL_UNITS_PER_SQUARE +
						RCL_UNITS_PER_SQUARE / 2 +
						dirDir as RCL_Unit * (RCL_UNITS_PER_SQUARE / 2) -
						dirDir as RCL_Unit;
				}
			}

			if !xCollides && !yCollides { // if non-diagonal collision happend, corner collision can't happen
				if xSquare != xSquareNew && ySquare != ySquareNew { // corner?
					let mut xyCollides = false;
					self.collCheck(&mut xyCollides, xSquareNew, ySquareNew, computeHeight, bottomLimit, topLimit);

					if xyCollides {
						// normally should slide, but let's KISS
						cornerNew = corner;
					}
				}
			}

			collHandle(xCollides, xSquare, xDir, &mut cornerNew.x);
			collHandle(yCollides, ySquare, yDir, &mut cornerNew.y);

			// #undef collCheck
			// #undef collHandle

			camera.position.x = cornerNew.x - xDir as RCL_Unit * RCL_CAMERA_COLL_RADIUS;
			camera.position.y = cornerNew.y - yDir as RCL_Unit * RCL_CAMERA_COLL_RADIUS;
		}

		if computeHeight && (movesInPlane || heightOffset != 0 || force) {
			camera.height += heightOffset;

			let xSquare1 = RCL_divRoundDown(camera.position.x - RCL_CAMERA_COLL_RADIUS, RCL_UNITS_PER_SQUARE) as i16;
			let xSquare2 = RCL_divRoundDown(camera.position.x + RCL_CAMERA_COLL_RADIUS, RCL_UNITS_PER_SQUARE) as i16;
			let ySquare1 = RCL_divRoundDown(camera.position.y - RCL_CAMERA_COLL_RADIUS, RCL_UNITS_PER_SQUARE) as i16;
			let ySquare2 = RCL_divRoundDown(camera.position.y + RCL_CAMERA_COLL_RADIUS, RCL_UNITS_PER_SQUARE) as i16;

			let mut bottomLimit = floorHeightFunction(xSquare1, ySquare1);
			let mut topLimit =
				if let Some(chf) = ceilingHeightFunc {
					chf(xSquare1, ySquare1)
				} else {
					RCL_INFINITY
				}
			;

			let mut height:RCL_Unit = 0;

			// #define
			#[inline]
			fn checkSquares(xSquare:i16, ySquare:i16, height:&mut RCL_Unit, bottomLimit:&mut RCL_Unit, topLimit:&mut RCL_Unit) {
				*height = floorHeightFunction(xSquare, ySquare);
				*bottomLimit = *bottomLimit.max(height);
				*height =
					if let Some(chf) = ceilingHeightFunc {
						chf(xSquare, ySquare)
					} else {
						RCL_INFINITY
					}
				;
				*topLimit = *topLimit.min(height);
			}

			if xSquare2 != xSquare1 {
				checkSquares(xSquare2, ySquare1, &mut height, &mut bottomLimit, &mut topLimit);
			}

			if ySquare2 != ySquare1 {
				checkSquares(xSquare1, ySquare2, &mut height, &mut bottomLimit, &mut topLimit);
			}

			if xSquare2 != xSquare1 && ySquare2 != ySquare1 {
				checkSquares(xSquare2, ySquare2, &mut height, &mut bottomLimit, &mut topLimit);
			}

			camera.height = RCL_clamp(camera.height, bottomLimit + RCL_CAMERA_COLL_HEIGHT_BELOW, topLimit - RCL_CAMERA_COLL_HEIGHT_ABOVE);

			// #undef checkSquares
		}
	}
}