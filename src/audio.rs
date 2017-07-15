extern crate portaudio;
use portaudio as pa;

use std::sync::mpsc::*;

pub const SAMPLE_RATE: f32 = 44100.0;
pub const FRAMES: usize = 512;

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

pub fn get_default_device(pa: &pa::PortAudio) -> Result<u32, pa::Error> {
    let pa::DeviceIndex(default_input_index) = pa.default_input_device()?;
    Ok(default_input_index)
}

pub fn start_listening_default(pa: &pa::PortAudio, sender: Sender<Vec<f32>>) -> Result<pa::Stream<pa::NonBlocking, pa::Input<f32>>, pa::Error> {
    let default = get_default_device(&pa)?;
    start_listening(&pa, default, sender)
}

pub fn start_listening(pa: &pa::PortAudio, device_index: u32,
                       sender: Sender<Vec<f32>>) -> Result<pa::Stream<pa::NonBlocking, pa::Input<f32>>, pa::Error> {
    let device_info = try!(pa.device_info(pa::DeviceIndex(device_index)));
    let latency = device_info.default_low_input_latency;

    // Construct the input stream parameters.

    let input_params = pa::StreamParameters::<f32>::new(pa::DeviceIndex(device_index), 1, true, latency);

    // Check that the stream format is supported.
    try!(pa.is_input_format_supported(input_params, SAMPLE_RATE as f64));

    // Construct the settings with which we'll open our stream.
    let stream_settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE as f64, FRAMES as u32);

    // This callback A callback to pass to the non-blocking stream.
    let callback = move |pa::InputStreamCallbackArgs { buffer, .. }| {
        match sender.send(Vec::from(buffer)) {
            Ok(_) => pa::Continue,
            Err(_) => pa::Complete //this happens when receiver is dropped
        }
    };

    let mut stream = try!(pa.open_non_blocking_stream(stream_settings, callback));
    try!(stream.start());
    
    Ok(stream)
}

#[test]
#[ignore] //ignored because TravisCI doesn't have any audio devices to test with
fn start_listening_returns_successfully() {
    // Just a note on unit tests here, portaudio doesn't seem to
    // respond well to being initialized many times, and starts
    // throwing errors.
    let pa = init().expect("Could not init portaudio");

    let devices = get_device_list(&pa).expect("Getting devices had an error");
    assert!(devices.len() > 0);
    
    let (sender, _) = channel();
    start_listening_default(&pa, sender).expect("Error starting listening to first channel");
}
