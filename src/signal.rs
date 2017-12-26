#[derive(Debug, Clone)]
pub struct Signal {
    pub samples: Vec<f32>,
    pub sample_rate: f32
}

impl Signal {
    pub fn empty() -> Signal {
        Signal::default()
    }
    
    pub fn new(samples: &[f32], sample_rate: f32) -> Signal {
        Signal {
            samples: Signal::remove_mean_offset(samples),
            sample_rate: sample_rate
        }
    }

    fn remove_mean_offset(samples: &[f32]) -> Vec<f32> {
        let mean = samples.iter().sum::<f32>()/samples.len() as f32;
        samples.iter().map(|x| x - mean).collect()
    }

    pub fn aligned_to_rising_edge(&self) -> &[f32] {
        let rising_edge = self.samples
            .iter()
            .enumerate()
            .skip_while(|&(_,x)| !x.is_sign_negative())
            .skip_while(|&(_,x)| x.is_sign_negative())
            .map(|(i,_)| i)
            .next().unwrap_or(0);
        &self.samples[rising_edge..]
    }

    pub fn is_silence(&self) -> bool {
        self.samples.iter().all(|&x| x.abs() < 0.05)
    }

}

impl Default for Signal {
    fn default() -> Signal {
        Signal {
            samples: Vec::new(),
            sample_rate: 44100.0
        }
    }
}
