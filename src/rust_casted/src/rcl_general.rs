/**
  General definitions common for Pokitto raycasting demos.

  The demos use mode 13: 1 byte per pixel = 256 colors. Bitmaps (textures,
  sprites, ...) are also in this format (use the provided python script to
  convert png images).

  author: Miloslav "drummyfish" Ciz
  license: CC0 1.0
*/

//#include "stdio.h" // for debugging raycastlibg

// Redefine camera vertical FOV: RCL_UNITS_PER_SQUARE would normally mean 360 degrees, but it's not an actual angle, just linear approximation, so this is okay.
const RCL_VERTICAL_FOV:RCL_Unit = RCL_UNITS_PER_SQUARE;

// This has to be defined to the name of the function that will render pixels.

// #include "raycastlib.h"
use crate::rcl::*;
// #include "Pokitto.h"
// Pokitto::Core pokitto;

const PLAYER_SPEED:RCL_Unit = (4 * RCL_UNITS_PER_SQUARE); // #ifndef PLAYER_SPEED

const PLAYER_ROTATION_SPEED:RCL_Unit = (RCL_UNITS_PER_SQUARE / 2); // #ifndef PLAYER_ROTATION_SPEED

const PLAYER_JUMP_SPEED:RCL_Unit = 500; // #ifndef PLAYER_JUMP_SPEED

const HEAD_BOB_HEIGHT:RCL_Unit = 100; // #ifndef HEAD_BOB_HEIGHT

const HEAD_BOB_STEP:u8 = 10; // #ifndef HEAD_BOB_STEP

const GRAVITY_ACCELERATION:RCL_Unit = ((3 * RCL_UNITS_PER_SQUARE) / 2); // #ifndef GRAVITY_ACCELERATION

const SCREEN_WIDTH:u8 = 110;
const SCREEN_HEIGHT:u8 = 88;
pub(crate) const MIDDLE_ROW:u8 = SCREEN_HEIGHT / 2;
const MIDDLE_COLUMN:u8 = SCREEN_WIDTH / 2;

const SUBSAMPLE:u8 = 2; // #ifndef SUBSAMPLE

const SUBSAMPLED_WIDTH:u8 = (SCREEN_WIDTH / SUBSAMPLE);

const TEXTURE_W:u8 = 32;
const TEXTURE_H:u8 = 32;

/// Transparent color for sprites and GUI.
const TRANSPARENT_COLOR:u8 = 0x8f;

/// Gives a middle color of given hue (0 to 15).
pub const fn HUE(c:u8) -> u8 { c * 16 + 8 }

#[inline]
pub fn putSubsampledPixel(screenbuffer: &mut [u8], x:u8, y:u8, color:u8) {
	let mut offset =
		y as usize * SCREEN_WIDTH as usize +
		x as usize * SUBSAMPLE    as usize
	;
	for i in 0..SUBSAMPLE {
		screenbuffer[offset + i as usize] = color;
	}
}

fn encodeHSV(hue:u8, saturation:u8, value:u8) -> u8 {
	if value > 15 {
		if saturation > 84 {
			// "normal" color, as 0bSHHHVVVV, VVVV != 0
			((saturation / 85 - 1) << 7) | ((hue / 32) << 4) | ((value - 16) / 15)
		} else {
			// saturation near 0 => gray, as 0bVVVV0000, VVVV != 0
			value / 16
		}
	} else {
		// value near 0 => black, as 0b00000000
		0
	}
}

fn decodeHSV(hsv:u8) -> (u8,u8,u8) {
  let topHalf    = hsv & 0b11110000;
  let bottomHalf = hsv & 0b00001111;

  if topHalf != 0 {
    // "normal" color
		let value = if bottomHalf != 15  { (bottomHalf + 1) * 16 } else { 255 };
    let saturation = (1 + ((hsv & 0b10000000) >> 7)) * 127;
    let hue = ((hsv & 0b01110000) >> 4) * 32;

		return (hue, saturation, value);
  } else {
    // gray/white/black
		return ( 0, 0, bottomHalf * 17 );
  }
}

