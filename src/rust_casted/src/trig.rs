//use crate::rcl::*;

//pub fn cos_table1(a:f32) -> f32 {
//	let mut index = (a * FACTOR) as u16;
//	index %= TABLE_SIZE; // one instuction (mask)
//	index = if index >= 0 { // one instruction isel
//		index
//	} else {
//		TABLE_SIZE-index
//	};
//	return _SineDoubleTable[index as usize];
//}
//
//pub fn cos_table2(a:f32) -> f32 {
//	let mut index = (a * FACTOR) as u16;
//	index %= TABLE_SIZE; // one instuction (mask)
//	if index < 0 {
//		index += TABLE_SIZE;
//	}
//	return SineDoubleTable[index];
//}