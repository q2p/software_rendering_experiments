use crate::{ SCREEN_WIDTH, SCREEN_HEIGHT };
use crate::vector::{Vec3d, VecUVW, Vector };
use crate::controls::*;
use core::f32::consts::PI;

pub struct SoftwareRenderer {
	s_mesh:MeshBasic,
	s_texture:Texture,
	s_made_with:Texture,
	image_depth:[f32; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize],
}

impl SoftwareRenderer {
	#[inline(always)]
	pub const fn new() -> SoftwareRenderer {
		SoftwareRenderer {
			s_mesh: MeshBasic::new(),
			s_texture: Texture::zero_resolution_texture(),
			s_made_with: Texture::zero_resolution_texture(),
			image_depth: [0f32; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize]
		}
	}
	#[inline(always)]
	pub fn init(&mut self) {
		load_mesh(&mut self.s_mesh, MODEL_BIN);
		load_texture(&mut self.s_texture, TEXTURE_BIN);
		load_texture(&mut self.s_made_with, MADE_WITH_BIN);
	}
	#[inline(always)]

	fn rotation_matrix_z(a:f32) -> Matrix4x4 {
		let a_cos = a.cos();
		let a_sin = a.sin();

		Matrix4x4 {
			matrix: [
				[  a_cos,  a_sin,   0f32,   0f32 ],
				[ -a_sin,  a_cos,   0f32,   0f32 ],
				[   0f32,   0f32,   1f32,   0f32 ],
				[   0f32,   0f32,   0f32,   1f32 ],
			]
		}
	}
	fn rotation_matrix_x(a:f32) -> Matrix4x4 {
		let a_cos = a.cos();
		let a_sin = a.sin();

		Matrix4x4 {
			matrix: [
				[   1f32,   0f32,   0f32,   0f32 ],
				[   0f32,  a_cos,  a_sin,   0f32 ],
				[   0f32, -a_sin,  a_cos,   0f32 ],
				[   0f32,   0f32,   0f32,   1f32 ],
			]
		}
	}
	fn rotation_matrix_y(a:f32) -> Matrix4x4 {
		let a_cos = a.cos();
		let a_sin = a.sin();

		Matrix4x4 {
			matrix: [
				[  a_cos,   0f32, -a_sin,   0f32 ],
				[   0f32,   1f32,   0f32,   0f32 ],
				[  a_sin,   0f32,  a_cos,   0f32 ],
				[   0f32,   0f32,   0f32,   1f32 ],
			]
		}
	}
	pub fn render(&mut self, image:&mut [u8], tick_number:u32, controls:&Controls) {
		for i in 0..(SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4) {
			image[i] = 0;
		}
		for i in 0..(SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize) {
			self.image_depth[i] = 0f32;
		}

		// Projection Matrix
		let fov = 5f32;
		let plane_near = 0.5f32;
		let plane_far = 1000f32;
		let clip_plane_near = 0.01;
		// let aspect = SCREEN_HEIGHT as f32 / SCREEN_WIDTH as f32; // TODO: swap?
		// TODO: почему?
		let aspect = 1.5f32;

		let projection_matrix = projection_matrix(fov, aspect, plane_near, plane_far);

		/*	// TODO: projection matrix можно заменить просто на несколько hard coded комманд, без всяких операций с матрицей
			// TODO play around struct Matrix4x4 projection_matrix = { .matrix = {0} };
			let mut projection_matrix = Matrix4x4::zeros();
			// fill_matrix_4x4(&mut projection_matrix, 0f32);
			// TODO: fill matrix with 0s
			projection_matrix.matrix[0][0] = aspect * fFovRad;
			projection_matrix.matrix[1][1] =          fFovRad;
			projection_matrix.matrix[2][2] =   plane_far              / (plane_far - plane_near);
			projection_matrix.matrix[3][2] = (-plane_far * plane_near) / (plane_far - plane_near);
			projection_matrix.matrix[2][3] = 1.0f32;
			projection_matrix.matrix[3][3] = 0.0f32;*/

		// Illumination
		// let light_dir = normalize(&Vec3d::xyz1(1f32, 0.5f32, 0.25f32));

		// Create "Point At" Matrix for camera
		let camera = Vec3d::xyz1(0f32, 0f32, 0f32);
		// unsafe { (m_prec_x - (SCREEN_WIDTH/2) as f32) / 256f32 },
		// unsafe { (m_prec_y - (SCREEN_WIDTH/2) as f32) / 256f32 },
		let look_dir = Vec3d::xyz1(0f32 /* (theta*2f32).sin(), */, 0f32, 1f32);
		let up = Vec3d::xyz1(0f32, 1f32, 0f32);
		let target = camera + look_dir;
		let camera_matrix = point_at_matrix(&camera, &target, &up);

		for i in 0..1 {
			let theta = (tick_number as f32) * 0.01 + i as f32;

			let rotx =  tick_number as f32 * 0.006f32;
			let roty = (tick_number as f32 * 0.013f32).cos() * PI * 0.1f32 - PI * 0.2f32;

			let mut rot_x_m = Self::rotation_matrix_x(roty);
			let mut rot_y_m = Self::rotation_matrix_y(rotx);

			let mat_rot = mul_matrix(&rot_y_m, &rot_x_m);

			// TODO: заменить все повороты, развороты на единую world matrix
			let mut mat_world = Matrix4x4::ones_cascade(); // Form World Matrix
			mat_world = mul_matrix(&mat_world, &transition_matrix(0f32, -1.25f32, 0f32));
			mat_world = mul_matrix(&mat_world, &mat_rot);
			// TODO: без трансформации моделька рендерится на 7 юнитов ниже чем нужно
			mat_world = mul_matrix(&mat_world, &transition_matrix(0f32, 7.3f32, 10f32));

			// Make view matrix from camera
			let view_matrix = inverse_transformation_matrix(&camera_matrix);

			// Draw Triangles
			for triangle in &self.s_mesh.triangles[0usize..self.s_mesh.amount_of_triangles as usize] {
				// World Matrix Transform
				let mut translated_triangle = Triangle {
					vertices: [
						multiply_vector_matrix(&triangle.vertices[0], &mat_world),
						multiply_vector_matrix(&triangle.vertices[1], &mat_world),
						multiply_vector_matrix(&triangle.vertices[2], &mat_world),
					],
					uvs: triangle.uvs.clone(),
				};

				// Get lines either side of triangle
				// let line1 = sub_vec3d(&translated_triangle.vertices[1], &translated_triangle.vertices[0]);
				// let line2 = sub_vec3d(&translated_triangle.vertices[2], &translated_triangle.vertices[0]);

				// Calculate triangle Normal
				// Take cross product of lines to get normal to triangle surface
				// You normally need to normalise a normal!
				// let normal = normalize(&cross_product(&line1, &line2));

				// Get Ray from triangle to camera
				// let ray_towards_triangle = normalize(&sub_vec3d(&translated_triangle.vertices[0], &camera));

				// If ray is aligned with normal, then triangle is visible
				// if Vec3d::dot_product(&ray_towards_triangle, &normal) < 0f32 {
				if true {
					// if true {
					// How "aligned" are light direction and triangle surface normal?
					// let light = Vec3d::dot_product(&light_dir, &normal); // Todo math max
					// let color = ((light * 255f32).clamp(0f32, 255f32) + 0.5f32) as u8;

					// Convert World Space --> View Space
					let mut viewed_triangle = Triangle {
						vertices: [
							multiply_vector_matrix(&translated_triangle.vertices[0], &view_matrix),
							multiply_vector_matrix(&translated_triangle.vertices[1], &view_matrix),
							multiply_vector_matrix(&translated_triangle.vertices[2], &view_matrix),
						],
						uvs: translated_triangle.uvs.clone(),
					};

					// Clip Viewed Triangle against near plane, this could form two additional additional triangles.
					let (clipped_n, clipped_array) = triangle_clip_against_plane(
						&Vec3d::xyz1(0f32,0f32,clip_plane_near), // Clip Position w = 0 or 1?
						&Vec3d::xyz1(0f32,0f32,1.0f32), // Clip Normal w = 0 or 1?
						&viewed_triangle
					);

					// We may end up with multiple triangles form the clip, so project as required
					for clipped_triangle in &clipped_array[0..clipped_n] {
						// Project triangles from 3D --> 2D
						let mut projected_triangle = Triangle {
							vertices: [
								multiply_vector_matrix(&clipped_triangle.vertices[0], &projection_matrix),
								multiply_vector_matrix(&clipped_triangle.vertices[1], &projection_matrix),
								multiply_vector_matrix(&clipped_triangle.vertices[2], &projection_matrix),
							],
							uvs: clipped_triangle.uvs.clone(),
						};

						projected_triangle.uvs[0].u /= projected_triangle.vertices[0].w;
						projected_triangle.uvs[1].u /= projected_triangle.vertices[1].w;
						projected_triangle.uvs[2].u /= projected_triangle.vertices[2].w;

						projected_triangle.uvs[0].v /= projected_triangle.vertices[0].w;
						projected_triangle.uvs[1].v /= projected_triangle.vertices[1].w;
						projected_triangle.uvs[2].v /= projected_triangle.vertices[2].w;

						projected_triangle.uvs[0].w = 1f32 / projected_triangle.vertices[0].w;
						projected_triangle.uvs[1].w = 1f32 / projected_triangle.vertices[1].w;
						projected_triangle.uvs[2].w = 1f32 / projected_triangle.vertices[2].w;

						// Scale into view, we moved the normalising into cartesian space out of the matrix.vector function from the previous videos, so do this manually
						projected_triangle.vertices[0] = div_vec3d(&projected_triangle.vertices[0], projected_triangle.vertices[0].w);
						projected_triangle.vertices[1] = div_vec3d(&projected_triangle.vertices[1], projected_triangle.vertices[1].w);
						projected_triangle.vertices[2] = div_vec3d(&projected_triangle.vertices[2], projected_triangle.vertices[2].w);

						// X/Y are inverted so put them back
						projected_triangle.vertices[0].x *= -1f32; // TODO: vec invert func
						projected_triangle.vertices[1].x *= -1f32;
						projected_triangle.vertices[2].x *= -1f32;
						projected_triangle.vertices[0].y *= -1f32;
						projected_triangle.vertices[1].y *= -1f32;
						projected_triangle.vertices[2].y *= -1f32;

						// Move to center of screen
						projected_triangle.vertices[0].x += 1f32;
						projected_triangle.vertices[0].y += 1f32;
						projected_triangle.vertices[1].x += 1f32;
						projected_triangle.vertices[1].y += 1f32;
						projected_triangle.vertices[2].x += 1f32;
						projected_triangle.vertices[2].y += 1f32;

						// scale to screen size
						projected_triangle.vertices[0].x *= 0.5 * SCREEN_WIDTH as f32;
						projected_triangle.vertices[0].y *= 0.5 * SCREEN_HEIGHT as f32;
						projected_triangle.vertices[1].x *= 0.5 * SCREEN_WIDTH as f32;
						projected_triangle.vertices[1].y *= 0.5 * SCREEN_HEIGHT as f32;
						projected_triangle.vertices[2].x *= 0.5 * SCREEN_WIDTH as f32;
						projected_triangle.vertices[2].y *= 0.5 * SCREEN_HEIGHT as f32;

						// Clip triangles against all four screen edges, this could yield
						// a bunch of triangles, so create a queue that we traverse to
						//  ensure we only test new triangles generated against planes
						let (clipped_t, clipped_t_array) = triangle_clip_against_plane(
							&Vec3d::xyz1(0f32, 0f32, 0f32), // Clip Position
							&Vec3d::xyz1(0f32, 1f32, 0f32), // Clip Normal
							&projected_triangle
						);
						for t in &clipped_t_array[0..clipped_t] {
							let (clipped_t, clipped_t_array) = triangle_clip_against_plane(
								&Vec3d::xyz1(0f32, (SCREEN_HEIGHT - 1) as f32, 0f32), // Clip Position
								&Vec3d::xyz1(0f32, -1f32, 0f32), // Clip Normal
								&t
							);
							for t in &clipped_t_array[0..clipped_t] {
								let (clipped_t, clipped_t_array) = triangle_clip_against_plane(
									&Vec3d::xyz1(0f32, 0f32, 0f32), // Clip Position
									&Vec3d::xyz1(1f32, 0f32, 0f32), // Clip Normal
									&t
								);
								for t in &clipped_t_array[0..clipped_t] {
									let (clipped_t, clipped_t_array) = triangle_clip_against_plane(
										&Vec3d::xyz1(SCREEN_WIDTH as f32, 0f32, 0f32), // Clip Position
										&Vec3d::xyz1(-1f32, 0f32, 0f32), // Clip Normal
										&t
									);
									for t in &clipped_t_array[0..clipped_t] {
										unsafe {
											// // fill_triangle_safe(&mut image, &t, &RGB::rgb(126,127,126));
											texture_triangle_safe(image, &mut self.image_depth, &t, &self.s_texture);
											// draw_triangle_safe(&mut image, &t);
										}
									}
								}
							}
						}
					}
				}
			}

			Self::render_outline(image);

			self.render_overlay(image);
		}

		// Sort triangles from back to front
		/*sort(vecTrianglesToRaster.begin(), vecTrianglesToRaster.end(), [](triangle &t1, triangle &t2)
		{
			float z1 = (t1.p[0].z + t1.p[1].z + t1.p[2].z) / 3.0f;
			float z2 = (t2.p[0].z + t2.p[1].z + t2.p[2].z) / 3.0f;
			return z1 > z2;
		});*/
		// Loop through all transformed, viewed, projected, and sorted triangles
	}

