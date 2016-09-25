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

#[test]
fn fft_on_sine_wave() {
    use std::f64::consts;
    
    let sample_rate = 44100.0 as f64;
    let amplitude = 1.0 as f64;
    let frames = 16384;
    let frequency = 10000.0 as f64; //10KHz
    let samples = (0..frames)
        .map(|x| {
            let t = x as f64 / sample_rate;
            (2.0 as f64 * consts::PI * frequency * t).sin() * amplitude
        }).collect();

    let result = fft(samples, sample_rate);

    let peak = result.iter()
        .fold(None as Option<FrequencyBucket>, |max, next|
              if max.is_none() || max.clone().unwrap().intensity < next.intensity { Some(next.clone()) } else { max }
        ).unwrap();

    println!("{:?}", peak);

    assert!(peak.min_freq <= frequency);
    assert!(peak.max_freq >= frequency);
}
