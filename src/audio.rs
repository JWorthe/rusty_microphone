extern crate portaudio;
use portaudio as pa;

const SAMPLE_RATE: f64 = 44100.0;
const FRAMES: usize = 512;

pub fn get_device_list() -> Result<Vec<(u32, String)>, pa::Error> {
    let pa = try!(pa::PortAudio::new());
    let default_host = try!(pa.default_host_api());
    println!("Using default host: {:#?}", default_host);

    let mut list = Vec::new();
    let devices = try!(pa.devices());
    for device in devices {
        let (pa::DeviceIndex(idx), info) = try!(device);
        if info.max_input_channels == 0 {
            continue;
        }
        list.push((idx, info.name.to_string()));
    }
    Ok(list)
}

pub fn run(device_index: u32) -> Result<(), pa::Error> {
    let pa = try!(pa::PortAudio::new());

    let input_info = try!(pa.device_info(pa::DeviceIndex(device_index)));
    println!("Using {} for input", input_info.name);

    // Construct the input stream parameters.
    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(pa::DeviceIndex(device_index), 1, true, latency);

    // Check that the stream format is supported.
    try!(pa.is_input_format_supported(input_params, SAMPLE_RATE));

    // Construct the settings with which we'll open our duplex stream.
    let settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES as u32);

    // We'll use this channel to send the count_down to the main thread for fun.
    let (sender, receiver) = ::std::sync::mpsc::channel();

    // A callback to pass to the non-blocking stream.
    let callback = move |pa::InputStreamCallbackArgs { buffer, .. }| {
        sender.send(buffer.iter().map(|x| *x as f64).collect()).ok();
        pa::Continue
    };

    // Construct a stream with input and output sample types of f32.
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));

    try!(stream.start());

    let mut samples_index = 0;
    // Do some stuff!
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

    try!(stream.stop());

    Ok(())
}
