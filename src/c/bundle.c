#include <math.h>
// #include <emscripten/emscripten.h>

#define  u8 unsigned char
#define  s8   signed char
#define u16 unsigned short int
#define s16   signed short int
#define u32 unsigned long int
#define s32   signed long int
//#define u64 unsigned long long int // TODO: добавляет зависимость на _i64Add
//#define s64   signed long long int // TODO: добавляет зависимость на _i64Add
#define f32 float
//#define f64 double // TODO: добавляет зависимость на _i64Add

#include "model_bin.c"

// ==== UTIL ====

f32 limit_f32(f32 min, f32 value, f32 max) {
	if(value < min)
		return min;
	if(value > max)
		return max;
	return value;
}

u8 min_u8(u8 a, u8 b) {
	return a < b ? a : b;
}

// ==== DISPlAY ====

#define screen_width 128
#define screen_height 128
#define PIf 3.14159265358979323846f

static u8 image[screen_width * screen_height * 4];

// ==== TIMING VARS ====

static u32 tickNumber = 0;

// ==== INPUT ====

static u8 is_pointer_locked = 0;
static u8 m_rounded_x = screen_width/2;
static u8 m_rounded_y = screen_height/2;
static f32 m_prec_x = screen_width/2;
static f32 m_prec_y = screen_height/2;
static u8 m_down = 0;
static u8 m_up = 0;
void input_loop(u8 locked_pointer, f32 abs_x, f32 abs_y, f32 delta_x, f32 delta_y, f32 scale, u8 mouse_down, u8 mouse_up) {
	if(!is_pointer_locked || !locked_pointer) {
		m_prec_x = abs_x / scale;
		m_prec_y = abs_y / scale;
	}
	is_pointer_locked = locked_pointer;
	if(is_pointer_locked) {
		m_prec_x += delta_x / scale;
		m_prec_y += delta_y / scale;
	}
	m_prec_x = limit_f32((f32)0, m_prec_x, (f32)screen_width);
	m_prec_y = limit_f32((f32)0, m_prec_y, (f32)screen_width);
	m_rounded_x = min_u8((u8)(m_prec_x+0.5F), screen_width - 1);
	m_rounded_y = min_u8((u8)(m_prec_y+0.5F), screen_height - 1);
	m_down = mouse_down;
	m_up = mouse_up;
}

struct vec2f {
	float x, y;
};

struct vec3f {
	float x ,y, z;
};

struct vec3f addVec3f(struct vec3f vec1, struct vec3f vec2) {
 struct vec3f ret;

	ret.x = vec1.x+vec2.x;
	ret.y = vec1.x+vec2.y;
	ret.z = vec1.x+vec2.z;

	return ret;
}

struct triangle {
	struct vec3f verticies[3];
};

struct matrix4x4 {
	float matrix[4][4];
};

void fill_matrix_4x4(struct matrix4x4 *matrix, float value) {
	matrix->matrix[0][0] = value; matrix->matrix[0][1] = value;
	matrix->matrix[0][2] = value;
	matrix->matrix[0][3] = value;
	matrix->matrix[1][0] = value;
	matrix->matrix[1][1] = value;
	matrix->matrix[1][2] = value;
	matrix->matrix[1][3] = value;
	matrix->matrix[2][0] = value;
	matrix->matrix[2][1] = value;
	matrix->matrix[2][2] = value;
	matrix->matrix[2][3] = value;
	matrix->matrix[3][0] = value;
	matrix->matrix[3][1] = value;
	matrix->matrix[3][2] = value;
	matrix->matrix[3][3] = value;
}

void multiply_vector_matrix(struct vec3f *in, struct matrix4x4 *matrix, struct vec3f *out) {
	out->x  = in->x * matrix->matrix[0][0] + in->y * matrix->matrix[1][0] + in->z * matrix->matrix[2][0] + matrix->matrix[3][0];
	out->y  = in->x * matrix->matrix[0][1] + in->y * matrix->matrix[1][1] + in->z * matrix->matrix[2][1] + matrix->matrix[3][1];
	out->z  = in->x * matrix->matrix[0][2] + in->y * matrix->matrix[1][2] + in->z * matrix->matrix[2][2] + matrix->matrix[3][2];
	float w = in->x * matrix->matrix[0][3] + in->y * matrix->matrix[1][3] + in->z * matrix->matrix[2][3] + matrix->matrix[3][3];

	if(w != 0) { // TODO: когда так бывает?
		out->x /= w;
		out->y /= w;
		out->z /= w;
	}
}

