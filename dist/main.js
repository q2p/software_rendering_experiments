'use strict'

var hex1 = (a) => new Uint8Array(a.match(/.{1,2}/g).map((i) => parseInt(i, 16)));
var hex2 = (a) => {
	var b = [];
	for (var i = 0; i < a.length; i += 2)
		b.push(parseInt(a.substr(i, 2), 16));
	return new Uint8Array(b);
};
var width = 1;
var height = 1;
var scale = -1;
var instance;
const canvas = document.getElementsByTagName("canvas")[0];
function r() {
	var dw = document.body.clientWidth;
	var dh = document.body.clientHeight;

	// min size

	// if ((dw * height) / (dh * width) >= 1)
	// 	scale = Math.max(1, Math.floor(dh / height));
	// else
	// 	scale = Math.max(1, Math.floor(dw / width));

	let ww = width;
	let hh = height;
	if ((dw * hh) / (dh * ww) >= 1) {
		if(hh > dh) {
			do {
				hh /= 2;
				ww /= 2;
			} while(hh > dh);
			canvas.classList.remove("p");
			canvas.classList.add("s");
		} else {
			let scale = Math.floor(dh / hh);
			ww *= scale;
			hh *= scale;
			canvas.classList.remove("s");
			canvas.classList.add("p");
		}
	} else {
		if(ww > dw) {
			do {
				hh /= 2;
				ww /= 2;
			} while(ww > dw);
			canvas.classList.remove("p");
			canvas.classList.add("s");
		} else {
			let scale = Math.floor(dw / ww);
			ww *= scale;
			hh *= scale;
			canvas.classList.remove("s");
			canvas.classList.add("p");
		}
	}

	canvas.style.width  = Math.floor(ww) + "px";
	canvas.style.height = Math.floor(hh) + "px";
}
var ctx = canvas.getContext("2d");
var tk;
var img;
function animationLoop(timestamp) {
	tk(timestamp,locked,mx,my,rx,ry,scale,md,mu);
	rx = ry = md = mu = 0;
	ctx.putImageData(img, 0, 0);
	requestAnimationFrame(animationLoop);
}
let imports = {
	env: {
		// memoryBase: 0,
		// tableBase: 0,
		// cos: Math.cos,
		// __table_base: 0,
		// __memory_base: 0,
		memory: new WebAssembly.Memory({ initial: 256 }),
		// table: new WebAssembly.Table({ initial: 0, element: 'anyfunc' }),
	}
};
let rpl = canvas.requestPointerLock || canvas.mozRequestPointerLock || canvas.webkitRequestPointerLock;
fetch("emc.wasm").then(response => response.arrayBuffer()).then(bytes => {
    return WebAssembly.instantiate(bytes, imports);
}).then(results => {
	instance = results.instance;
	tk = instance.exports.t;
	width = instance.exports.w();
	height = instance.exports.h();
	canvas.width = width;
	canvas.height = height;
	var offset = instance.exports.p.value;
	let byteSize = width * height * 4;
	// var usub = new Uint8ClampedArray(imports.env.memory.buffer, offset, byteSize);
	var usub = new Uint8ClampedArray(instance.exports.memory.buffer, offset, byteSize);
	instance.exports.i();
	img = new ImageData(usub, width, height);
	r();
	animationLoop(0);
}).catch(e => {
	console.log(e);
});

let locked = false;
function changeCallback() {
	locked = document.pointerLockElement === canvas || document.mozPointerLockElement === canvas || document.webkitPointerLockElement === canvas;
}

let mx = 0, my = 0, rx = 0, ry = 0;
function moveCallback(e) {
	rx += e.movementX || e.mozMovementX || e.webkitMovementX || 0;
	ry += e.movementY || e.mozMovementY || e.webkitMovementY || 0;

	let rect = canvas.getBoundingClientRect();
	mx = e.clientX - rect.left;
	my = e.clientY - rect.top;
}
let md = 0, mu = 0;
function mouseDown() {
	md=1;
	if(!locked) rpl();
}
function mouseUp() {
	mu=1;
}

function lockError(e) {
	changeCallback();
}

// Hook pointer lock state change events
document.addEventListener('pointerlockchange', changeCallback, false);
document.addEventListener('mozpointerlockchange', changeCallback, false);
document.addEventListener('webkitpointerlockchange', changeCallback, false);

// Hook mouse move events
document.addEventListener("mousemove", moveCallback, false);

document.addEventListener("mousedown", mouseDown, false);
document.addEventListener("mouseup", mouseUp, false);

document.addEventListener('pointerlockerror', lockError, false);
document.addEventListener('mozpointerlockerror', lockError, false);
