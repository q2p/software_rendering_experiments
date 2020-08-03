use crate::{SCREEN_WIDTH, SCREEN_HEIGHT, RGBA, SCREEN_SPACE};
use tiny_lib::{matrices::*, vector::*, util::*};
use crate::controls::*;
use core::f32::consts::PI;
use core::cmp::min;


// Inspired by https://www.shadertoy.com/view/ldX3Ws (inigo quilez - iq/2013)
// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.

// TODO: 
//    - Saturn rings
//    - asteroid belt
//    - shading
//    - textures
//    - shadows / eclipses
//    - skybox / milky way

const eps:f32 = 0.001;
const NUMSPH:usize = 9;

pub struct Shader1();

#[derive(Clone)]
struct Planet {
	r:f32, // r radius (km)
	d:f32, // d distance (km)
	c:Vec3, // c color (rgb)
	op:f32, // op orbital period (days)
	rp:f32, // rp rotation period (days)
	ga:f32, // ga geometric albedo (fraction)
}

const planets_v:[Planet;NUMSPH] = [
	Planet { // Sol
		r: 695700.0,
		d: 0.0,
		c: Vec3::new( 1.0, 0.5, 0.0 ),
		op: 0.0,
		rp: 0.0,
		ga: 1.0,
	},
	Planet { 
		// Mercury
		r: 2439.7,
		d: (69816.9+46001.2+57909.05)/3.0*1000.0,
		c: Vec3::new( 0.6, 0.6, 0.6 ),
		op: 87.9691,
		rp: 58.646,
		ga: 0.142,
	},
	Planet { // Venus
		r: 6051.8,
		d: (108939.0+107477.0+108208.0)/3.0*1000.0,
		c: Vec3::new(0.8, 0.76, 0.66 ),  // 204, 195, 168
		op: 224.701,
		rp: -243.025,
		ga: 0.67,
	},
	Planet { // Earth
		r: 6378.1,
		d: (152100.0+147095.0+149598.0)/3.0*1000.0,
		c: Vec3::new(0.3, 0.3, 0.5),
		op: 365.256363,
		rp: 0.99726968,
		ga: 0.367,
	},
	Planet { // Mars
		r: 3389.5,
		d: (249.2+206.7+227.9392)/3.0*1000.0*1000.0,
		c: Vec3::new(0.66, 0.44, 0.28), // 168, 112, 72
		op: 686.971,
		rp: 1.025957,
		ga: 0.170,
	},
	Planet { // Jupiter
		r: 69911.0,
		d: (816.04+740.55+778.299)/3.0*1000.0*1000.0,
		c: Vec3::new( 0.73, 0.68, 0.62 ), // 187, 173, 157
			op: 4332.59,
			rp: 9.925/24.0,
			ga: 0.52,
	},
	Planet { // Saturn
		r: 58232.0,
		d: (1509.0+1350.0+1429.39)/3.0*1000.0*1000.0,
		c: Vec3::new( 0.65, 0.58, 0.43 ), // 166, 149, 109
		op: 10759.22,
		rp: 10.55/24.0,
		ga: 0.47,
	},
	Planet { // Uranus
		r: 25362.0,
		d: (3008.0+2742.0+2875.04)/3.0*1000.0*1000.0,
		c: Vec3::new( 0.75, 0.88, 0.91 ), // 190, 228, 231
		op: 30688.5,
		rp: 0.71833,
		ga: 0.51,
	},
	Planet { // Moon
		r: 1737.1,
		d: (362600.0+405400.0+384399.0)/3.0,
		c: Vec3::new( 0.39, 0.38, 0.37 ), // 100, 97, 94
		op: 27.321661,
		rp: 27.321661,
		ga: 0.136,
	}
];

#[inline(always)]
fn getPlanet(i:u8) -> &'static Planet {
	unsafe { planets_v.get_unchecked(i as usize) }
}

// test if position is inside sphere boundary
#[inline(always)]
fn nSphere( pos:&Vec3, sph:&Vec4 ) -> Vec3 {
	Vec3 {
		x: (pos.x-sph.x)/sph.w,
		y: (pos.y-sph.y)/sph.w,
		z: (pos.z-sph.z)/sph.w,
	}
}

// ?
#[inline(always)]
fn iSphere(ro:Vec3, rd:Vec3, sph:&Vec4 ) -> f32 {
	let oc = ro - sph.xyz();
	let b = oc.dot(&rd);
	let c = oc.dot(&oc) - sph.w * sph.w;
	let h = b*b - c;
	if h < 0.0  {
		return -1.0;
	} else {
		return -b - h.sqrt();
	}
}

// ?
// #[inline(always)]
// fn sSphere( ro:&Vec3, rd:&Vec3, sph:&Vec4 ) -> f32 {
// 		let oc = ro - sph.xyz();
// 		let b = oc.dot( rd );
// 		let c = oc.dot( oc ) - sph.w*sph.w;
	
// 		return step( min( -b, min( c, b*b - c ) ), 0.0 );
// }

// return negative if nothing hit
fn intersect( ro:Vec3, rd:Vec3, nor:&mut Vec3, rad:&mut f32, id:&mut u8, sphere:&[Vec4;NUMSPH] ) -> f32 {
	let mut res = 1e20;
	let mut fou = -1.0;
	
	*nor = Vec3::ZERO;

	for i in 0..NUMSPH as u8 {
		let sph = &sphere[i as usize];
		let t = iSphere( ro, rd, sph ); 
		if t > eps && t < res {
			res = t;
			*nor = nSphere( &(ro + t * rd), sph );
			fou = 1.0;
			*rad = sphere[i as usize].w;
			*id = i;
		}
	}
							
	return fou * res;					  
}

