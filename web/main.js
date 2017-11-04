// This is read and used by `site.js`
var Module = {
    noInitialRun: true,
    noExitRuntime: true,
    onRuntimeInitialized: main
};

function jsArrayToF32ArrayPtr(jsArray, callback) {
    var data = (jsArray instanceof Float32Array) ? jsArray : new Float32Array(jsArray);
    var nDataBytes = data.length * data.BYTES_PER_ELEMENT;
    var dataPtr = Module._malloc(nDataBytes);

    var dataHeap = new Uint8Array(Module.HEAPU8.buffer, dataPtr, nDataBytes);
    dataHeap.set(new Uint8Array(data.buffer));

    var result = callback(dataPtr, jsArray.length);

    Module._free(dataPtr);
    
    return result;
}

function jsArrayToF32ArrayPtrMutateInPlace(jsArray, mutate) {
    var data = new Float32Array(jsArray);
    var nDataBytes = data.length * data.BYTES_PER_ELEMENT;
    var dataPtr = Module._malloc(nDataBytes);

    var dataHeap = new Uint8Array(Module.HEAPU8.buffer, dataPtr, nDataBytes);
    dataHeap.set(new Uint8Array(data.buffer));

    mutate(dataPtr, jsArray.length);

    var mutatedData = new Float32Array(Module.HEAPU8.buffer, dataPtr, jsArray.length);
    var result = Array.prototype.slice.call(mutatedData);
    
    Module._free(dataPtr);
    
    return result;
}

function findFundamentalFrequency(data, samplingRate) {
    return jsArrayToF32ArrayPtr(data, function(dataPtr, dataLength) {
        return Module._find_fundamental_frequency(dataPtr, dataLength, samplingRate);
    });
}

var nDataBytes = null;
var dataPtr = null;
var dataHeap = null;
function findFundamentalFrequencyNoFree(data, samplingRate) {
    var length = Math.min(data.length, 512);
    //assume data is already a Float32Array and its length won't change from call to call
    if (!dataPtr) {
        nDataBytes = length * data.BYTES_PER_ELEMENT;
        dataPtr = Module._malloc(nDataBytes);
        dataHeap = new Uint8Array(Module.HEAPU8.buffer, dataPtr, nDataBytes);
    }
    dataHeap.set(new Uint8Array(data.buffer, data.buffer.byteLength - nDataBytes));
    return Module._find_fundamental_frequency(dataPtr, length, samplingRate);    
}


function hzToCentsError(hz) {
    return Module._hz_to_cents_error(hz);
}

var hzToPitch = function(hz) {
    var wrapped = Module.cwrap('hz_to_pitch', 'string', ['number']);
    hzToPitch = wrapped;
    return wrapped(hz);
};

function correlation(data) {
    return jsArrayToF32ArrayPtrMutateInPlace(data, function(dataPtr, dataLength) {
        Module._correlation(dataPtr, dataLength);
    });
}

function update(signal, sampleRate) {
    var fundamental = findFundamentalFrequencyNoFree(signal, sampleRate);

    var pitch = hzToPitch(fundamental);
    var error = hzToCentsError(fundamental);

    document.getElementById('pitch-label').innerHTML = pitch;
    if (error > 0) {
        document.getElementById('pitch-error-direction').innerHTML = 'sharp';
        document.getElementById('pitch-error').innerHTML = error;
    } else {
        document.getElementById('pitch-error-direction').innerHTML = 'flat';
        document.getElementById('pitch-error').innerHTML = -error;
    }    
}

function draw(dataArray, canvas, canvasCtx) {
    // This draw example is currently heavily based on an example
    // from MDN:
    // https://developer.mozilla.org/en-US/docs/Web/API/AnalyserNode
    var bufferLength = Math.min(dataArray.length, 512);

    canvasCtx.fillStyle = 'rgb(200, 200, 200)';
    canvasCtx.fillRect(0, 0, canvas.width, canvas.height);

    canvasCtx.lineWidth = 2;
    canvasCtx.strokeStyle = 'rgb(0, 0, 0)';

    canvasCtx.beginPath();

    var sliceWidth = canvas.width * 1.0 / bufferLength;
    var x = 0;

    for (var i = 0; i < bufferLength; i++) {
        var y = (dataArray[i] * canvas.height / 2) + canvas.height / 2;

        if (i === 0) {
            canvasCtx.moveTo(x, y);
        } else {
            canvasCtx.lineTo(x, y);
        }

        x += sliceWidth;
    }

    canvasCtx.stroke();
};

function main() {
    var canvas = document.getElementById("oscilloscope");
    var canvasCtx = canvas.getContext("2d");
    
    navigator.mediaDevices.getUserMedia({ audio: true })
        .then(function(stream) {
            var context = new window.AudioContext();
            var input = context.createMediaStreamSource(stream);
            var analyser = context.createAnalyser();
            input.connect(analyser);

            var lastTimestamp = 0;
            var timestampMod = 0;

            function analyserNodeCallback(timestamp) {
                timestampMod += 1;
                if (timestampMod === 100) {
                    timestampMod = 0;
                    var dt = timestamp - lastTimestamp;
                    lastTimestamp = timestamp;
                    var framerate = 100000/dt;
                    document.getElementById('frame-rate').innerHTML = framerate.toFixed(2) + 'Hz';
                }
                
                var dataArray = new Float32Array(analyser.fftSize);
                analyser.getFloatTimeDomainData(dataArray);
                update(dataArray, context.sampleRate);
                draw(dataArray, canvas, canvasCtx);
                window.requestAnimationFrame(analyserNodeCallback);
            }

            window.requestAnimationFrame(analyserNodeCallback);
        })
        .catch(function(err) {
            console.err('Could not get the microphone');
            console.err(err);
        });
}