	fn render_outline(image: &mut [u8]) {
		let w = SCREEN_WIDTH as usize;
		let h = SCREEN_HEIGHT as usize;
		for y in 1..h - 1 {
			let yt = w * (y - 1);
			let yc = w * (y);
			let yb = w * (y + 1);
			for x in 1..w - 1 {
				let xt = yt + x;
				let xx = yc + x;
				let xb = yb + x;
				let xl = xx - 1;
				let xr = xx + 1;
				unsafe {
					if
						*image.get_unchecked(xx * 4 + 3) == 0 && (
						*image.get_unchecked(xt * 4 + 3) == 0xFF ||
						*image.get_unchecked(xl * 4 + 3) == 0xFF ||
						*image.get_unchecked(xr * 4 + 3) == 0xFF ||
						*image.get_unchecked(xb * 4 + 3) == 0xFF)
					{
						*image.get_unchecked_mut(xx * 4 + 0) = 0xFE;
						*image.get_unchecked_mut(xx * 4 + 1) = 0xFE;
						*image.get_unchecked_mut(xx * 4 + 2) = 0xFE;
						*image.get_unchecked_mut(xx * 4 + 3) = 0xFE;
					}
				}
			}
		}
	}

	fn render_overlay(&mut self, image: &mut [u8]) -> () {
		for i in 0..(SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize) {
			let off = i * 4;
			unsafe {
				let a = *self.s_made_with.pixels.get_unchecked(off + 3);
				if a != 0 {
					*image.get_unchecked_mut(off + 0) = *self.s_made_with.pixels.get_unchecked(off + 0);
					*image.get_unchecked_mut(off + 1) = *self.s_made_with.pixels.get_unchecked(off + 1);
					*image.get_unchecked_mut(off + 2) = *self.s_made_with.pixels.get_unchecked(off + 2);
					*image.get_unchecked_mut(off + 3) = 0xFF;
				}
			}
		}
	}
}

fn mul_vec3d(vec:&Vec3d, multiplier:f32) -> Vec3d {
	return Vec3d::xyz1(vec.x * multiplier, vec.y * multiplier, vec.z * multiplier);
}

fn div_vec3d(vec:&Vec3d, divider:f32) -> Vec3d {
	return Vec3d::xyz1(vec.x / divider, vec.y / divider, vec.z / divider);
}

fn cross_product(vec1:&Vec3d, vec2:&Vec3d) -> Vec3d {
	return Vec3d {
		x: vec1.y * vec2.z - vec1.z * vec2.y,
		y: vec1.z * vec2.x - vec1.x * vec2.z,
		z: vec1.x * vec2.y - vec1.y * vec2.x,
		w: vec1.w * vec2.w - vec1.w * vec2.w, // TODO: 0 / 1 / this code?
	};
}

fn normalize2(vec:&Vec3d) -> Vec3d {
	let len = vec.len();
	return Vec3d {
		x: vec.x / len,
		y: vec.y / len,
		z: vec.z / len,
		w: vec.w, // TODO: делить на len?
	};
}

fn normalize(vec:&Vec3d) -> Vec3d {
	let rlen = vec.inv_len();
	return Vec3d {
		x: vec.x * rlen,
		y: vec.y * rlen,
		z: vec.z * rlen,
		w: vec.w, // TODO: делить на len?
	};
}

#[derive(Copy, Clone)]
struct Triangle {
	vertices:[Vec3d; 3],
	uvs:[VecUVW; 3],
}
impl Triangle {
	const fn zero_spaced_verticies() -> Triangle {
		return Triangle {
			vertices: [
				Vec3d::ZERO,
				Vec3d::ZERO,
				Vec3d::ZERO,
			],
			uvs: [
				VecUVW::ZERO,
				VecUVW::ZERO,
				VecUVW::ZERO,
			]
		};
	}
}

