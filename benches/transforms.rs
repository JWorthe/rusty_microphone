#[macro_use]
extern crate bencher;

extern crate rusty_microphone;

use bencher::Bencher;

use std::f32::consts::PI;

const SAMPLE_RATE: f32 = 44100.0;
const FRAMES: usize = 1024;


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

fn bench_correlation_on_sine_wave(b: &mut Bencher) {
    let frequency = 440.0 as f32; //concert A
    let samples = sample_sinusoud(1.0, frequency, 0.0);
    
    b.iter(|| {
        rusty_microphone::transforms::find_fundamental_frequency_correlation(&samples, SAMPLE_RATE)
    })
}
benchmark_group!(transforms, bench_correlation_on_sine_wave);
benchmark_main!(transforms);