fn mainImage( iTime:f32, fragCoord:Vec2, res:Vec2, sphere:&mut [Vec4;NUMSPH] ) -> Vec3 {
	let q:Vec2 = fragCoord;
	let mut p:Vec2 = Vec2::new(-1.0, -1.0) + 2.0 * q;
	p.x *= res.x/res.y;
	
	//-----------------------------------------------------
	// animate planets
	//-----------------------------------------------------
	let an = 0.3*iTime;

	// other planets
	for i in 0..NUMSPH as u8 {
		// get planet info
		let mut planet = getPlanet(i).clone();
				
		// rescale for illustrative purposes
		planet.d = (planet.d+1.0).powf(1.0/2.125).max(0.0) / 2.5;
		planet.r = (planet.r+1.0).powf(1.0/1.9) / 1.0;
				
		// find rotation from time elapsed and orbital period
		//float a = -an / pow(op+1.0, 1.0/2.0 ) * 50.0;
		let a = -an / planet.op * 600.0;

		// set animated position of planets
		sphere[i as usize] = Vec4::new( planet.d * a.cos(),  0.0,  planet.d * a.sin(),  planet.r );
	}
	// the sun
	//sphere[0].w = sphere[5].w;
	//sphere[0].w = log(sphere[0].w);
	sphere[0].x = 0.0;
	sphere[0].y = 0.0;
	sphere[0].z = 0.0;
	// moon
	let r = sphere[8].w;
	//sphere[8] /= 10.0;
	sphere[8] += sphere[3];
	sphere[8].w = r;
		
			
	//-----------------------------------------------------
	// camera
	//-----------------------------------------------------
	//Vec3(3.5*sin(an),1.5*cos(0.5*an)+22.2,2.5*cos(an));
	//Vec3 ro = Vec3(200.0, 200.0,100.0);
	//Vec3 ta = Vec3(0.0,0.0,-1000.0);

	//Vec3 ro1 = normalize( Vec3(0.1,1.0,0.75)) * sphere[0].w * 7.0;
	//Vec3 ta1 = Vec3(0.0,-1.0,-1.0) * length(sphere[7].xyz);
	//Vec3 ro2 = normalize( Vec3(1.0,0.0,0.01)) * sphere[0].w * 1.01;
	//Vec3 ta2 = Vec3(2.50,0.0,-5.0) * length(sphere[7].xyz);

	let ro1:Vec3 = ( Vec3::new(0.0, 1.0, 0.0) - sphere[7].xyz().normalize()).normalize() * sphere[7].xyz().len() * 1.0;
	let ta1:Vec3 = Vec3::ZERO * sphere[7].xyz().len();

	let ro2:Vec3 = ( Vec3::cross_product(&sphere[7].xyz(), &Vec3::new(0.0,1.0,0.0) )).normalize() * sphere[0].w * 1.0095;
	let ta2:Vec3 = sphere[7].xyz(); // Vec3(2.50,0.0,-5.0) * length(sphere[7].xyz);

	// set ray origin and target
	let ro:Vec3 = Vec3::lerp( ro1, ro2, 1.0-f32::smoothstep(-1.0,1.0,(an*0.3).cos()) );
	let ta:Vec3 = Vec3::lerp( ta1, ta2, 1.0-f32::smoothstep(-1.0,1.0,(an*0.3).cos()) );

	// calculate camera orientation
	let ww:Vec3 = ( ta - ro ).normalize();
	let uu:Vec3 = ( Vec3::cross_product(&ww,&Vec3::new(0.0,1.0,0.0) ) ).normalize();
	let vv:Vec3 = ( Vec3::cross_product(&uu,&ww)).normalize();
	let rd:Vec3 = ( p.x*uu + p.y*vv + 2.0*ww ).normalize();

	//-----------------------------------------------------
	// render
	//-----------------------------------------------------
		
	// background colour
	let mut col = Vec3::new(0.0, 0.0, 0.15);
		
	// vertical gradient
	// col *= 0.98+0.1*rd.y;
	col.z += 0.1+0.1*rd.y;
		
	// cast ray to find planets
	let mut nor = Vec3::ZERO;
	let mut rad = 0.5;
	let mut id = 0;
	let t = intersect(ro,rd,&mut nor, &mut rad, &mut id, sphere)* 0.0001;

	return Vec3::new(t, t, t);
	if t > 0.0 {
		// planet stats do-hickey
		col = getPlanet(id).c;
	}
	// vigneting
	// col = col * (1.0 - 0.45*Vec2::dot(&(q - Vec2::new(0.5, 0.5)),&(q - Vec2::new(0.5, 0.5))));

	return col;
}

impl Shader1 {
	pub fn render(&mut self, image:&mut [RGBA], tick_number:u32, controls:&Controls) {
		let iTime = tick_number as f32 / 100f32;
		let res = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

		let mut sphere = [Vec4::ZERO; NUMSPH];

		for y in 0..SCREEN_HEIGHT {
			for x in 0..SCREEN_WIDTH {
				let fragCoord = Vec2::new(x as f32 / SCREEN_WIDTH as f32, (SCREEN_HEIGHT - y) as f32 / SCREEN_HEIGHT as f32);

				let c = mainImage(iTime, fragCoord, res, &mut sphere);

				image[y as usize*SCREEN_WIDTH as usize + x as usize] = RGBA {
					r: (c.x * 255.0).clamp(0.0, 255.0).round() as u8,
					g: (c.y * 255.0).clamp(0.0, 255.0).round() as u8,
					b: (c.z * 255.0).clamp(0.0, 255.0).round() as u8,
					a: 255
				};
			}
		}
	}
}