const MAX_TEXTURE_SIZE:usize = 1024;
struct Texture {
	pixels:[u8; MAX_TEXTURE_SIZE*MAX_TEXTURE_SIZE*4],
	resolution_u:u16,
	resolution_v:u16,
}
impl Texture {
	const fn zero_resolution_texture() -> Texture {
		return Texture {
			pixels: [0; MAX_TEXTURE_SIZE*MAX_TEXTURE_SIZE*4],
			resolution_u: 0,
			resolution_v: 0,
		};
	}
}

struct Matrix4x4 {
	matrix:[[f32;4];4],
}
impl Matrix4x4 {
	fn zeros() -> Matrix4x4 {
		return Matrix4x4 {
			matrix: [
				[0f32, 0f32, 0f32, 0f32],
				[0f32, 0f32, 0f32, 0f32],
				[0f32, 0f32, 0f32, 0f32],
				[0f32, 0f32, 0f32, 0f32]
			]
		};
	}
	fn ones_cascade() -> Matrix4x4 {
		return Matrix4x4 {
			matrix: [
				[1f32, 0f32, 0f32, 0f32],
				[0f32, 1f32, 0f32, 0f32],
				[0f32, 0f32, 1f32, 0f32],
				[0f32, 0f32, 0f32, 1f32]
			]
		};
	}
}

fn point_at_matrix(pos:&Vec3d, target:&Vec3d, up:&Vec3d) -> Matrix4x4 {
	// TODO: нужны ли все эти new_up, new_right, или их можно просчитать заранее?
	// TODO: need normalization?
	let new_forward = normalize(&(*target - *pos));

	let a = mul_vec3d(&new_forward, Vec3d::dot_product(up, &new_forward));
	let new_up = normalize(&(*up - a));

	let new_right = cross_product(&new_up, &new_forward);

	return Matrix4x4 {
		matrix: [
			[  new_right.x,   new_right.y,   new_right.z, 0f32],
			[     new_up.x,      new_up.y,      new_up.z, 0f32],
			[new_forward.x, new_forward.y, new_forward.z, 0f32],
			[        pos.x,         pos.y,         pos.z, 1f32]
		]
	};
}

fn inverse_transformation_matrix(matrix:&Matrix4x4) -> Matrix4x4 {
	return Matrix4x4 {
		matrix: [
			[matrix.matrix[0][0], matrix.matrix[1][0], matrix.matrix[2][0], 0f32],
			[matrix.matrix[0][1], matrix.matrix[1][1], matrix.matrix[2][1], 0f32],
			[matrix.matrix[0][2], matrix.matrix[1][2], matrix.matrix[2][2], 0f32],
			[
				-(matrix.matrix[3][0] * matrix.matrix[0][0] + matrix.matrix[3][1] * matrix.matrix[0][1] + matrix.matrix[3][2] * matrix.matrix[0][2]),
				-(matrix.matrix[3][0] * matrix.matrix[1][0] + matrix.matrix[3][1] * matrix.matrix[1][1] + matrix.matrix[3][2] * matrix.matrix[1][2]),
				-(matrix.matrix[3][0] * matrix.matrix[2][0] + matrix.matrix[3][1] * matrix.matrix[2][1] + matrix.matrix[3][2] * matrix.matrix[2][2]),
				1f32
			]
		]
	};
}

fn multiply_vector_matrix(inp:&Vec3d, matrix:&Matrix4x4) -> Vec3d {
	let mut out = Vec3d {
		x: inp.x * matrix.matrix[0][0] + inp.y * matrix.matrix[1][0] + inp.z * matrix.matrix[2][0] + matrix.matrix[3][0],
		y: inp.x * matrix.matrix[0][1] + inp.y * matrix.matrix[1][1] + inp.z * matrix.matrix[2][1] + matrix.matrix[3][1],
		z: inp.x * matrix.matrix[0][2] + inp.y * matrix.matrix[1][2] + inp.z * matrix.matrix[2][2] + matrix.matrix[3][2],
		w: inp.x * matrix.matrix[0][3] + inp.y * matrix.matrix[1][3] + inp.z * matrix.matrix[2][3] + matrix.matrix[3][3],
	};

	let w = inp.x * matrix.matrix[0][3] + inp.y * matrix.matrix[1][3] + inp.z * matrix.matrix[2][3] + matrix.matrix[3][3];

	if w != 0f32 { // TODO: когда w == 0 ?
		out.x /= w;
		out.y /= w;
		out.z /= w;
	}

	return out;
}

const MAX_BASIC_MESH_TRIANGLES:usize = 2048;
struct MeshBasic {
	triangles:[Triangle; MAX_BASIC_MESH_TRIANGLES],
	amount_of_triangles:u16,
}
impl MeshBasic {
	const fn new() -> MeshBasic {
		return MeshBasic {
			amount_of_triangles: 0,
			triangles: [Triangle::zero_spaced_verticies(); MAX_BASIC_MESH_TRIANGLES]
		}
	}
}

fn u8_to_u16(u1:u8, u2:u8) -> u16 {
	((u1 as u16) << 8) |
		((u2 as u16) << 0)
}

fn u8_to_u32(u1:u8, u2:u8, u3:u8, u4:u8) -> u32 {
	((u1 as u32) << 24) |
		((u2 as u32) << 16) |
		((u3 as u32) <<  8) |
		((u4 as u32) <<  0)
}

fn u8_to_f32(u1:u8, u2:u8, u3:u8, u4:u8) -> f32 {
	f32::from_bits(u8_to_u32(u1, u2, u3, u4))
}

fn load_mesh(mesh:&mut MeshBasic, binary:&[u8]) {
	let         verticies_amount = u8_to_u16(binary[0], binary[1]);
	let texture_verticies_amount = u8_to_u16(binary[2], binary[3]);
	mesh.amount_of_triangles = u8_to_u16(binary[4], binary[5]);

	let mut         verticies = [Vec3d::ZERO; 4*1024]; // TODO: dynamic buffer?
	let mut texture_verticies = [VecUVW::ZERO; 4*1024]; // TODO: dynamic buffer?

	let mut offset = 6;
	for i in 0usize..verticies_amount as usize {
		verticies[i].x = u8_to_f32(binary[offset + 0], binary[offset + 1], binary[offset + 2], binary[offset + 3]);
		offset += 4;
		verticies[i].y = u8_to_f32(binary[offset + 0], binary[offset + 1], binary[offset + 2], binary[offset + 3]);
		offset += 4;
		verticies[i].z = u8_to_f32(binary[offset + 0], binary[offset + 1], binary[offset + 2], binary[offset + 3]);
		offset += 4;
	}
	for i in 0usize..texture_verticies_amount as usize {
		texture_verticies[i].u = u8_to_f32(binary[offset + 0], binary[offset + 1], binary[offset + 2], binary[offset + 3]);
		offset += 4;
		texture_verticies[i].v = u8_to_f32(binary[offset + 0], binary[offset + 1], binary[offset + 2], binary[offset + 3]);
		offset += 4;
	}
	for i in 0usize..mesh.amount_of_triangles as usize {
		mesh.triangles[i].vertices[0] =         verticies[u8_to_u16(binary[offset + 0], binary[offset + 1]) as usize];
		offset += 2;
		mesh.triangles[i].vertices[1] =         verticies[u8_to_u16(binary[offset + 0], binary[offset + 1]) as usize];
		offset += 2;
		mesh.triangles[i].vertices[2] =         verticies[u8_to_u16(binary[offset + 0], binary[offset + 1]) as usize];
		offset += 2;
		mesh.triangles[i].   uvs[0] = texture_verticies[u8_to_u16(binary[offset + 0], binary[offset + 1]) as usize];
		offset += 2;
		mesh.triangles[i].   uvs[1] = texture_verticies[u8_to_u16(binary[offset + 0], binary[offset + 1]) as usize];
		offset += 2;
		mesh.triangles[i].   uvs[2] = texture_verticies[u8_to_u16(binary[offset + 0], binary[offset + 1]) as usize];
		offset += 2;
	}
}

fn load_texture(texture:&mut Texture, binary:&[u8]) {
	texture.resolution_u = u8_to_u16(binary[0], binary[1]);
	texture.resolution_v = u8_to_u16(binary[2], binary[3]);
	texture.pixels[0..binary.len()-4].copy_from_slice(&binary[4..binary.len()]);
}

