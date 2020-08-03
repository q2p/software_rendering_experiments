const RCL_PROFILE:bool = false;

pub struct Profiler {
	calls:u32
}

impl Profiler {
	pub fn call(&mut self) {
		if RCL_PROFILE {
			self.calls.overflowing_add(1);
		}
	}
}

macro_rules! profile {
	( $( $p:ident ),+ ) => {
		$(
			pub static mut $p:Profiler = Profiler { calls: 0 };
		)+

		fn print_profile() {
			if RCL_PROFILE {
//				println!("profile fn calls:\n");
//				$(
//					println!("  {}: {}\n", stringify!($p), unsafe { &$p }.calls);
//				)+
			}
		}
  }
}

// function call counters for profiling
profile![
	RCL_sqrtInt,
	RCL_clamp,
	RCL_cosInt,
	RCL_angleToDirection,
	RCL_dist,
	RCL_len,
	RCL_pointIsLeftOfRay,
	RCL_castRayMultiHit,
	RCL_castRay,
	RCL_absVal,
	RCL_normalize,
	RCL_vectorsAngleCos,
	RCL_perspectiveScale,
	RCL_wrap,
	RCL_divRoundDown
];