// Personal: hue = 0..224; saturation = ..; value = ..
fn convertHSVtoRGB(hue:u8, saturation:u8, value:u8) -> (u8,u8,u8) {
	// adds precision
  const M:u8 = 16;

  let chroma:u16 = (value as u16 * saturation as u16) / 256;

	// 224 * 16 / 42 = 85
  let h = ((hue as u16 * M as u16) / 42) as u8;

  let a:u8 = ((h as i8 % (2 * M as i8)) - M as i8).abs();

  let x:i32 = (chroma as i16 * (M as i16 - a as i16)) / M as i16;

	let mut r = 0;
	let mut g = 0;
	let mut b = 0;

       if h <= 1 * M { r = chroma as u8;   g = x as u8;      b = 0;            }
  else if h <= 2 * M { r = x as u8;        g = chroma as u8; b = 0;            }
  else if h <= 3 * M { r = 0;              g = chroma as u8; b = x as u8;      }
  else if h <= 4 * M { r = 0;              g = x as u8;      b = chroma as u8; }
  else if h <= 5 * M { r = x as u8;        g = 0;            b = chroma as u8; }
  else if h <= 6 * M { r = chroma as u8;   g = 0;            b = x as u8;      }
  else               { r = 0;              g = 0;            b = 0;            }

  let m:i32 = value as i32 - chroma as i32;

	r = (r as i16 + m as i16) as u8;
	g = (g as i16 + m as i16) as u8;
	b = (b as i16 + m as i16) as u8;

	return (r, g, b);
}

/// Inits and loads a general 256 color palette.
fn initPalette() -> [[u8;3];256] {
	let mut palette:[[u8;3];256] = [[0;3];256]; // was unsigned short

  // the palette is HSV-based because it makes brightness addition fast, which is important for fog/shadow diminishing
	for i in 0..256 {
		let mut r = 0;
		let mut g = 0;
		let mut b = 0;

    let (h, s, v) = decodeHSV(i);
    let (r, g, b) = convertHSVtoRGB(h,s,v);
    palette[i as usize] = [r,g,b];
  }

	return palette;
}

/// Adds given intensity to a color.
#[inline]
pub fn addIntensity(color:u8, add:i8) -> u8 {
  let newValue:u8 = (color as i16 + add as i16) as u8;

	return if (newValue >> 4) == (color >> 4) {
		newValue
	} else {
		if add > 0 {
			color | 0x0f
		} else {
			0
		}
	}
}

#[inline]
fn plusIntensity(color:u8, plus:u8) -> u8 {
  let newValue = color + plus;
  return
		if (newValue >> 4) == (color >> 4) {
			newValue
		} else {
			color | 0x0f
		};
}

#[inline]
fn minusIntensity(color:u8, minus:u8) -> u8 {
  let newValue = color - minus;
  return
		if (newValue >> 4) == (color >> 4) {
			newValue
		} else {
			0
		};
}

/// Samples an image by normalized coordinates - each coordinate is in range 0 to RCL_UNITS_PER_SQUARE (from raycastlib).
#[inline]
pub fn sampleImage(image:&[u8], mut x:RCL_Unit, mut y:RCL_Unit) -> u8 {
  x = RCL_wrap(x,RCL_UNITS_PER_SQUARE);
  y = RCL_wrap(y,RCL_UNITS_PER_SQUARE);

  let index =
   (x / (RCL_UNITS_PER_SQUARE / TEXTURE_W as i32)) * TEXTURE_H as i32 +
   (y / (RCL_UNITS_PER_SQUARE / TEXTURE_W as i32));

  return image[2 + index as usize];
}