//void draw_line_unsafe(u8 *screen, s16 x0, s16 y0, s16 x1, s16 y1) {
//	s32 dx = (s32)abs_s16(x1-x0), sx = x0<x1 ? 1 : -1;
//	s32 dy = (s32)abs_s16(y1-y0), sy = y0<y1 ? 1 : -1;
//	s32 err = (dx>dy ? dx : -dy)/2, e2;
//
// 	while(1) {
//		u32 offset = ((u32)y0*(u32)SCREEN_HEIGHT+(u32)x0)*4;
//		screen[offset+0] = (u8)0xFF;
//		screen[offset+1] = (u8)0xFF;
//		screen[offset+2] = (u8)0xFF;
//		screen[offset+4] = 0;
//		if (x0==x1 && y0==y1) break;
//		e2 = err;
//		if (e2 >-dx) { err -= dy; x0 += sx; }
//		if (e2 < dy) { err += dx; y0 += sy; }
//	}
//}
//
fn draw_line_safe(screen:&mut [u8], mut x0:i32, mut y0:i32, x1:i32, y1:i32) {
	let dx = (x1-x0).abs();
	let dy = (y1-y0).abs();
	let sx = if x0 < x1 { 1 } else { -1 };
	let sy = if y0 < y1 { 1 } else { -1 };
	let mut err = (if dx>dy { dx } else { -dy })/2;
	let mut e2;

	loop {
		if
		x0 >= 0 && x0 < SCREEN_WIDTH  as i32 &&
			y0 >= 0 && y0 < SCREEN_HEIGHT as i32
		{
			let offset = (y0 as usize * SCREEN_WIDTH as usize + x0 as usize)*4;
			screen[offset+0] = 0xFFu8;
			screen[offset+1] = 0xFFu8;
			screen[offset+2] = 0xFFu8;
			screen[offset+3] = 0xFFu8;
		}

		if x0 == x1 && y0 == y1 {
			break;
		}

		e2 = err;
		if e2 > -dx { err -= dy; x0 += sx; }
		if e2 <  dy { err += dx; y0 += sy; }
	}
}

const MODEL_BIN:&[u8] = include_bytes!("../res/model.rust3d");
const TEXTURE_BIN:&[u8] = include_bytes!("../res/texture.rust2d");
const MADE_WITH_BIN:&[u8] = include_bytes!("../res/made_with.rust2d");

fn draw_triangle_safe(screen:&mut [u8], triangle:&Triangle) {
	draw_line_safe(screen, triangle.vertices[0].x as i32, triangle.vertices[0].y as i32, triangle.vertices[1].x as i32, triangle.vertices[1].y as i32);
	draw_line_safe(screen, triangle.vertices[0].x as i32, triangle.vertices[0].y as i32, triangle.vertices[2].x as i32, triangle.vertices[2].y as i32);
	draw_line_safe(screen, triangle.vertices[1].x as i32, triangle.vertices[1].y as i32, triangle.vertices[2].x as i32, triangle.vertices[2].y as i32);
}

fn fill_triangle_safe(screen:&mut [u8], triangle:&Triangle, color:&RGB) {
	// TODO: другие алгоритмы
	unsafe {
		fill_triangle_1(
			screen,
			(triangle.vertices[0].x as i16).clamp(0, SCREEN_WIDTH as i16 - 1), (triangle.vertices[0].y as i16).clamp(0, SCREEN_HEIGHT as i16 - 1),
			(triangle.vertices[1].x as i16).clamp(0, SCREEN_WIDTH as i16 - 1), (triangle.vertices[1].y as i16).clamp(0, SCREEN_HEIGHT as i16 - 1),
			(triangle.vertices[2].x as i16).clamp(0, SCREEN_WIDTH as i16 - 1), (triangle.vertices[2].y as i16).clamp(0, SCREEN_HEIGHT as i16 - 1),
			color
		);
	}
}

fn texture_triangle_safe(screen:&mut [u8], depth_buffer:&mut [f32], triangle:&Triangle, texture:&Texture) {
	// TODO: другие алгоритмы
	// texture_triangle_1(
	texture_triangle_unsafe(
		screen,
		depth_buffer,
		(triangle.vertices[0].x.round() as i16).clamp(0, SCREEN_WIDTH as i16), (triangle.vertices[0].y.round() as i16).clamp(0, SCREEN_HEIGHT as i16), triangle.uvs[0].u, triangle.uvs[0].v, triangle.uvs[0].w, // TODO: rounding / not rounding macro
		(triangle.vertices[1].x.round() as i16).clamp(0, SCREEN_WIDTH as i16), (triangle.vertices[1].y.round() as i16).clamp(0, SCREEN_HEIGHT as i16), triangle.uvs[1].u, triangle.uvs[1].v, triangle.uvs[1].w,
		(triangle.vertices[2].x.round() as i16).clamp(0, SCREEN_WIDTH as i16), (triangle.vertices[2].y.round() as i16).clamp(0, SCREEN_HEIGHT as i16), triangle.uvs[2].u, triangle.uvs[2].v, triangle.uvs[2].w,
		texture,
	);
}

#[derive(Copy, Clone)]
struct RGB {
	r:u8,g:u8,b:u8
}
impl RGB {
	const fn black() -> RGB {
		return RGB { r: 0, g:0, b:0 };
	}
	const fn rgb(r:u8, g:u8, b:u8) -> RGB {
		return RGB { r, g, b };
	}
}

// Triangle Renderer 1
unsafe fn draw_line(screen:&mut [u8], y:u16, x0:u16, x1:u16, color:&RGB) {
	let offset = (y as usize * SCREEN_WIDTH as usize)*4;
	for i in x0..x1 {
		*screen.get_unchecked_mut(offset+i as usize*4+0) = color.r;
		*screen.get_unchecked_mut(offset+i as usize*4+1) = color.g;
		*screen.get_unchecked_mut(offset+i as usize*4+2) = color.b;
		*screen.get_unchecked_mut(offset+i as usize*4+3) = 0xFFu8;
	}
}
unsafe fn fill_bottom_flat_triangle(screen:&mut [u8], ty:i16, by:i16, lx:i16, rx:i16, tx:i16, color:&RGB) {
	debug_assert!(ty <= by);
	debug_assert!(lx <= tx);
	debug_assert!(tx <= rx);
	let invslope1 = (lx - tx) as f32 / (by - ty) as f32;
	let invslope2 = (rx - tx) as f32 / (by - ty) as f32;

	let mut curx1 = tx as f32;
	let mut curx2 = tx as f32;

	for scan_line_y in ty..=by {
		let c1;
		let c2;
		if curx1 > curx2 {
			c1 = curx2;
			c2 = curx1;
		} else {
			c1 = curx1;
			c2 = curx2;
		}
		let x1 = (c1 as i16    ).clamp(0, SCREEN_WIDTH as i16) as u16;
		let x2 = (c2 as i16 + 1).clamp(0, SCREEN_WIDTH as i16) as u16;
		draw_line(screen, scan_line_y as u16, x1, x2, color);
		curx1 += invslope1;
		curx2 += invslope2;
	}
}
unsafe fn fill_top_flat_triangle(screen:&mut [u8], mut ty:i16, mut by:i16, mut lx:i16, mut rx:i16, mut bx:i16, color:&RGB) {
	debug_assert!(ty <= by);
	debug_assert!(lx <= bx);
	debug_assert!(bx <= rx);
	let invslope1 = (bx - lx) as f32 / (by - ty) as f32;
	let invslope2 = (bx - rx) as f32 / (by - ty) as f32;

	let mut curx1= bx as f32;
	let mut curx2= bx as f32;

	let mut scan_line_y = by;
	while scan_line_y >= ty { // scan_line_y > ty тоже пойдёт
		// if scan_line_y >= 0 && scan_line_y < SCREEN_HEIGHT as i16 {
		let c1;
		let c2;
		if curx1 > curx2 {
			c1 = curx2;
			c2 = curx1;
		} else {
			c1 = curx1;
			c2 = curx2;
		}
		let x1 = (c1 as i16    ).clamp(0, SCREEN_WIDTH as i16) as u16;
		let x2 = (c2 as i16 + 1).clamp(0, SCREEN_WIDTH as i16) as u16;
		draw_line(screen, scan_line_y as u16, x1, x2, color);
		// }
		curx1 -= invslope1;
		curx2 -= invslope2;
		scan_line_y -= 1;
	}
}

