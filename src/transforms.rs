pub fn remove_mean_offset(input: &Vec<f64>) -> Vec<f64> {
    let mean_input = input.iter().sum::<f64>()/input.len() as f64;
    input.iter().map(|x|x-mean_input).collect()
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
    let intensities = remove_mean_offset(&input);
    
    if intensities.iter().all(|&x| x.abs() < 0.1) {
        return None;
    }
    
    let correlation = correlation(&intensities);

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

    let refined_peak_index = refine_fundamentals(&correlation, peak_index as f64);

    if is_noise(&correlation, refined_peak_index) {
        None
    }
    else {
        Some(sample_rate / refined_peak_index)
    }
}

fn refine_fundamentals(correlation: &Vec<f64>, initial_guess: f64) -> f64 {
    let mut low_bound = initial_guess - 0.5;
    let mut high_bound = initial_guess + 0.5;

    for _ in 0..5 {
        let data_points = 2 * correlation.len() / high_bound.ceil() as usize;
        let low_guess = score_guess(&correlation, low_bound, data_points);
        let high_guess = score_guess(&correlation, high_bound, data_points);

        let midpoint = (low_bound + high_bound) / 2.0;
        if high_guess > low_guess {
            low_bound = midpoint;
        }
        else {
            high_bound = midpoint;
        }
    }
    (low_bound + high_bound) / 2.0
}

fn is_noise(correlation: &Vec<f64>, fundamental: f64) -> bool {
    let value_at_point = interpolate(&correlation, fundamental);
    let score_data_points = 2 * correlation.len() / fundamental.ceil() as usize;
    let score = score_guess(&correlation, fundamental, score_data_points);

    value_at_point > 2.0*score
}

fn score_guess(correlation: &Vec<f64>, period: f64, data_points: usize) -> f64 {
    let mut score = 0.0;
    for i in 1..data_points {
        let expected_sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        score += expected_sign * 0.5 * i as f64 * interpolate(&correlation, i as f64 * period / 2.0);
    }
    score
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
    fn correlation_on_sine_wave() {
        let frequency = 440.0 as f64; //concert A
        
        let samples = sample_sinusoud(1.0, frequency, 0.0);
        let fundamental = find_fundamental_frequency_correlation(&samples, SAMPLE_RATE).expect("Find fundamental returned None");
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

        let fundamental = find_fundamental_frequency_correlation(&samples, SAMPLE_RATE).expect("Find fundamental returned None");

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
        "C ",
        "C#",
        "D ",
        "Eb",
        "E ",
        "F ",
        "F#",
        "G ",
        "G#",
        "A ",
        "Bb",
        "B "
    ];

    let midi_number = hz_to_midi_number(hz);
    //midi_number of 0 is C-1.

    let rounded_pitch = midi_number.round() as i32;
    let name = pitch_names[rounded_pitch as usize % pitch_names.len()].to_string();
    let octave = rounded_pitch / pitch_names.len() as i32 - 1; //0 is C-1
    if octave < 0 {
        return "< C 1".to_string();
    }

    format!("{}{}", name, octave)
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


pub fn align_to_rising_edge(samples: &Vec<f64>) -> Vec<f64> {
    remove_mean_offset(&samples)
        .iter()
        .skip_while(|x| !x.is_sign_negative())
        .skip_while(|x| x.is_sign_negative())
        .cloned()
        .collect()
}
