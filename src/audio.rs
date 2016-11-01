extern crate portaudio;
use portaudio as pa;

use std::sync::mpsc::*;

const SAMPLE_RATE: f64 = 44100.0;
const FRAMES: usize = 512;

pub fn init() -> Result<pa::PortAudio, pa::Error> {
    pa::PortAudio::new()
}

pub fn get_device_list(pa: &pa::PortAudio) -> Result<Vec<(u32, String)>, pa::Error> {
    // This pa.devices gives a Result of devices, each of which is
    // also a Result. So a Result<Collection<Result<Device>>>. We
    // mould it into devices: a Vec<(index, DeviceInfo)>.
    let devices = try!(try!(pa.devices()).map(|device| {
        device.map(|(pa::DeviceIndex(idx), info)| (idx, info))
    }).collect::<Result<Vec<_>, _>>());
    
    let list = devices.iter().filter(|&&(_, ref info)| info.max_input_channels > 0)
        .map(|&(idx, ref info)| (idx, info.name.to_string()))
        .collect();
    Ok(list)
}

#[test]
fn get_device_list_returns_devices() {
    let pa = init().expect("Could not init portaudio");
    let devices = get_device_list(&pa).expect("Getting devices had an error");

    // all machines should have at least one input stream, even if
    // that's just a virtual stream with a name like "default".
    assert!(devices.len() > 0);
}


pub struct OpenRecordingChannel {
    pub receiver: Receiver<Vec<f32>>,
    pub stream: pa::Stream<pa::NonBlocking, pa::Input<f32>>
}

pub fn start_listening(pa: &pa::PortAudio, device_index: u32) -> Result<OpenRecordingChannel, pa::Error> {
    let device_info = try!(pa.device_info(pa::DeviceIndex(device_index)));
    let latency = device_info.default_low_input_latency;

    // Construct the input stream parameters.

    let input_params = pa::StreamParameters::<f32>::new(pa::DeviceIndex(device_index), 1, true, latency);

    // Check that the stream format is supported.
    try!(pa.is_input_format_supported(input_params, SAMPLE_RATE));

    // Construct the settings with which we'll open our stream.
    let stream_settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES as u32);

    // This channel will let us read from and control the audio stream
    let (sender, receiver) = channel();

    // This callback A callback to pass to the non-blocking stream.
    let callback = move |pa::InputStreamCallbackArgs { buffer, .. }| {
        sender.send(buffer.iter().cloned().collect()).ok();
        pa::Continue
    };

    let mut stream = try!(pa.open_non_blocking_stream(stream_settings, callback));
    try!(stream.start());
    
    Ok(OpenRecordingChannel {
        receiver: receiver,
        stream: stream
    })
}

#[test]
fn start_listening_returns_successfully() {
    let pa = init().expect("Could not init portaudio");
    let devices = get_device_list(&pa).expect("Getting devices had an error");
    let device = devices.first().expect("Should have at least one device");
    start_listening(&pa, device.0).expect("Error starting listening to first channel");
}


/*
   let mut samples_index = 0;
    while let Ok(samples) = receiver.recv() {
        samples_index += 1;
        if samples_index % 100 != 0 {
            continue;
        }

        let frequency_domain = ::transforms::fft(samples, SAMPLE_RATE);
            
        let max_frequency = frequency_domain.iter()
            .fold(None as Option<::transforms::FrequencyBucket>, |max, next|
                  if max.is_none() || max.clone().unwrap().intensity < next.intensity { Some(next.clone()) } else { max }
            ).unwrap().max_freq;
        println!("{}Hz", max_frequency.floor());
    }
*/