unsafe fn fill_triangle_1(screen:&mut [u8], mut x1:i16, mut y1:i16, mut x2:i16, mut y2:i16, mut x3:i16, mut y3:i16, color:&RGB) {
	// at first sort the three vertices by y-coordinate ascending so v1 is the topmost vertice

	if y2 < y1 {
		core::mem::swap(&mut x1, &mut x2);
		core::mem::swap(&mut y1, &mut y2);
	}

	if y3 < y1 {
		core::mem::swap(&mut x1, &mut x3);
		core::mem::swap(&mut y1, &mut y3);
	}

	if y3 < y2 {
		core::mem::swap(&mut x2, &mut x3);
		core::mem::swap(&mut y2, &mut y3);
	}

	// here we know that v1.y <= v2.y <= v3.y
	if y2 == y3 { // trivial case of bottom-flat triangle
		fill_bottom_flat_triangle(screen, y1, y2, x2, x3, x1, color);
	} else if y1 == y2 { // trivial case of top-flat triangle
		fill_top_flat_triangle(screen, y1, y3, x1, x2, x3, color);
	} else { // general case - split the triangle in a topflat and bottom-flat one
		let portion = (y2 - y1) as f32 / (y3 - y1) as f32;

		let xdiff = x3-x1;
		let x4 = x1 + (portion * xdiff as f32) as i16;
		fill_bottom_flat_triangle(screen, y1, y2, x2, x4, x1, color);
		fill_top_flat_triangle(screen, y2, y3, x2, x4, x3, color);
	}
}

/*fn texture_triangle_1(
	screen:&mut [u8],
	mut x1:i16, mut y1:i16, mut u1: f32, mut v1:f32,
	mut x2:i16, mut y2:i16, mut u2: f32, mut v2:f32,
	mut x3:i16, mut y3:i16, mut u3: f32, mut v3:f32,
	texture:&Texture
) {
	// at first sort the three vertices by y-coordinate ascending so v1 is the topmost vertice
	// TODO: optimize
	/*
	// naive
	if y1 <= y2 && y2 <= y3 { // 123
		// Nothing
	} else if y1 <= y3 && y3 <= y2 { // 132
		core::mem::swap(&mut x2, &mut x3);
		core::mem::swap(&mut y2, &mut y3);
	} else if y2 <= y1 && y1 <= y3 { // 213
		core::mem::swap(&mut x1, &mut x2);
		core::mem::swap(&mut y1, &mut y2);
	} else if y2 <= y3 && y3 <= y1 { // 231
		let lx = x2;
		let ly = y2;

		x2 = x3;
		y2 = y3;

		x3 = x1;
		y3 = y1;

		x1 = lx;
		y1 = ly;
	} else if y3 <= y1 && y1 <= y2 { // 312
		let lx = x3;
		let ly = y3;

		x3 = x2;
		y3 = y2;

		x2 = x1;
		y2 = y1;

		x1 = lx;
		y1 = ly;
	} else if y3 <= y2 && y2 <= y1 { // 321
		core::mem::swap(&mut x1, &mut x3);
		core::mem::swap(&mut y1, &mut y3);
	}
	*/

	if y2 < y1 {
		core::mem::swap(&mut x1, &mut x2);
		core::mem::swap(&mut y1, &mut y2);
		core::mem::swap(&mut u1, &mut u2);
		core::mem::swap(&mut v1, &mut v2);
	}

	if y3 < y1 {
		core::mem::swap(&mut x1, &mut x3);
		core::mem::swap(&mut y1, &mut y3);
		core::mem::swap(&mut u1, &mut u3);
		core::mem::swap(&mut v1, &mut v3);
	}

	if y3 < y2 {
		core::mem::swap(&mut x2, &mut x3);
		core::mem::swap(&mut y2, &mut y3);
		core::mem::swap(&mut u2, &mut u3);
		core::mem::swap(&mut v2, &mut v3);
	}

	let dx1 = x2-x1;
	let dy1 = y2-y1;
	let du1 = u2-u1;
	let dv1 = v2-v1;

	let dx2 = x3-x1;
	let dy2 = y3-y1;
	let du2 = u3-u1;
	let dv2 = v3-v1;

	let mut dax_step = 0f32;
	let mut dbx_step = 0f32;
	let mut du1_step = 0f32; let mut dv1_step = 0f32;
	let mut du2_step = 0f32; let mut dv2_step = 0f32;

	if dy1 != 0 { dax_step = dx1 as f32 / dy1.abs() as f32; }
	if dy2 != 0 { dbx_step = dx2 as f32 / dy2.abs() as f32; }

	if dy1 != 0 { du1_step = du1 as f32 / dy1.abs() as f32; }
	if dy1 != 0 { dv1_step = dv1 as f32 / dy1.abs() as f32; }

	if dy2 != 0 { du2_step = du2 as f32 / dy2.abs() as f32; }
	if dy2 != 0 { dv2_step = dv2 as f32 / dy2.abs() as f32; }

	if dy1 != 0 {
		for i in y1 as isize ..=y2 as isize  {
			let mut ax = x1 as isize + ((i - y1 as isize) as f32 * dax_step) as isize;
			let mut bx = x1 as isize + ((i - y1 as isize) as f32 * dbx_step) as isize;

			let mut tex_su = u1 + (i - y1 as isize) as f32 * du1_step;
			let mut tex_sv = v1 + (i - y1 as isize) as f32 * dv1_step;

			let mut tex_eu = u1 + (i - y1 as isize) as f32 * du2_step;
			let mut tex_ev = v1 + (i - y1 as isize) as f32 * dv2_step;

			if ax > bx {
				core::mem::swap(&mut ax, &mut bx);
				core::mem::swap(&mut tex_su, &mut tex_eu);
				core::mem::swap(&mut tex_sv, &mut tex_ev);
			}

			let mut tex_u = tex_su;
			let mut tex_v = tex_sv;

			let t_step = 1f32 / (bx - ax) as f32;
			let mut t = 0f32;

			for j in ax..bx {
				let tex_u = (((1f32 - t) * tex_su + t * tex_eu) as i16).clamp(0, texture.resolution_u as i16 - 1) as usize;
				let tex_v = (((1f32 - t) * tex_sv + t * tex_ev) as i16).clamp(0, texture.resolution_v as i16 - 1) as usize;

				screen[4*(i as usize * SCREEN_WIDTH as usize + j as usize) + 0] = texture.pixels[4*(tex_v * texture.resolution_u as usize + tex_u) + 0];
				screen[4*(i as usize * SCREEN_WIDTH as usize + j as usize) + 1] = texture.pixels[4*(tex_v * texture.resolution_u as usize + tex_u) + 1];
				screen[4*(i as usize * SCREEN_WIDTH as usize + j as usize) + 2] = texture.pixels[4*(tex_v * texture.resolution_u as usize + tex_u) + 2];
				screen[4*(i as usize * SCREEN_WIDTH as usize + j as usize) + 3] = texture.pixels[4*(tex_v * texture.resolution_u as usize + tex_u) + 3];
				// x = j
				// y = i

				t += t_step;
			}
		}

		let dx1 = x3 - x2;
		let dy1 = y3 - y2;
		let du1 = u3 - u2;
		let dv1 = v3 - v2;

		if dy1 != 0 { dax_step = dx1 as f32 / dy1.abs() as f32; }
		if dy2 != 0 { dbx_step = dx2 as f32 / dy2.abs() as f32; }

		du1_step = 0f32;
		dv1_step = 0f32;

		if dy1 != 0 { du1_step = du1 as f32 / dy1.abs() as f32; }
		if dy1 != 0 { dv1_step = dv1 as f32 / dy1.abs() as f32; }

		for i in y2 as isize ..=y3 as isize {
			let mut ax = x2 as isize + ((i - y2 as isize) as f32 * dax_step) as isize;
			let mut bx = x1 as isize + ((i - y1 as isize) as f32 * dbx_step) as isize;

			let mut tex_su = u2 + (i - y2 as isize) as f32 * du1_step;
			let mut tex_sv = v2 + (i - y2 as isize) as f32 * dv1_step;

			let mut tex_eu = u1 + (i - y1 as isize) as f32 * du2_step;
			let mut tex_ev = v1 + (i - y1 as isize) as f32 * dv2_step;

			if ax > bx {
				core::mem::swap(&mut ax, &mut bx);
				core::mem::swap(&mut tex_su, &mut tex_eu);
				core::mem::swap(&mut tex_sv, &mut tex_ev);
			}

			let mut tex_u = tex_su;
			let mut tex_v = tex_sv;

			let t_step = 1f32 / (bx - ax) as f32;
			let mut t = 0f32;

			for j in ax..bx {
				let tex_u = (((1f32 - t) * tex_su + t * tex_eu) as i16).clamp(0, texture.resolution_u as i16 - 1) as usize;
				let tex_v = (((1f32 - t) * tex_sv + t * tex_ev) as i16).clamp(0, texture.resolution_v as i16 - 1) as usize;

				screen[4 * (i as usize * SCREEN_WIDTH as usize + j as usize) + 0] = texture.pixels[4 * (tex_v * texture.resolution_u as usize + tex_u) + 0];
				screen[4 * (i as usize * SCREEN_WIDTH as usize + j as usize) + 1] = texture.pixels[4 * (tex_v * texture.resolution_u as usize + tex_u) + 1];
				screen[4 * (i as usize * SCREEN_WIDTH as usize + j as usize) + 2] = texture.pixels[4 * (tex_v * texture.resolution_u as usize + tex_u) + 2];
				screen[4 * (i as usize * SCREEN_WIDTH as usize + j as usize) + 3] = texture.pixels[4 * (tex_v * texture.resolution_u as usize + tex_u) + 3];
				// x = j
				// y = i

				t += t_step;
			}
		}
	}

