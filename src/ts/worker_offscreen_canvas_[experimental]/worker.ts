var ctx:CanvasRenderingContext2D;

const width = 192;
const height = 108;
var instance:WebAssembly.Instance;
var md = {
  tick: <any>undefined,
  getImage: <any>undefined
};

var img:ImageData;

function animationLoop() {
  md.tick();
  ctx.putImageData(img, 0, 0);
  (<any>ctx).commit().then(animationLoop);
}

self.onmessage = function(message) {
	ctx = message.data.i.getContext("2d");
  fetch("w.wasm").then(response =>
    response.arrayBuffer()
  ).then(bytes =>
    WebAssembly.instantiate(bytes, { env: { cos: Math.cos } })
  ).then(results => {
    var instance = results.instance;
    md = {
      tick: instance.exports.fill,
      getImage: instance.exports.getImage
    };
  
    let byteSize = width * height * 4;
    var offset = md.getImage();
  
    var usub = new Uint8ClampedArray(instance.exports.memory.buffer, offset, byteSize);
    img = new ImageData(usub, width, height);
    animationLoop();
  });
}