const width = 128;
const height = 128;

var scale = -1;

var instance:WebAssembly.Instance;

const canvas = document.getElementsByTagName("canvas")[0];
function r() {
	var dw = document.body.clientWidth;
	var dh = document.body.clientHeight;

	if((dw * height) / (dh * width) >= 1)
		scale = Math.max(1, Math.floor(dh / height));
	else
		scale = Math.max(1, Math.floor(dw / width));
	
	canvas.style.width  = (width*scale)+"px";
	canvas.style.height = (height*scale)+"px";
}

var ctx:CanvasRenderingContext2D = canvas.getContext("2d");

var md:(timestamp:number)=>void;

var img:ImageData;

function animationLoop(timestamp:number) {
  md(timestamp);
  ctx.putImageData(img, 0, 0);
  requestAnimationFrame(animationLoop);
}

r();
let imports = {
    env: {
        memoryBase: 0,
        tableBase: 0,
        cos: Math.cos,
        __table_base: 0,
        __memory_base: 0,
        memory: new WebAssembly.Memory({ initial: 256 }),
        table: new WebAssembly.Table({ initial: 0, element: 'anyfunc' }),
    }
};

fetch("emc.wasm").then(response =>
  response.arrayBuffer()
).then(bytes => {
  return WebAssembly.instantiate(bytes, imports);
}).then(results => {
  var instance = results.instance;
  md = instance.exports._t;

  let byteSize = width * height * 4;
  var offset = instance.exports._p();

  var usub = new Uint8ClampedArray(imports.env.memory.buffer, offset, byteSize);
  img = new ImageData(usub, width, height);
  animationLoop(0);
}).catch(e => {
    console.log(e);
});