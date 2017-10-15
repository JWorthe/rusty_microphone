// This is read and used by `site.js`
var Module = {
    noInitialRun: true,
    noExitRuntime: true,
    onRuntimeInitialized: main
};

function jsArrayToF32ArrayPtr(jsArray, callback) {
    var data = new Float32Array(jsArray);
    var nDataBytes = data.length * data.BYTES_PER_ELEMENT;
    var dataPtr = Module._malloc(nDataBytes);

    var dataHeap = new Uint8Array(Module.HEAPU8.buffer, dataPtr, nDataBytes);
    dataHeap.set(new Uint8Array(data.buffer));

    var result = callback(dataHeap.byteOffset, jsArray.length);

    Module._free(dataPtr);
    
    return result;
}

function findFundamentalFrequency(data, samplingRate) {
    return jsArrayToF32ArrayPtr(data, function(dataPtr, dataLength) {
        return Module._find_fundamental_frequency(dataPtr, dataLength, samplingRate);
    });
}

function hzToCentsError(hz) {
    return Module._hz_to_cents_error(hz);
}

function hzToPitch(hz) {
    var wrapped = Module.cwrap('hz_to_pitch', 'string', ['number']);
    return wrapped(hz);
}

function main() {
    var data = [1, 0, -1, 0, 1, 0, -1, 0, 1, 0, -1, 0, 1, 0, -1, 0];
    var fundamental = findFundamentalFrequency(data, 44100.0);

    var error = hzToCentsError(450.0);
    var pitch = hzToPitch(450.0);
    
    console.log("Javascript here. Our fundamental frequency according to Rust is " + fundamental + "Hz");
    console.log("The other math shows a pitch of " + pitch + ", and an error of " + error);
}
