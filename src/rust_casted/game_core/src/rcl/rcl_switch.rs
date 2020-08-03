use crate::demo1::*;
use crate::rcl::*;

// RCL_PIXEL_FUNCTION
pub const FPS:u8 = 255;
pub const pixelFunc:RCL_PixelFunction = crate::demo1::pixelFunc;
pub const floorHeightFunction:RCL_ArrayFunction = floorHeightAt;
pub const ceilingHeightFunc:Option<RCL_ArrayFunction> = Some(ceilingHeightAt);