//	// here we know that v1.y <= v2.y <= v3.y
//	if y2 == y3 { // trivial case of bottom-flat triangle
//		fill_bottom_flat_triangle(screen, y1, y2, x2, x3, x1, color);
//	} else if y1 == y2 { // trivial case of top-flat triangle
//		fill_top_flat_triangle(screen, y1, y3, x1, x2, x3, color);
//	} else { // general case - split the triangle in a topflat and bottom-flat one
//		let portion = (y2 - y1) as f32 / (y3 - y1) as f32;
//
//		let xdiff = x3-x1;
//		let x4 = x1 + (portion * xdiff as f32) as i16;
//		fill_bottom_flat_triangle(screen, y1, y2, x2, x4, x1, color);
//		fill_top_flat_triangle(screen, y2, y3, x2, x4, x3, color);
//	}
}*/

#[inline(always)]
fn Draw(
	screen:&mut [u8],
	texture:&Texture,
	x:usize, y:usize, u:f32, v:f32,
) {
	screen[4 * (y * SCREEN_WIDTH as usize + x) + 0] = texture.pixels[4 * ((v * texture.resolution_v as f32) as usize * texture.resolution_u as usize + (u * texture.resolution_u as f32) as usize) + 0];
	screen[4 * (y * SCREEN_WIDTH as usize + x) + 1] = texture.pixels[4 * ((v * texture.resolution_v as f32) as usize * texture.resolution_u as usize + (u * texture.resolution_u as f32) as usize) + 1];
	screen[4 * (y * SCREEN_WIDTH as usize + x) + 2] = texture.pixels[4 * ((v * texture.resolution_v as f32) as usize * texture.resolution_u as usize + (u * texture.resolution_u as f32) as usize) + 2];
	screen[4 * (y * SCREEN_WIDTH as usize + x) + 3] = texture.pixels[4 * ((v * texture.resolution_v as f32) as usize * texture.resolution_u as usize + (u * texture.resolution_u as f32) as usize) + 3];
}

fn texture_triangle(
	screen:&mut [u8],
	depth_buffer:&mut [f32],
	mut x1:i16, mut y1:i16, mut u1:f32, mut v1:f32, mut w1:f32,
	mut x2:i16, mut y2:i16, mut u2:f32, mut v2:f32, mut w2:f32,
	mut x3:i16, mut y3:i16, mut u3:f32, mut v3:f32, mut w3:f32,
	texture: &Texture
) {
	if y2 < y1 {
		core::mem::swap(&mut y1, &mut y2);
		core::mem::swap(&mut x1, &mut x2);
		core::mem::swap(&mut u1, &mut u2);
		core::mem::swap(&mut v1, &mut v2);
		core::mem::swap(&mut w1, &mut w2);
	}

	if y3 < y1 {
		core::mem::swap(&mut y1, &mut y3);
		core::mem::swap(&mut x1, &mut x3);
		core::mem::swap(&mut u1, &mut u3);
		core::mem::swap(&mut v1, &mut v3);
		core::mem::swap(&mut w1, &mut w3);
	}

	if y3 < y2 {
		core::mem::swap(&mut y2, &mut y3);
		core::mem::swap(&mut x2, &mut x3);
		core::mem::swap(&mut u2, &mut u3);
		core::mem::swap(&mut v2, &mut v3);
		core::mem::swap(&mut w2, &mut w3);
	}

	let dy1 = y2 - y1;
	let dx1 = x2 - x1;
	let dv1 = v2 - v1;
	let du1 = u2 - u1;
	let dw1 = w2 - w1;

	let dy2 = y3 - y1;
	let dx2 = x3 - x1;
	let dv2 = v3 - v1;
	let du2 = u3 - u1;
	let dw2 = w3 - w1;

	let mut tex_u;
	let mut tex_v;
	let mut tex_w;

	let mut dax_step = 0f32;
	let mut dbx_step = 0f32;
	let mut du1_step = 0f32;
	let mut dv1_step = 0f32;
	let mut du2_step = 0f32;
	let mut dv2_step = 0f32;
	let mut dw1_step = 0f32;
	let mut dw2_step = 0f32;

	if dy2 != 0 {
		dbx_step = dx2 as f32 / dy2.abs() as f32;
		du2_step = du2 / dy2.abs() as f32;
		dv2_step = dv2 / dy2.abs() as f32;
		dw2_step = dw2 / dy2.abs() as f32;
	}

	if dy1 != 0 {
		dax_step = dx1 as f32 / dy1.abs() as f32;
		du1_step = du1 / dy1.abs() as f32;
		dv1_step = dv1 / dy1.abs() as f32;
		dw1_step = dw1 / dy1.abs() as f32;

		for y in y1..=y2 {
			let mut ax = x1 + ((y - y1) as f32 * dax_step) as i16;
			let mut bx = x1 + ((y - y1) as f32 * dbx_step) as i16;

			let mut tex_su = u1 + (y - y1) as f32 * du1_step;
			let mut tex_sv = v1 + (y - y1) as f32 * dv1_step;
			let mut tex_sw = w1 + (y - y1) as f32 * dw1_step;

			let mut tex_eu = u1 + (y - y1) as f32 * du2_step;
			let mut tex_ev = v1 + (y - y1) as f32 * dv2_step;
			let mut tex_ew = w1 + (y - y1) as f32 * dw2_step;

			if ax > bx {
				core::mem::swap(&mut ax, &mut bx);
				core::mem::swap(&mut tex_su, &mut tex_eu);
				core::mem::swap(&mut tex_sv, &mut tex_ev);
				core::mem::swap(&mut tex_sw, &mut tex_ew);
			}

			tex_u = tex_su;
			tex_v = tex_sv;
			tex_w = tex_sw;

			let tstep = 1f32 / ((bx - ax) as f32);
			let mut t = 0f32;

			for x in ax..bx {
				tex_u = (1f32 - t) * tex_su + t * tex_eu;
				tex_v = (1f32 - t) * tex_sv + t * tex_ev;
				tex_w = (1f32 - t) * tex_sw + t * tex_ew;
				if y >= 0 && x >= 0 && y < SCREEN_HEIGHT as i16 && x < SCREEN_WIDTH as i16 {
					if tex_w > depth_buffer[y as usize * SCREEN_WIDTH as usize + x as usize] {
						Draw(screen, texture, x as usize, y as usize, tex_u / tex_w, tex_v / tex_w);
						depth_buffer[y as usize * SCREEN_WIDTH as usize + x as usize] = tex_w;
					}
				}
				t += tstep;
			}
		}
	}

	let dy1 = y3 - y2;
	let dx1 = x3 - x2;
	let dv1 = v3 - v2;
	let du1 = u3 - u2;
	let dw1 = w3 - w2;

	if dy2 != 0 {
		dbx_step = dx2 as f32 / dy2.abs() as f32;
	}

	du1_step = 0f32;
	dv1_step = 0f32;

	if dy1 != 0 {
		dax_step = dx1 as f32 / dy1.abs() as f32;
		du1_step = du1 / dy1.abs() as f32;
		dv1_step = dv1 / dy1.abs() as f32;
		dw1_step = dw1 / dy1.abs() as f32;

		for y in y2..=y3 {
			let mut ax = x2 + ((y - y2) as f32 * dax_step) as i16;
			let mut bx = x1 + ((y - y1) as f32 * dbx_step) as i16;

			let mut tex_su = u2 + (y - y2) as f32 * du1_step;
			let mut tex_sv = v2 + (y - y2) as f32 * dv1_step;
			let mut tex_sw = w2 + (y - y2) as f32 * dw1_step;

			let mut tex_eu = u1 + (y - y1) as f32 * du2_step;
			let mut tex_ev = v1 + (y - y1) as f32 * dv2_step;
			let mut tex_ew = w1 + (y - y1) as f32 * dw2_step;

			if ax > bx {
				core::mem::swap(&mut ax, &mut bx);
				core::mem::swap(&mut tex_su, &mut tex_eu);
				core::mem::swap(&mut tex_sv, &mut tex_ev);
				core::mem::swap(&mut tex_sw, &mut tex_ew);
			}

			tex_u = tex_su;
			tex_v = tex_sv;
			tex_w = tex_sw;

			let tstep = 1f32 / (bx - ax) as f32;
			let mut t = 0f32;

			for x in ax..bx {
				tex_u = (1f32 - t) * tex_su + t * tex_eu;
				tex_v = (1f32 - t) * tex_sv + t * tex_ev;
				tex_w = (1f32 - t) * tex_sw + t * tex_ew;

				if y >= 0 && x >= 0 && y < SCREEN_HEIGHT as i16 && x < SCREEN_WIDTH as i16 {
					if tex_w > depth_buffer[y as usize * SCREEN_WIDTH as usize + x as usize] {
						Draw(screen, texture, x as usize, y as usize, tex_u / tex_w, tex_v / tex_w);
						depth_buffer[y as usize * SCREEN_WIDTH as usize + x as usize] = tex_w;
					}
				}
				t += tstep;
			}
		}
	}
}

