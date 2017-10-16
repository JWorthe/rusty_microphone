use transforms;

use std::os::raw::c_char;
use std::ffi::CString;
use std::slice;

#[no_mangle]
pub extern "C" fn find_fundamental_frequency(signal: *const f32, signal_length: isize, sample_rate: f32) -> f32 {
    let signal_slice = unsafe {
        &slice::from_raw_parts(signal, signal_length as usize)
    };
    
    transforms::find_fundamental_frequency(&signal_slice, sample_rate).unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn hz_to_cents_error(hz: f32) -> f32 {
    transforms::hz_to_cents_error(hz)
}

#[no_mangle]
pub extern "C" fn hz_to_pitch(hz: f32) -> *mut c_char {
    let pitch = transforms::hz_to_pitch(hz);
    CString::new(pitch)
		.unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn correlation(signal: *mut f32, signal_length: isize) {
    let signal_slice = unsafe {
        &slice::from_raw_parts(signal, signal_length as usize)
    };

    let correlated_signal = transforms::correlation(&signal_slice);

    unsafe {
        for (i, cor) in correlated_signal.iter().enumerate() {
            *signal.offset(i as isize) = *cor;
        }
    }
}
