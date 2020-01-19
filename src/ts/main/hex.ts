var hex1 = (a:any) => new Uint8Array(a.match(/.{1,2}/g).map((i:any)=>parseInt(i,16)));

var hex2 = (a:any) => {
  var b = [];
  for(var i = 0; i < a.length; i+=2)
    b.push(parseInt(a.substr(i,2),16));
  
  return new Uint8Array(b);
}