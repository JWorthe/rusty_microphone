extern crate dft;

#[derive(Clone, Debug)]
pub struct FrequencyBucket {
    pub min_freq: f64,
    pub max_freq: f64,
    pub intensity: f64
}

impl FrequencyBucket {
    pub fn ave_freq(&self) -> f64 {
        (self.min_freq + self.max_freq) / 2.0
    }
}

pub fn fft(input: &Vec<f64>, sample_rate: f64) -> Vec<FrequencyBucket> {
    let frames = input.len();
    let mean_input = input.iter().sum::<f64>()/input.len() as f64;
    
    let mut intensities = input.iter().map(|x|x-mean_input).collect::<Vec<_>>();
    let plan = dft::Plan::new(dft::Operation::Forward, frames);
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
    let positive_buckets = frequency_domain.iter().filter(|x| x.intensity > 0.0).cloned().collect::<Vec<_>>();
    let average_intensity = positive_buckets.iter().map(|x| x.intensity).sum::<f64>() / frequency_domain.len() as f64;
    let significant_buckets = positive_buckets.iter().filter(|x| x.intensity > average_intensity).cloned().collect::<Vec<_>>();
    
    let max_bucket = significant_buckets.iter()
        .fold(None as Option<::transforms::FrequencyBucket>, |max, next|
              if max.is_none() || max.clone().unwrap().intensity < next.intensity { Some(next.clone()) } else { max }
        ).unwrap();

    Some(max_bucket.ave_freq())
}

pub fn correlation(input: &Vec<f64>) -> Vec<f64> {
    let mut correlation = Vec::with_capacity(input.len());
    for offset in 0..input.len() {
        let mut c = 0.0;
        for i in 0..input.len()-offset {
            let j = i+offset;
            c += input[i] * input[j];
        }
        correlation.push(c);
    }
    correlation
}

pub fn find_fundamental_frequency_correlation(input: &Vec<f64>, sample_rate: f64) -> Option<f64> {
    let correlation = correlation(&input);

    let mut first_peak_width = 0;
    for offset in 0..correlation.len() {
        if correlation[offset] < 0.0 {
            first_peak_width = offset;
            break;
        }
    }
    if first_peak_width == 0 {
        return None;
    }

    let peak = correlation.iter()
        .enumerate()
        .skip(first_peak_width)
        .fold((first_peak_width, 0.0 as f64), |(xi, xmag), (yi, &ymag)| if ymag > xmag { (yi, ymag) } else { (xi, xmag) });

    let (peak_index, _) = peak;
    Some(sample_rate / peak_index as f64)
}

fn interpolate(correlation: &Vec<f64>, x: f64) -> f64 {
    if x < 0.0 {
        println!("<0");
        correlation[0]
    }
    else if x >= correlation.len() as f64 {
        println!(">len");
        correlation[correlation.len()-1]
    }
    else {
        let x0 = x.floor();
        let y0 = correlation[x0 as usize];
        let x1 = x.ceil();
        let y1 = correlation[x1 as usize];

        (y0*(x1-x) + y1*(x-x0)) / (x1-x0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    
    const SAMPLE_RATE: f64 = 44100.0;
    const FRAMES: usize = 512;

    fn frequency_resolution() -> f64 {
        SAMPLE_RATE / 2.0 / FRAMES as f64
    }

    fn sin_arg(f: f64, t: f64, phase: f64) -> f64 {
        2.0 as f64 * PI * f * t + phase
    }

    fn sample_sinusoud(amplitude: f64, frequency: f64, phase: f64) -> Vec<f64> {
        (0..FRAMES)
            .map(|x| {
                let t = x as f64 / SAMPLE_RATE;
                sin_arg(frequency, t, phase).sin() * amplitude
            }).collect()
    }
    
    #[test]
    fn fft_on_sine_wave() {
        let frequency = 440.0 as f64; //concert A
        
        let samples = sample_sinusoud(1.0, frequency, 0.0);
        let frequency_domain = fft(&samples, SAMPLE_RATE);
        let fundamental = find_fundamental_frequency(&frequency_domain).unwrap();

        assert!((fundamental-frequency).abs() < frequency_resolution(), "expected={}, actual={}", frequency, fundamental);
    }

    #[test]
    fn fft_on_two_sine_waves() {
        //Unfortunately, real signals won't be this neat
        let samples1a = sample_sinusoud(2.0, 440.0, 0.0);
        let samples2a = sample_sinusoud(1.0, 880.0, 0.0);
        let expected_fundamental = 440.0;
        
        let samples = samples1a.iter().zip(samples2a.iter())
            .map(|(a, b)| a+b)
            .collect();
        let frequency_domain = fft(&samples, SAMPLE_RATE);
        
        let fundamental = find_fundamental_frequency(&frequency_domain).unwrap();

        assert!((fundamental-expected_fundamental).abs() < frequency_resolution(), "expected_fundamental={}, actual={}", expected_fundamental, fundamental);
    }

    #[test]
    fn correlation_on_sine_wave() {
        let frequency = 440.0 as f64; //concert A
        
        let samples = sample_sinusoud(1.0, frequency, 0.0);
        let fundamental = find_fundamental_frequency_correlation(&samples, SAMPLE_RATE).unwrap();
        assert!((fundamental-frequency).abs() < frequency_resolution(), "expected={}, actual={}", frequency, fundamental);
    }

    #[test]
    fn correlation_on_two_sine_waves() {
        //Unfortunately, real signals won't be this neat
        let samples1a = sample_sinusoud(2.0, 440.0, 0.0);
        let samples2a = sample_sinusoud(1.0, 880.0, 0.0);
        let expected_fundamental = 440.0;
        
        let samples = samples1a.iter().zip(samples2a.iter())
            .map(|(a, b)| a+b)
            .collect();

        let fundamental = find_fundamental_frequency_correlation(&samples, SAMPLE_RATE).unwrap();

        assert!((fundamental-expected_fundamental).abs() < frequency_resolution(), "expected_fundamental={}, actual={}", expected_fundamental, fundamental);
    }

    #[test]
    fn interpolate_half_way() {
        assert_eq!(0.5, interpolate(&vec!(0.0, 1.0), 0.5))
    }
}

pub fn hz_to_midi_number(hz: f64) -> f64 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

pub fn hz_to_cents_error(hz: f64) -> f64 {
    let midi_number = hz_to_midi_number(hz);
    let cents = (midi_number * 100.0).round() % 100.0;
    if cents >= 50.0 {
        cents - 100.0
    }
    else {
        cents
    }
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

    let midi_number = hz_to_midi_number(hz);
    //midi_number of 0 is C-1.

    let rounded_pitch = midi_number.round() as i32;
    let name = pitch_names[rounded_pitch as usize % pitch_names.len()].to_string();
    let octave = rounded_pitch / pitch_names.len() as i32 - 1; //0 is C-1
    if octave < 0 {
        return "< C1".to_string();
    }

    let mut cents = ((midi_number * 100.0).round() % 100.0) as i32;
    if cents >= 50 {
        //don't need to adjust pitch here, that's already been done by the round above
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


pub fn align_to_rising_edge(samples: &Vec<f64>) -> Vec<f64> {
    samples.iter()
        .skip_while(|x| !x.is_sign_negative())
        .skip_while(|x| x.is_sign_negative())
        .cloned()
        .collect()
}
