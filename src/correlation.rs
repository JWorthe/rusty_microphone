use signal::Signal;
use pitch::Pitch;

#[derive(Debug, Default, Clone)]
pub struct Correlation {
    pub value: Vec<f32>
}

impl Correlation {
    pub fn from_signal(signal: &Signal) -> Correlation {
        let samples = &signal.samples;
        Correlation {
            value: (0..samples.len()).map(|offset| {
                samples.iter().take(samples.len() - offset)
                    .zip(samples.iter().skip(offset))
                    .map(|(sig_i, sig_j)| sig_i * sig_j)
                    .sum()
            }).collect()
        }
    }

    pub fn find_fundamental_frequency(&self, signal: &Signal) -> Option<Pitch> {
        if signal.is_silence() {
            // silence
            return None;
        }

        let first_peak_end = match self.value.iter().position(|&c| c < 0.0) {
            Some(p) => p,
            None => {
                // musical signals will drop below 0 at some point
                return None
            }
        };
        
        let peak = self.value.iter()
            .enumerate()
            .skip(first_peak_end)
            .fold((first_peak_end, 0.0), |(xi, xmag), (yi, &ymag)| if ymag > xmag { (yi, ymag) } else { (xi, xmag) });

        let (peak_index, _) = peak;

        let refined_peak_index = self.refine_fundamentals(peak_index as f32 - 0.5, peak_index as f32 + 0.5);

        if self.is_noise(refined_peak_index) {
            None
        }
        else {
            Some(Pitch::new(signal.sample_rate / refined_peak_index))
        }
    }

    fn refine_fundamentals(&self, low_bound: f32, high_bound: f32) -> f32 {
        let data_points = 2 * self.value.len() / high_bound.ceil() as usize;
        let range = high_bound - low_bound;
        let midpoint = (low_bound + high_bound) / 2.0;
        
        if (range * data_points as f32) < 1.0 {
            midpoint
        }
        else {
            let low_guess = self.score_guess(low_bound, data_points);
            let high_guess = self.score_guess(high_bound, data_points);
            
            if high_guess > low_guess {
                self.refine_fundamentals(midpoint, high_bound)
            }
            else {
                self.refine_fundamentals(low_bound, midpoint)
            }
        }
    }

    fn score_guess(&self, period: f32, data_points: usize) -> f32 {
        (1..data_points).map(|i| {
            let expected_sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            let x = i as f32 * period / 2.0;
            let weight = 0.5 * i as f32;
            expected_sign * weight * self.interpolate(x)
        }).sum()
    }

    fn interpolate(&self, x: f32) -> f32 {
        if x.floor() < 0.0 {
            self.value[0]
        }
        else if x.ceil() >= self.value.len() as f32 {
            self.value[self.value.len()-1]
        }
        else {
            let x0 = x.floor();
            let y0 = self.value[x0 as usize];
            let x1 = x.ceil();
            let y1 = self.value[x1 as usize];

            if x0 as usize == x1 as usize {
                y0
            }
            else {
                (y0*(x1-x) + y1*(x-x0)) / (x1-x0)
            }
        }
    }

    fn is_noise(&self, fundamental: f32) -> bool {
        let value_at_point = self.interpolate(fundamental);
        let score_data_points = 2 * self.value.len() / fundamental.ceil() as usize;
        let score = self.score_guess(fundamental, score_data_points);

        value_at_point > 2.0*score
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const SAMPLE_RATE: f32 = 44100.0;
    const FRAMES: u16 = 512;

    fn frequency_resolution() -> f32 {
        SAMPLE_RATE / 2.0 / f32::from(FRAMES)
    }

    fn sin_arg(f: f32, t: f32) -> f32 {
        2.0 as f32 * PI * f * t
    }

    fn sample_sinusoid(amplitude: f32, frequency: f32) -> Signal {
        let samples: Vec<f32> = (0..FRAMES)
            .map(|x| {
                let t = f32::from(x) / SAMPLE_RATE;
                sin_arg(frequency, t).sin() * amplitude
            }).collect();
        
        Signal::new(&samples, SAMPLE_RATE)
    }
    
    #[test]
    fn correlation_on_sine_wave() {
        let frequency = 440.0f32; //concert A
        
        let signal = sample_sinusoid(1.0, frequency);
        let fundamental = Correlation::from_signal(&signal).find_fundamental_frequency(&signal).expect("Find fundamental returned None");
        assert!((fundamental.hz-frequency).abs() < frequency_resolution(), "expected={}, actual={}", frequency, fundamental);
    }

    #[test]
    fn interpolate_half_way() {
        let corr = Correlation {
            value: vec!(0.0, 1.0)
        };
        assert_eq!(0.5, corr.interpolate(0.5))
    }
}
