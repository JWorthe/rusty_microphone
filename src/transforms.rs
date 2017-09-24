fn remove_mean_offset(signal: &[f32]) -> Vec<f32> {
    let mean = signal.iter().sum::<f32>()/signal.len() as f32;
    signal.iter().map(|x| x - mean).collect()
}

pub fn correlation(signal: &[f32]) -> Vec<f32> {
    (0..signal.len()).map(|offset| {
        signal.iter().take(signal.len() - offset)
            .zip(signal.iter().skip(offset))
            .map(|(sig_i, sig_j)| sig_i * sig_j)
            .sum()
    }).collect()
}

pub fn find_fundamental_frequency(signal: &[f32], sample_rate: f32) -> Option<f32> {
    let normalized_signal = remove_mean_offset(signal);
    
    if normalized_signal.iter().all(|&x| x.abs() < 0.05) {
        // silence
        return None;
    }
    
    let correlation = correlation(&normalized_signal);

    let first_peak_end = match correlation.iter().position(|&c| c < 0.0) {
        Some(p) => p,
        None => {
            // musical signals will drop below 0 at some point
            return None
        }
    };
    
    let peak = correlation.iter()
        .enumerate()
        .skip(first_peak_end)
        .fold((first_peak_end, 0.0), |(xi, xmag), (yi, &ymag)| if ymag > xmag { (yi, ymag) } else { (xi, xmag) });

    let (peak_index, _) = peak;

    let refined_peak_index = refine_fundamentals(&correlation, peak_index as f32 - 0.5, peak_index as f32 + 0.5);

    if is_noise(&correlation, refined_peak_index) {
        None
    }
    else {
        Some(sample_rate / refined_peak_index)
    }
}

fn refine_fundamentals(correlation: &[f32], low_bound: f32, high_bound: f32) -> f32 {
    let data_points = 2 * correlation.len() / high_bound.ceil() as usize;
    let range = high_bound - low_bound;
    let midpoint = (low_bound + high_bound) / 2.0;
    
    if (range * data_points as f32) < 1.0 {
        midpoint
    }
    else {
        let low_guess = score_guess(correlation, low_bound, data_points);
        let high_guess = score_guess(correlation, high_bound, data_points);
        
        if high_guess > low_guess {
            refine_fundamentals(correlation, midpoint, high_bound)
        }
        else {
            refine_fundamentals(correlation, low_bound, midpoint)
        }
    }
}


fn is_noise(correlation: &[f32], fundamental: f32) -> bool {
    let value_at_point = interpolate(correlation, fundamental);
    let score_data_points = 2 * correlation.len() / fundamental.ceil() as usize;
    let score = score_guess(correlation, fundamental, score_data_points);

    value_at_point > 2.0*score
}

fn score_guess(correlation: &[f32], period: f32, data_points: usize) -> f32 {
    (1..data_points).map(|i| {
        let expected_sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        let x = i as f32 * period / 2.0;
        let weight = 0.5 * i as f32;
        expected_sign * weight * interpolate(correlation, x)
    }).sum()
}