#[inline(always)]
unsafe fn draw_unsafe(
	screen:&mut [u8],
	texture:&Texture,
	pix_idx:usize, uv_idx:usize,
	depth_buffer:&mut [f32],
	tex_w:f32
) {
	let a = *texture.pixels.get_unchecked(4 * uv_idx + 3);
	if a != 0 {
		*screen.get_unchecked_mut(4 * pix_idx + 0) = *texture.pixels.get_unchecked(4 * uv_idx + 0);
		*screen.get_unchecked_mut(4 * pix_idx + 1) = *texture.pixels.get_unchecked(4 * uv_idx + 1);
		*screen.get_unchecked_mut(4 * pix_idx + 2) = *texture.pixels.get_unchecked(4 * uv_idx + 2);
		*screen.get_unchecked_mut(4 * pix_idx + 3) = 0xFF;
		*depth_buffer.get_unchecked_mut(pix_idx) = tex_w;
	}
}

#[inline(always)]
fn draw_safe(
	screen:&mut [u8],
	texture:&Texture,
	pix_idx:usize, uv_idx:usize,
) {
	screen[4 * pix_idx + 0] = texture.pixels[4 * uv_idx + 0];
	screen[4 * pix_idx + 1] = texture.pixels[4 * uv_idx + 1];
	screen[4 * pix_idx + 2] = texture.pixels[4 * uv_idx + 2];
	screen[4 * pix_idx + 3] = texture.pixels[4 * uv_idx + 3];
}

fn texture_triangle_unsafe(
	screen:&mut [u8],
	depth_buffer:&mut [f32],
	mut x1:i16, mut y1:i16, mut u1:f32, mut v1:f32, mut w1:f32,
	mut x2:i16, mut y2:i16, mut u2:f32, mut v2:f32, mut w2:f32,
	mut x3:i16, mut y3:i16, mut u3:f32, mut v3:f32, mut w3:f32,
	texture: &Texture
) {
	if y2 < y1 {
		core::mem::swap(&mut y1, &mut y2);
		core::mem::swap(&mut x1, &mut x2);
		core::mem::swap(&mut u1, &mut u2);
		core::mem::swap(&mut v1, &mut v2);
		core::mem::swap(&mut w1, &mut w2);
	}

	if y3 < y1 {
		core::mem::swap(&mut y1, &mut y3);
		core::mem::swap(&mut x1, &mut x3);
		core::mem::swap(&mut u1, &mut u3);
		core::mem::swap(&mut v1, &mut v3);
		core::mem::swap(&mut w1, &mut w3);
	}

	if y3 < y2 {
		core::mem::swap(&mut y2, &mut y3);
		core::mem::swap(&mut x2, &mut x3);
		core::mem::swap(&mut u2, &mut u3);
		core::mem::swap(&mut v2, &mut v3);
		core::mem::swap(&mut w2, &mut w3);
	}

	let dy1 = y2 - y1;
	let dx1 = x2 - x1;
	let dv1 = v2 - v1;
	let du1 = u2 - u1;
	let dw1 = w2 - w1;

	let dy2 = y3 - y1;
	let dx2 = x3 - x1;
	let dv2 = v3 - v1;
	let du2 = u3 - u1;
	let dw2 = w3 - w1;

	let mut tex_u;
	let mut tex_v;
	let mut tex_w;

	let mut dax_step = 0f32;
	let mut dbx_step = 0f32;
	let mut du1_step = 0f32;
	let mut dv1_step = 0f32;
	let mut du2_step = 0f32;
	let mut dv2_step = 0f32;
	let mut dw1_step = 0f32;
	let mut dw2_step = 0f32;

	if dy2 != 0 {
		dbx_step = dx2 as f32 / dy2.abs() as f32;
		du2_step = du2 / dy2.abs() as f32;
		dv2_step = dv2 / dy2.abs() as f32;
		dw2_step = dw2 / dy2.abs() as f32;
	}

	if dy1 != 0 {
		dax_step = dx1 as f32 / dy1.abs() as f32;
		du1_step = du1 / dy1.abs() as f32;
		dv1_step = dv1 / dy1.abs() as f32;
		dw1_step = dw1 / dy1.abs() as f32;

		for y in y1..=y2 {
			let mut ax = x1 + ((y - y1) as f32 * dax_step) as i16;
			let mut bx = x1 + ((y - y1) as f32 * dbx_step) as i16;

			let mut tex_su = u1 + (y - y1) as f32 * du1_step;
			let mut tex_sv = v1 + (y - y1) as f32 * dv1_step;
			let mut tex_sw = w1 + (y - y1) as f32 * dw1_step;

			let mut tex_eu = u1 + (y - y1) as f32 * du2_step;
			let mut tex_ev = v1 + (y - y1) as f32 * dv2_step;
			let mut tex_ew = w1 + (y - y1) as f32 * dw2_step;

			if ax > bx {
				core::mem::swap(&mut ax, &mut bx);
				core::mem::swap(&mut tex_su, &mut tex_eu);
				core::mem::swap(&mut tex_sv, &mut tex_ev);
				core::mem::swap(&mut tex_sw, &mut tex_ew);
			}

			let tstep = 1f32 / ((bx - ax) as f32);
			let mut t = 0f32;

			for x in ax..bx {
				tex_u = (1f32 - t) * tex_su + t * tex_eu;
				tex_v = (1f32 - t) * tex_sv + t * tex_ev;
				tex_w = (1f32 - t) * tex_sw + t * tex_ew;
				// if y >= 0 && x >= 0 && y < SCREEN_HEIGHT as i16 && x < SCREEN_WIDTH as i16 {
				let pix_idx = y as usize * SCREEN_WIDTH as usize + x as usize;
				unsafe {
					// if tex_w > depth_buffer[pix_idx] {
					if tex_w > *depth_buffer.get_unchecked(pix_idx) {
						let cu = tex_u / tex_w;
						let cv = tex_v / tex_w;
						let uv_idx = (cv * texture.resolution_v as f32) as usize * texture.resolution_u as usize + (cu * texture.resolution_u as f32) as usize;
						draw_unsafe(screen, texture, pix_idx, uv_idx, depth_buffer, tex_w);
						// draw_safe(screen, texture, pix_idx, uv_idx);
						// depth_buffer[pix_idx] = tex_w;
					}
				}
				// }
				t += tstep;
			}
		}
	}

	let dy1 = y3 - y2;
	let dx1 = x3 - x2;
	let dv1 = v3 - v2;
	let du1 = u3 - u2;
	let dw1 = w3 - w2;

	if dy2 != 0 {
		dbx_step = dx2 as f32 / dy2.abs() as f32;
	}

	du1_step = 0f32;
	dv1_step = 0f32;

	if dy1 != 0 {
		dax_step = dx1 as f32 / dy1.abs() as f32;
		du1_step = du1 / dy1.abs() as f32;
		dv1_step = dv1 / dy1.abs() as f32;
		dw1_step = dw1 / dy1.abs() as f32;

		for y in y2..=y3 {
			let mut ax = x2 + ((y - y2) as f32 * dax_step) as i16;
			let mut bx = x1 + ((y - y1) as f32 * dbx_step) as i16;

			let mut tex_su = u2 + (y - y2) as f32 * du1_step;
			let mut tex_sv = v2 + (y - y2) as f32 * dv1_step;
			let mut tex_sw = w2 + (y - y2) as f32 * dw1_step;

			let mut tex_eu = u1 + (y - y1) as f32 * du2_step;
			let mut tex_ev = v1 + (y - y1) as f32 * dv2_step;
			let mut tex_ew = w1 + (y - y1) as f32 * dw2_step;

			if ax > bx {
				core::mem::swap(&mut ax, &mut bx);
				core::mem::swap(&mut tex_su, &mut tex_eu);
				core::mem::swap(&mut tex_sv, &mut tex_ev);
				core::mem::swap(&mut tex_sw, &mut tex_ew);
			}

			if ax < 0 {
				if bx < 0 {
					continue;
				}

				if bx >= SCREEN_WIDTH as i16 {
					bx = SCREEN_WIDTH as i16  - 1;
				}
				ax = 0;
			}

			let tstep = 1f32 / (bx - ax) as f32;
			let mut t = 0f32;

			for x in ax..bx {
				tex_u = (1f32 - t) * tex_su + t * tex_eu;
				tex_v = (1f32 - t) * tex_sv + t * tex_ev;
				tex_w = (1f32 - t) * tex_sw + t * tex_ew;

				// if y >= 0 && x >= 0 && y < SCREEN_HEIGHT as i16 && x < SCREEN_WIDTH as i16 {
				let pix_idx = y as usize * SCREEN_WIDTH as usize + x as usize;
				unsafe {
					// if tex_w > depth_buffer[pix_idx] {
					if tex_w > *depth_buffer.get_unchecked(pix_idx) {
						let cu = tex_u / tex_w;
						let cv = tex_v / tex_w;
						let uv_idx = (cv * texture.resolution_v as f32) as usize * texture.resolution_u as usize + (cu * texture.resolution_u as f32) as usize;
						draw_unsafe(screen, texture, pix_idx, uv_idx, depth_buffer, tex_w);
						// draw_safe(screen, texture, pix_idx, uv_idx);
						// depth_buffer[pix_idx] = tex_w;
					}
				}
				// }
				t += tstep;
			}
		}
	}
}

