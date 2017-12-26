use model::Model;
use signal::Signal;
use pitch::Pitch;

use std::os::raw::c_char;
use std::ffi::CString;
use std::slice;
use std::f32;

#[no_mangle]
pub extern "C" fn find_fundamental_frequency(signal_ptr: *const f32, signal_length: isize, sample_rate: f32) -> f32 {
    let signal_slice = unsafe {
        &slice::from_raw_parts(signal_ptr, signal_length as usize)
    };
    let signal = Signal::new(signal_slice, sample_rate);
    let model = Model::from_signal(signal);

    model.pitch.map_or(f32::NAN, |p| p.hz)
}

#[no_mangle]
pub extern "C" fn hz_to_cents_error(hz: f32) -> f32 {
    let pitch = Pitch::new(hz);
    pitch.cents_error()
}

#[no_mangle]
pub extern "C" fn hz_to_pitch(hz: f32) -> *mut c_char {
    let pitch = Pitch::new(hz);
    CString::new(format!("{}", pitch))
		.unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn correlation(signal_ptr: *mut f32, signal_length: isize, sample_rate: f32) {
    let signal_slice = unsafe {
        &slice::from_raw_parts(signal_ptr, signal_length as usize)
    };

    let signal = Signal::new(signal_slice, sample_rate);
    let model = Model::from_signal(signal);

    unsafe {
        for (i, cor) in model.correlation.value.iter().enumerate() {
            *signal_ptr.offset(i as isize) = *cor;
        }
    }
}
