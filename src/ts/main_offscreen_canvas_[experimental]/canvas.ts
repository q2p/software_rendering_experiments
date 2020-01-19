const width = 192;
const height = 108;
var scale = -1;
const canvas = document.getElementsByTagName("canvas")[0];
const offscreenCanvas = (<any>canvas).transferControlToOffscreen();
canvas.width = width;
canvas.height = height;
function resize() {
	var dw = document.body.clientWidth;
	var dh = document.body.clientHeight;

	if((dw * height) / (dh * width) >= 1)
		scale = Math.max(1, Math.floor(dh / height));
	else
		scale = Math.max(1, Math.floor(dw / width));
	
	canvas.style.width  = (width*scale)+"px";
	canvas.style.height = (height*scale)+"px";
}
document.body.addEventListener("onresize", resize, false);
resize();

const worker = new Worker('worker.js');

worker.postMessage({i: offscreenCanvas}, [offscreenCanvas]);