fn intersect_vector_plane(plane_p:&Vec3d, plane_n:&Vec3d, line_start:&Vec3d, line_end:&Vec3d) -> (Vec3d, f32) {
	// TODO: let plane_n = normalize(plane_n);
	let plane_d = -Vec3d::dot_product(&plane_n, &plane_p);
	let ad = Vec3d::dot_product(line_start, &plane_n);
	let bd = Vec3d::dot_product(line_end,   &plane_n);
	let t = (-plane_d -  ad) / (bd - ad);
	let line_start_to_end = *line_end - *line_start;
	let line_to_intersect = mul_vec3d(&line_start_to_end, t);
	return (*line_start + line_to_intersect, t);
}

fn triangle_clip_against_plane(plane_p:&Vec3d, plane_n:&Vec3d, in_tri:&Triangle) -> (usize, [Triangle;2]) {
	// Make sure plane normal is indeed normal
	// TODO: let plane_n = normalize(plane_n);

	// Return signed shortest distance from point to plane, plane normal must be normalised
	fn dist(plane_p:&Vec3d, plane_n:&Vec3d, p:&Vec3d) -> f32 {
		plane_n.x*p.x + plane_n.y*p.y + plane_n.z*p.z - Vec3d::dot_product(plane_n, plane_p)
	}

	// Create two temporary storage arrays to classify points either side of plane
	let zv3:Vec3d = Vec3d::ZERO;
	let zv2:VecUVW = VecUVW::ZERO;
	let mut  inside_points = [&zv3;3];
	let mut outside_points = [&zv3;3];
	let mut  inside_uvs    = [&zv2;3];
	let mut outside_uvs    = [&zv2;3];
	let mut  inside_counter = 0;
	let mut outside_counter = 0;

	for i in 0..3 {
		let v = &in_tri.vertices[i];
		let uv = &in_tri.uvs[i];
		// Get signed distance of each point in triangle to plane
		let d = dist(plane_p, &plane_n, v);

		// If distance sign is positive, point lies on "inside" of plane
		if d >= 0f32 {
			inside_points[inside_counter] = v;
			inside_uvs[inside_counter] = uv;
			inside_counter += 1;
		} else {
			outside_points[outside_counter] = v;
			outside_uvs[outside_counter] = uv;
			outside_counter += 1;
		}
	}

	// Now classify triangle points, and break the input triangle into smaller output triangles if required. There are four possible outcomes...
	match inside_counter {
		0 => {
			// All points lie on the outside of plane, so clip whole triangle It ceases to exist
			return (0, [Triangle::zero_spaced_verticies(); 2]); // No returned triangles are valid
		},
		3 => {
			// All points lie on the inside of plane, so do nothing and allow the triangle to simply pass through
			return (1, [in_tri.clone(), Triangle::zero_spaced_verticies()]); // Just the one returned original triangle is valid
		},
		1 => {
			// Triangle should be clipped. As two points lie outside the plane, the triangle simply becomes a smaller triangle
			let mut ot = in_tri.clone();
			// The inside point is valid, so keep that...
			ot.vertices[0] = inside_points[0].clone();
			ot.uvs[0] = inside_uvs[0].clone();

			// but the two new points are at the locations where the original sides of the triangle (lines) intersect with the plane

			let (v, t) = intersect_vector_plane(plane_p, &plane_n, &inside_points[0], &outside_points[0]);
			ot.vertices[1] = v;
			ot.uvs[1] = *inside_uvs[0] + (*outside_uvs[0] - *inside_uvs[0]) * t;

			let (v, t) = intersect_vector_plane(plane_p, &plane_n, &inside_points[0], &outside_points[1]);
			ot.vertices[2] = v;
			ot.uvs[2] = *inside_uvs[0] + (*outside_uvs[1] - *inside_uvs[0]) * t;

			return (1, [ot, Triangle::zero_spaced_verticies()]); // Return the newly formed single triangle
		},
		2 => {
			// Triangle should be clipped. As two points lie inside the plane,
			// the clipped triangle becomes a "quad". Fortunately, we can
			// represent a quad with two new triangles

			// Copy appearance info to new triangles
			let mut ot1 = in_tri.clone();
			let mut ot2 = in_tri.clone();

			// The first triangle consists of the two inside points and a new
			// point determined by the location where one side of the triangle
			// intersects with the plane
			ot1.vertices[0] = inside_points[0].clone();
			ot1.vertices[1] = inside_points[1].clone();
			ot1.uvs[0] = inside_uvs[0].clone();
			ot1.uvs[1] = inside_uvs[1].clone();
			let (v, t) = intersect_vector_plane(plane_p, &plane_n, &inside_points[0], &outside_points[0]);
			ot1.vertices[2] = v;
			ot1.uvs[2] = *inside_uvs[0] + (*outside_uvs[0] - *inside_uvs[0]) * t;

			// The second triangle is composed of one of he inside points, a new point determined by the intersection of the other side of the triangle and the plane, and the newly created point above
			ot2.vertices[0] = inside_points[1].clone();
			ot2.uvs[0] = inside_uvs[1].clone();
			ot2.vertices[1] = ot1.vertices[2].clone();
			ot2.uvs[1] = ot1.uvs[2].clone();
			// TODO: Bug ot2.uvs[1] = inside_uvs[2].clone();

			let (v, t) = intersect_vector_plane(plane_p, &plane_n, &inside_points[1], &outside_points[0]);
			ot2.vertices[2] = v;
			ot2.uvs[2] = *inside_uvs[1] + (*outside_uvs[0] - *inside_uvs[1]) * t;

			return (2, [ot1, ot2]); // Return two newly formed triangles which form a quad
		},
		_ => unreachable!()
	}
}

fn projection_matrix(fov_degrees:f32, aspect_ratio:f32, plane_near:f32, plane_far:f32) -> Matrix4x4 {
	let fov_rad = 1f32 / (fov_degrees * PI / 360f32).tan();
	let mut matrix = Matrix4x4::zeros();
	matrix.matrix[0][0] = aspect_ratio * fov_rad;
	matrix.matrix[1][1] = fov_rad;
	matrix.matrix[2][2] = plane_far / (plane_far - plane_near);
	matrix.matrix[3][2] = (-plane_far * plane_near) / (plane_far - plane_near);
	matrix.matrix[2][3] = 1f32;
	matrix.matrix[3][3] = 0f32;
	return matrix;
}

fn transition_matrix(x:f32, y:f32, z:f32) -> Matrix4x4 {
	let mut matrix = Matrix4x4::ones_cascade();
	matrix.matrix[3][0] = x;
	matrix.matrix[3][1] = y;
	matrix.matrix[3][2] = z;
	return matrix;
}

fn mul_matrix(matrix1:&Matrix4x4, matrix2:&Matrix4x4) -> Matrix4x4 {
	let mut  matrix = Matrix4x4::zeros();
	for c in 0..4 {
		for r in 0..4 {
			matrix.matrix[r][c] =
				matrix1.matrix[r][0] * matrix2.matrix[0][c] +
					matrix1.matrix[r][1] * matrix2.matrix[1][c] +
					matrix1.matrix[r][2] * matrix2.matrix[2][c] +
					matrix1.matrix[r][3] * matrix2.matrix[3][c]
			;
		}
	}
	return matrix;
}