use std::fmt;
use std::f32;

#[derive(Debug, Clone, Copy)]
pub struct Pitch {
    pub hz: f32
}

impl Pitch {
    pub fn new(hz: f32) -> Pitch {
        Pitch {
            hz: hz
        }
    }
    
    fn midi_number(&self) -> f32 {
        69.0 + 12.0 * (self.hz / 440.0).log2()
    }

    pub fn cents_error(&self) -> f32 {
        if !self.hz.is_finite() {
            return f32::NAN;
        }
        
        let midi_number = self.midi_number();
        let cents = (midi_number - midi_number.floor()) * 100.0;
        if cents >= 50.0 {
            cents - 100.0
        }
        else {
            cents
        }
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.hz <= 0.0 || !self.hz.is_finite() {
            write!(f, "")
        } else {
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

            //midi_number of 0 is C-1.
            let rounded_pitch = self.midi_number().round() as i32;
            let name = pitch_names[rounded_pitch as usize % pitch_names.len()];
            let octave = rounded_pitch / pitch_names.len() as i32 - 1;

            write!(f, "{: <2}{}", name, octave)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn a4_is_correct() {
        assert_eq!(format!("{}", Pitch::new(440.0)), "A 4");
    }

    #[test]
    fn a2_is_correct() {
        assert_eq!(format!("{}", Pitch::new(110.0)), "A 2");
    }

    #[test]
    fn c4_is_correct() {
        assert_eq!(format!("{}", Pitch::new(261.63)), "C 4");
    }

    #[test]
    fn f5_is_correct() {
        assert_eq!(format!("{}", Pitch::new(698.46)), "F 5");
    }
}
