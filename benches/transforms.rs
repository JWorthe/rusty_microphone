#[macro_use]
extern crate bencher;

extern crate rusty_microphone;
use rusty_microphone::signal::Signal;
use rusty_microphone::correlation::Correlation;

use bencher::Bencher;

use std::f32::consts::PI;

const SAMPLE_RATE: f32 = 44100.0;
const FRAMES: u16 = 512;


fn sin_arg(f: f32, t: f32, phase: f32) -> f32 {
    2.0 as f32 * PI * f * t + phase
}

fn sample_sinusoud(amplitude: f32, frequency: f32, phase: f32) -> Vec<f32> {
    (0..FRAMES)
        .map(|x| {
            let t = f32::from(x) / SAMPLE_RATE;
            sin_arg(frequency, t, phase).sin() * amplitude
        }).collect()
}

fn bench_correlation_on_sine_wave(b: &mut Bencher) {
    let signal = Signal::new(
        &sample_sinusoud(1.0, 440.0f32, 0.0),
        SAMPLE_RATE
    );
    
    b.iter(|| {
        Correlation::from_signal(&signal);
    })
}
benchmark_group!(transforms, bench_correlation_on_sine_wave);
benchmark_main!(transforms);