#define max_basic_mesh_triangles 1024
struct mesh_basic {
	struct triangle triangles[max_basic_mesh_triangles];
	u16 amount_of_triangles;
};

u16 u8_to_u16(u8 first, u8 second) {
	return (
		(first << 8) |
		(second    )
	);
}

void load_mesh(struct mesh_basic *mesh, u8 *binary) {
	u16 verticies_amount = u8_to_u16(binary[0], binary[1]);
	mesh->amount_of_triangles = u8_to_u16(binary[2], binary[3]);

	struct vec3f verticies[1024];
	for(u16 i = 0; i != verticies_amount; i++) {
		verticies[i].x = binary[4+2*(i*3)+0];
		verticies[i].y = binary[4+2*(i*3)+1];
		verticies[i].z = binary[4+2*(i*3)+2];
	}
	
	for(u16 i = 0; i != mesh->amount_of_triangles; i++) {
		mesh->triangles[i].verticies[0] = verticies[binary[2+2*(verticies_amount*3 + i*3)+0]];
		mesh->triangles[i].verticies[1] = verticies[binary[2+2*(verticies_amount*3 + i*3)+1]];
		mesh->triangles[i].verticies[2] = verticies[binary[2+2*(verticies_amount*3 + i*3)+2]];
	}
}

s16 abs_s16_unsafe(s16 value) {
	return value & (u16)0x7FFF;
}
s16 abs_s16(s16 value) {
	return value < 0 ? -value : value;
}

void draw_line_unsafe(u8 *screen, s16 x0, s16 y0, s16 x1, s16 y1) {
	s32 dx = (s32)abs_s16(x1-x0), sx = x0<x1 ? 1 : -1;
	s32 dy = (s32)abs_s16(y1-y0), sy = y0<y1 ? 1 : -1; 
	s32 err = (dx>dy ? dx : -dy)/2, e2;
 
 	while(1) {
		u32 offset = ((u32)y0*(u32)screen_height+(u32)x0)*4;
		screen[offset+0] = (u8)0xFF;
		screen[offset+1] = (u8)0xFF;
		screen[offset+2] = (u8)0xFF;
		screen[offset+4] = 0;
		if (x0==x1 && y0==y1) break;
		e2 = err;
		if (e2 >-dx) { err -= dy; x0 += sx; }
		if (e2 < dy) { err += dx; y0 += sy; }
	}
}

void draw_line_safe(u8 *screen, s16 x0, s16 y0, s16 x1, s16 y1) {
	s32 dx = (s32)abs_s16(x1-x0), sx = x0<x1 ? 1 : -1;
	s32 dy = (s32)abs_s16(y1-y0), sy = y0<y1 ? 1 : -1; 
	s32 err = (dx>dy ? dx : -dy)/2, e2;
 
 	while(1) {
		if(
			x0 >= 0 && x0 <= screen_width &&
			y0 >= 0 && y0 <= screen_height
		) {
			u32 offset = ((u32)y0*(u32)screen_height+(u32)x0)*4;
			screen[offset+0] = (u8)0xFF;
			screen[offset+1] = (u8)0xFF;
			screen[offset+2] = (u8)0xFF;
			screen[offset+4] = 0;
		}

		if(x0 == x1 && y0 == y1)
			break;

		e2 = err;
		if (e2 > -dx) { err -= dy; x0 += sx; }
		if (e2 <  dy) { err += dx; y0 += sy; }
	}
}

static struct mesh_basic mesh;
static void init() {
	load_mesh(&mesh, (u8*) &model_bin);
}