fn interpolate(correlation: &[f32], x: f32) -> f32 {
    if x.floor() < 0.0 {
        correlation[0]
    }
    else if x.ceil() >= correlation.len() as f32 {
        correlation[correlation.len()-1]
    }
    else {
        let x0 = x.floor();
        let y0 = correlation[x0 as usize];
        let x1 = x.ceil();
        let y1 = correlation[x1 as usize];

        if x0 as usize == x1 as usize {
            y0
        }
        else {
            (y0*(x1-x) + y1*(x-x0)) / (x1-x0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    
    const SAMPLE_RATE: f32 = 44100.0;
    const FRAMES: usize = 512;

    fn frequency_resolution() -> f32 {
        SAMPLE_RATE / 2.0 / FRAMES as f32
    }

    fn sin_arg(f: f32, t: f32, phase: f32) -> f32 {
        2.0 as f32 * PI * f * t + phase
    }

    fn sample_sinusoud(amplitude: f32, frequency: f32, phase: f32) -> Vec<f32> {
        (0..FRAMES)
            .map(|x| {
                let t = x as f32 / SAMPLE_RATE;
                sin_arg(frequency, t, phase).sin() * amplitude
            }).collect()
    }
    
    #[test]
    fn correlation_on_sine_wave() {
        let frequency = 440.0 as f32; //concert A
        
        let samples = sample_sinusoud(1.0, frequency, 0.0);
        let fundamental = find_fundamental_frequency(&samples, SAMPLE_RATE).expect("Find fundamental returned None");
        assert!((fundamental-frequency).abs() < frequency_resolution(), "expected={}, actual={}", frequency, fundamental);
    }

    #[test]
    fn correlation_on_two_sine_waves() {
        //Unfortunately, real signals won't be this neat
        let samples1a = sample_sinusoud(2.0, 440.0, 0.0);
        let samples2a = sample_sinusoud(1.0, 880.0, 0.0);
        let expected_fundamental = 440.0;
        
        let samples: Vec<f32> = samples1a.iter().zip(samples2a.iter())
            .map(|(a, b)| a+b)
            .collect();

        let fundamental = find_fundamental_frequency(&samples, SAMPLE_RATE).expect("Find fundamental returned None");

        assert!((fundamental-expected_fundamental).abs() < frequency_resolution(), "expected_fundamental={}, actual={}", expected_fundamental, fundamental);
    }

    #[test]
    fn interpolate_half_way() {
        assert_eq!(0.5, interpolate(&vec!(0.0, 1.0), 0.5))
    }
}

fn hz_to_midi_number(hz: f32) -> f32 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

pub fn hz_to_cents_error(hz: f32) -> f32 {
    let midi_number = hz_to_midi_number(hz);
    let cents = (midi_number % 1.0) * 100.0;
    if cents >= 50.0 {
        cents - 100.0
    }
    else {
        cents
    }
}

pub fn hz_to_pitch(hz: f32) -> String {
    let pitch_names = [
        "C",
        "C♯",
        "D",
        "E♭",
        "E",
        "F",
        "F♯",
        "G",
        "G♯",
        "A",
        "B♭",
        "B"
    ];

    let midi_number = hz_to_midi_number(hz);
    //midi_number of 0 is C-1.

    let rounded_pitch = midi_number.round() as i32;
    let name = pitch_names[rounded_pitch as usize % pitch_names.len()];
    let octave = rounded_pitch / pitch_names.len() as i32 - 1; //0 is C-1
    if octave < 0 {
        return "< C 1".to_string();
    }

    format!("{: <2}{}", name, octave)
}

#[test]
fn a4_is_correct() {
    assert_eq!(hz_to_pitch(440.0), "A 4");
}

#[test]
fn a2_is_correct() {
    assert_eq!(hz_to_pitch(110.0), "A 2");
}

#[test]
fn c4_is_correct() {
    assert_eq!(hz_to_pitch(261.63), "C 4");
}

#[test]
fn f5_is_correct() {
    assert_eq!(hz_to_pitch(698.46), "F 5");
}


pub fn align_to_rising_edge(samples: &[f32]) -> Vec<f32> {
    remove_mean_offset(samples)
        .iter()
        .skip_while(|x| !x.is_sign_negative())
        .skip_while(|x| x.is_sign_negative())
        .cloned()
        .collect()
}


#[cfg(target_os = "emscripten")]
pub mod emscripten_api {
    #[no_mangle]
    pub extern "C" fn find_fundamental_frequency(signal: *const f32, signal_length: isize, sample_rate: f32) -> f32 {
        use std::slice;
        let signal_slice = unsafe {
            &slice::from_raw_parts(signal, signal_length as usize)
        };
        
        println!("Signal is {:?}", signal_slice);
        println!("Sample rate is {:?}Hz", sample_rate);
        super::find_fundamental_frequency(&signal_slice, sample_rate).unwrap_or(0.0)
    }
}
