use transforms;

#[derive(Default)]
pub struct Model {
    pub fundamental_frequency: Option<f32>,
    pub pitch: String,
    pub error: Option<f32>,
    pub signal: Vec<f32>,
    pub correlation: Vec<f32>
}

impl Model {
    pub fn new() -> Model {
        Model::default()
    }

    pub fn from_signal(signal: &[f32], sample_rate: f32) -> Model {
        let correlation = transforms::correlation(signal);
        let fundamental = transforms::find_fundamental_frequency(signal, sample_rate);
        let pitch = fundamental.map_or(
            String::new(),
            transforms::hz_to_pitch
        );
        
        let error = fundamental.map(transforms::hz_to_cents_error);
        
        Model {
            fundamental_frequency: fundamental,
            pitch: pitch,
            error: error,
            signal: transforms::align_to_rising_edge(signal),
            correlation: correlation
        }
    }
}
