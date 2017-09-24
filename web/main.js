// This is read and used by `site.js`
var Module = {
    noInitialRun: true,
    noExitRuntime: true,
    onRuntimeInitialized: main
};

function jsArrayToF32ArrayPtr(jsArray) {
    var data = new Float32Array(jsArray);
    var nDataBytes = data.length * data.BYTES_PER_ELEMENT;
    var dataPtr = Module._malloc(nDataBytes);

    var dataHeap = new Uint8Array(Module.HEAPU8.buffer, dataPtr, nDataBytes);
    dataHeap.set(new Uint8Array(data.buffer));
    return dataHeap.byteOffset;
}

function main() {
    var data = [1, 0, -1, 0, 1, 0, -1, 0, 1, 0, -1, 0, 1, 0, -1, 0];
    var dataLength = data.length;
    var ptr = jsArrayToF32ArrayPtr(data);

    var fundamental = _find_fundamental_frequency(ptr, data.length, 44100.0);
    console.log("Javascript here. Our fundamental frequency according to Rust is " + fundamental + "Hz");
}