pub struct Screen {
	pallete:[[u8;3];256],
	data:[[u8; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
}
impl Screen {
	pub fn new() -> Screen {
		Screen {
			pallete: initPalette(),
			data: [[0; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize]
		}
	}

	#[inline]
	pub const fn drawPixel(&mut self, x:i16, y:i16, color:u8) {
		// TODO: personal: is check nescessary?
		if
			x >= 0 && x < SCREEN_WIDTH  as i16 &&
			y >= 0 && y < SCREEN_HEIGHT as i16
		{
			self.data[y as usize][x as usize] = color;
		}
	}

	pub fn project(&self, image:&mut [u8]) {
		for y in 0..SCREEN_HEIGHT {
			for x in 0..SCREEN_WIDTH {
				let c = self.pallete[self.data[y as usize][x as usize] as usize];
				let offset = (y*SCREEN_WIDTH + x)*4;
				image[offset as usize + 0] = c[0];
				image[offset as usize + 1] = c[1];
				image[offset as usize + 2] = c[2];
				image[offset as usize + 3] = 255;
			}
		}
	}
}

/// Faster than drawSprite.
fn drawImage(screen:&mut Screen, image:&[u8], x:u8, y:u8) {
  for i in 0..image[0] {
    let xPos = x + i;
    let column = 2 + i * image[1];

    for j in 0..image[1] {
      let c = image[column + j];

      if c != TRANSPARENT_COLOR {
        screen.drawPixel(xPos as i16, (y + j) as i16, image[(column + j) as usize]);
			}
    }
  }
}

/// General player class stuffed with everything the demos need. You'll probably want to write your own.
pub struct Player {
  pub mCamera:RCL_Camera,
	pub mVericalSpeed:RCL_Unit,

	// In order to detect whether player is standing on ground (for jumping) we need the derivative of vertical speed (not just the vertical speed) => we need two values.
	pub mVericalSpeedPrev:RCL_Unit,

  pub mRunning:bool,
	pub mHeadBob:RCL_Unit,
  pub mHeadBobUp:bool,
}
impl Player {
  pub fn new() -> Player {
		Player {
			mCamera: RCL_Camera {
				position: RCL_Vector2D::ZERO,
				direction: 0,
				resolution: RCL_Vector2D {
					x: SCREEN_WIDTH as RCL_Unit / SUBSAMPLE as RCL_Unit,
					y: SCREEN_HEIGHT as RCL_Unit,
				},
				shear: 0,
				height: RCL_UNITS_PER_SQUARE * 3,
			},
			mVericalSpeed: 0,
			mVericalSpeedPrev: 0,
			mRunning: false,
			mHeadBob: 0,
			mHeadBobUp: true,
		}
  }

  pub fn setPosition(&mut self, x:RCL_Unit, y:RCL_Unit) {
    self.mCamera.position.x = x;
		self.mCamera.position.y = y;
  }

  pub fn setPosition_dir(&mut self, x:RCL_Unit, y:RCL_Unit, z:RCL_Unit, direction:RCL_Unit) {
    self.mCamera.position.x = x;
		self.mCamera.position.y = y;
		self.mCamera.height = z;
		self.mCamera.direction = direction;
  }

  pub fn setPositionSquare(&mut self, squareX:i16, squareY:i16) {
    self.setPosition(
      squareX as RCL_Unit * RCL_UNITS_PER_SQUARE + RCL_UNITS_PER_SQUARE / 2,
      squareY as RCL_Unit * RCL_UNITS_PER_SQUARE + RCL_UNITS_PER_SQUARE / 2
		);
  }

  pub fn update(
		&mut self, renderer:&mut RCL_Renderer_Global,
		moveDirection:i16, strafe:bool, turnDirection:i16, jump:bool,
		shearDirection:i16, floorHeightFunction:RCL_ArrayFunction,
		ceilingHeightFunction:RCL_ArrayFunction, computeHeight:bool, dt:u32
	) {
    let mut moveOffset = RCL_Vector2D::ZERO;

    if moveDirection != 0 {
      let horizontalStep:RCL_Unit = (dt as RCL_Unit * PLAYER_SPEED * (if self.mRunning { 2 } else { 1 })) / 1000 * (if moveDirection > 0 { 1 } else { -1 });

      moveOffset = RCL_angleToDirection(self.mCamera.direction + (if strafe { RCL_UNITS_PER_SQUARE / 4 } else { 0 }));

      moveOffset.x = (moveOffset.x * horizontalStep) / RCL_UNITS_PER_SQUARE;
      moveOffset.y = (moveOffset.y * horizontalStep) / RCL_UNITS_PER_SQUARE;

      self.mHeadBob += if self.mHeadBobUp { HEAD_BOB_STEP as RCL_Unit } else { -HEAD_BOB_STEP as RCL_Unit };

      if self.mHeadBob > HEAD_BOB_HEIGHT {
				self.mHeadBobUp = false;
      } else if self.mHeadBob < -HEAD_BOB_HEIGHT {
				self.mHeadBobUp = true;
			}
    } else {
			self.mHeadBob /= 2;
		}

    if turnDirection != 0 {
      let rotationStep = (dt as RCL_Unit * PLAYER_ROTATION_SPEED) / 1000;
      self.mCamera.direction = RCL_wrap(self.mCamera.direction + turnDirection as RCL_Unit * rotationStep as RCL_Unit, RCL_UNITS_PER_SQUARE);
    }

    let prevHeight = self.mCamera.height;

    renderer.RCL_moveCameraWithCollision(&mut self.mCamera,moveOffset,self.mVericalSpeed, floorHeightFunction, ceilingHeightFunction, computeHeight, false);

    let heightDiff = self.mCamera.height - prevHeight;

    if heightDiff == 0 {
			self.mVericalSpeed = 0; // hit floor/ceiling
		}

    if jump && self.mVericalSpeed == 0 && self.mVericalSpeedPrev == 0 {
			self.mVericalSpeed = PLAYER_JUMP_SPEED; // jump
		}

    if shearDirection != 0 {
			self.mCamera.shear = RCL_clamp((self.mCamera.shear + shearDirection) as RCL_Unit * 10, -self.mCamera.resolution.y, self.mCamera.resolution.y) as i16;
    } else {
			self.mCamera.shear /= 2;
		}

		self.mVericalSpeedPrev = self.mVericalSpeed;

    if computeHeight {
			self.mVericalSpeed -= (dt as RCL_Unit * GRAVITY_ACCELERATION) / 1000; // gravity
		}
  }
}

/// Sprite class, again just bare minimum to fit the needs. Prefer writing your own.
pub struct Sprite {
  pub mImage:&'static[u8],
	pub mPosition:RCL_Vector2D,
	pub mHeight:RCL_Unit,
	pub mPixelSize:RCL_Unit,
}
impl Sprite {
  pub const fn new(image:&'static[u8], squareX:i16, squareY:i16, z:RCL_Unit, pixelSize:RCL_Unit) -> Sprite {
		Sprite {
			mImage: image,
			mPixelSize: pixelSize,
			mPosition:RCL_Vector2D {
				x: squareX * RCL_UNITS_PER_SQUARE + RCL_UNITS_PER_SQUARE / 2,
				y: squareY * RCL_UNITS_PER_SQUARE + RCL_UNITS_PER_SQUARE / 2,
			},
			mHeight: z * RCL_UNITS_PER_SQUARE + RCL_UNITS_PER_SQUARE / 2,
		}
  }

	pub fn Sprite() -> Sprite {
		Sprite {
			mImage: 0,
			mHeight: 0,
			mPixelSize: 1,
			mPosition: RCL_Vector2D::ZERO,
		}
  }
}

pub struct RCL_General {
	/// 1D z-buffer for visibility determination.
	zBuffer:[RCL_Unit; SUBSAMPLED_WIDTH as usize],
	pub defaultConstraints:RCL_RayConstraints,
}
impl RCL_General {
	pub fn new() -> RCL_General {
		RCL_General {
			defaultConstraints: RCL_RayConstraints { maxHits: 0, maxSteps: 0 },
		}
	}

	pub fn initGeneral(&mut self) {
		// pokitto.begin();
		// pokitto.setFrameRate(FPS);
		// pokitto.display.setFont(fontTiny);
		// pokitto.display.persistence = 1;
		// pokitto.display.setInvisibleColor(-1);

		self.defaultConstraints.init();

		for i in 0..SUBSAMPLED_WIDTH {
			self.zBuffer[i] = 0;
		}
	}

	/// Draws a scaled sprite on screen in an optimized way. The sprite has to be square in resolution for that.
	#[inline]
	pub fn drawSpriteSquare(&mut self, screen:&mut Screen, sprite:&[u8], mut x:i16, mut y:i16, depth:RCL_Unit, size:u16, intensity:i8) {
		if
			size < 0 || size > 200 || // let's not mess up with the incoming array
			sprite[0] != sprite[1]    // only draw square sprites
		{
			return;
		}

		// TODO: personal: is SCREEN_HEIGHT enough?
		let mut samplingIndices:[u16;SCREEN_HEIGHT as usize] = [0u16; SCREEN_HEIGHT as usize];

		// optimization: precompute the indices

		for i in 0..size {
			samplingIndices[i as usize] = (i * sprite[0] as u16) / size;
		}

		x -= (size / 2) as i16;
		y -= (size / 2) as i16;

		let c:u8;

		let jTo:i16 = size as i16 - core::cmp::max(0,y + size as i16 - 88 );
		let iTo:i16 = size as i16 - core::cmp::max(0,x + size as i16 - 110);

		let mut i = core::cmp::max(-x,0) as u16;
		while (i as i16) < iTo {
			let xPos = x + i as i16;

			if self.zBuffer[(xPos / SUBSAMPLE as i16) as usize] <= depth {
				continue;
			}

			let columnLocation:i16 = 2 + samplingIndices[i as usize] * sprite[0];

			let mut j = core::cmp::max(-y,0);
			while j < jTo {
				c = sprite[columnLocation as usize + samplingIndices[j as usize] as usize];

				if c != TRANSPARENT_COLOR {
					screen.drawPixel(xPos, y + j, addIntensity(c, intensity));
				}
				j += 1;
			}
			i += 1;
		}
	}
}

/// Computes an average color of given texture.
fn computeAverageColor(texture:&'static[u8], excludeColor:Option<i16>) -> u8 {
	let excludeColor = excludeColor.unwrap_or(-1);
  let mut sumH:u32 = 0;
  let mut sumS:u32 = 0;
  let mut sumV:u32 = 0;
  let pixels = texture[0] as u16 * texture[1] as u16;
  let mut count:u32 = 0;

  for i in 0..pixels {
    let color = texture[2 + i];

    if color == excludeColor {
      continue;
		}

		let (h,s,v) = decodeHSV(texture[2 + i]);

    sumH += h as u32;
    sumS += s as u32;
    sumV += v as u32;
    count += 1;
  }

  return encodeHSV((sumH / count) as u8, (sumS / count) as u8, (sumV / count) as u8);
}