void render_loop() {
	for(u16 i = 0; i != screen_width*screen_height; i++) {
		image[i*4+0] = 0;
		image[i*4+1] = 0;
		image[i*4+2] = 0;
		image[i*4+3] = 0;
	}

	// Projection Matrix
	float fov = 90.0f;
	float planeNear = 0.1f;
	float planeFar = 1000.0f;
	float aspect = (float)screen_height / (float)screen_width; // TODO: swap?
	float fFovRad = 1.0f / tanf(fov * 0.5f / 180.0f * PIf);
	
	// TODO play around struct matrix4x4 projectionMatrix = { .matrix = {0} };
	struct matrix4x4 projectionMatrix;
	fill_matrix_4x4(&projectionMatrix, 0);
	// TODO: fill matrix with 0s
	projectionMatrix.matrix[0][0] = aspect * fFovRad;
	projectionMatrix.matrix[1][1] =          fFovRad;
	projectionMatrix.matrix[2][2] =   planeFar              / (planeFar - planeNear);
	projectionMatrix.matrix[3][2] = (-planeFar * planeNear) / (planeFar - planeNear);
	projectionMatrix.matrix[2][3] = 1.0f;
	projectionMatrix.matrix[3][3] = 0.0f;
	
	for(u16 i = 0; i != mesh.amount_of_triangles; i++) {
		struct triangle *t = &mesh.triangles[i];
		struct triangle projectedTriangle;
		multiply_vector_matrix(&t->verticies[0], &projectionMatrix, &projectedTriangle.verticies[0]);
		multiply_vector_matrix(&t->verticies[1], &projectionMatrix, &projectedTriangle.verticies[1]);
		multiply_vector_matrix(&t->verticies[2], &projectionMatrix, &projectedTriangle.verticies[2]);

		draw_line_safe(image, projectedTriangle.verticies[0].x, projectedTriangle.verticies[0].y, projectedTriangle.verticies[1].x, projectedTriangle.verticies[1].y);
		draw_line_safe(image, projectedTriangle.verticies[0].x, projectedTriangle.verticies[0].y, projectedTriangle.verticies[2].x, projectedTriangle.verticies[2].y);
		draw_line_safe(image, projectedTriangle.verticies[1].x, projectedTriangle.verticies[1].y, projectedTriangle.verticies[2].x, projectedTriangle.verticies[2].y);
	}

	u32 m = sizeof(model_bin);

	for(u32 i = 0; i != m; i++) {
		image[i    ] = m_up ? 0 : 255;
		image[i + 1] = (0x7F & (i - m_rounded_x)) + tickNumber * 8;
		image[i + 2] = (0x7F & (i - m_rounded_y)) + tickNumber * 4;
		image[i + 3] = m_down ? 0 : 255;
	}
	/*for(u8 x = 0; x < screen_width; x++) {
	for(u8 y = 0; y < screen_height; y++) {
		int idx = (y * screen_width + x) * 4;
		image[idx	] = m_up ? 0 : 255;
		image[idx + 1] = (0x7F & (x - m_rounded_x)) + tickNumber * 8;
		image[idx + 2] = (0x7F & (y - m_rounded_y)) + tickNumber * 4;
		image[idx + 3] = m_down ? 0 : 255;
	}
	}

	for(u8 y = 0; y != 2; y++) {
	for(u8 x = 0; x != 2; x++) {
		for(u8 by = 0; by != block_height; by++) {
		for(u8 bx = 0; bx != block_width; bx++) {
			u16 pixel = ((y*block_m_height+blocks_margin+by)*screen_width + x*block_m_width+blocks_margin+bx)*4;
			image[ pixel ] = 0;
			image[ pixel+1 ] = 0;
			image[ pixel+2 ] = 0;
			image[ pixel+3 ] = 0;
		}
		}
	}
	}*/
}

void logic_tick(u8 deltaTicks) {
	
}

// ==== TIMING ====

#define targetFPS 100
#define maxTicksPerFrame 4
#define targetSleep ((u8)((1000.0F/(f32)targetFPS)+0.99F))

static s32 lastTickTimeStamp = -0xFFFFFFF;
void timing_loop(s32 timeStamp) {
	s32 delta = timeStamp - lastTickTimeStamp;

	if(delta > 0) {
		u32 ticks = delta / targetSleep;

		if(ticks > maxTicksPerFrame) {
			ticks = maxTicksPerFrame;
			lastTickTimeStamp = (s32)timeStamp;
		} else {
			lastTickTimeStamp += (s32)(ticks*targetSleep);
		}

		if(ticks != 0) {
			tickNumber += ticks;
			logic_tick((u8)ticks);
			render_loop();
		}
	}
}

// ==== EXPORTS ====

#define EMSCRIPTEN_KEEPALIVE __attribute__((used)) __attribute__ ((visibility ("default")))
extern void EMSCRIPTEN_KEEPALIVE t(s32 timeStamp, u8 locked_pointer, f32 abs_x, f32 abs_y, f32 delta_x, f32 delta_y, f32 scale, u8 mouse_down, u8 mouse_up) {
	input_loop(locked_pointer, abs_x, abs_y, delta_x, delta_y, scale, mouse_down, mouse_up);
	timing_loop(timeStamp);
}
extern u8* EMSCRIPTEN_KEEPALIVE p() {
	return &image[0];
}
extern void EMSCRIPTEN_KEEPALIVE i() {
	init();
}