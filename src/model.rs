use signal::Signal;
use correlation::Correlation;
use pitch::Pitch;

#[derive(Default)]
pub struct Model {
    pub pitch: Option<Pitch>,
    pub signal: Signal,
    pub correlation: Correlation
}

impl Model {
    pub fn new() -> Model {
        Model::default()
    }

    pub fn from_signal(signal: Signal) -> Model {
        let correlation = Correlation::from_signal(&signal);
        let pitch = correlation.find_fundamental_frequency(&signal);
        
        Model {
            pitch: pitch,
            signal: signal,
            correlation: correlation
        }
    }

    pub fn pitch_display(&self) -> String {
        self.pitch.map_or(String::new(), |p| format!("{}", p))
    }
}
