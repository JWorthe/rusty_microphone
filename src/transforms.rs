extern crate dft;

#[derive(Clone, Debug)]
pub struct FrequencyBucket {
    pub min_freq: f64,
    pub max_freq: f64,
    pub intensity: f64
}

pub fn fft(input: Vec<f64>, sample_rate: f64) -> Vec<FrequencyBucket> {
    let frames = input.len();
    let plan = dft::Plan::new(dft::Operation::Forward, frames);
    let mut intensities = input.clone();
    dft::transform(&mut intensities, &plan);

    let frequency_resolution = sample_rate / 2.0 / frames as f64;
    
    intensities.iter().enumerate().map(|(index, &value)| {
        let index = index as f64;
        FrequencyBucket {
            min_freq: index * frequency_resolution,
            max_freq: (index+1.0) * frequency_resolution,
            intensity: value
        }
    }).collect()
}

pub fn find_fundamental_frequency(frequency_domain: &Vec<FrequencyBucket>) -> Option<f64> {
    //TODO look at all significant frequencies, find fundamental
    //TODO return None is none of them are significant
    
    let max_frequency = frequency_domain.iter()
        .fold(None as Option<::transforms::FrequencyBucket>, |max, next|
              if max.is_none() || max.clone().unwrap().intensity < next.intensity { Some(next.clone()) } else { max }
        ).unwrap().max_freq;
    
    Some(max_frequency)
}

#[test]
fn fft_on_sine_wave() {
    use std::f64::consts;
    
    let sample_rate = 44100.0 as f64;
    let amplitude = 1.0 as f64;
    let frames = 16384;
    let frequency = 10000.0 as f64; //10KHz
    let frequency_resolution = sample_rate / 2.0 / frames as f64;
    
    let samples = (0..frames)
        .map(|x| {
            let t = x as f64 / sample_rate;
            (2.0 as f64 * consts::PI * frequency * t).sin() * amplitude
        }).collect();

    let result = fft(samples, sample_rate);
    let fundamental = find_fundamental_frequency(&result);

    assert!((fundamental-frequency).abs() < frequency_resolution, "expected={}, actual={}", frequency, fundamental);
}

pub fn hz_to_pitch(hz: f64) -> String {
    let pitch_names = [
        "C",
        "C#",
        "D",
        "Eb",
        "E",
        "F",
        "F#",
        "G",
        "G#",
        "A",
        "Bb",
        "B"
    ];

    let midi_number = 69.0 + 12.0 * (hz / 440.0).log2();
    //midi_number of 0 is C-1.

    let rounded_pitch = midi_number.round() as usize;
    let name = pitch_names[rounded_pitch%pitch_names.len()].to_string();
    let octave = rounded_pitch / pitch_names.len() - 1; //0 is C-1

    let mut cents = ((midi_number * 100.0).round() % 100.0) as i32;
    if cents >= 50 {
        cents -= 100;
    }
    
    format!("{}{} {:+}", name, octave, cents)
}

#[test]
fn a4_is_correct() {
    assert_eq!(hz_to_pitch(440.0), "A4 +0");
}

#[test]
fn a2_is_correct() {
    assert_eq!(hz_to_pitch(110.0), "A2 +0");
}

#[test]
fn c4_is_correct() {
    assert_eq!(hz_to_pitch(261.63), "C4 +0");
}

#[test]
fn f5_is_correct() {
    assert_eq!(hz_to_pitch(698.46), "F5 +0");
}
