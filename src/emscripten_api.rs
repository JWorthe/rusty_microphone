use transforms;

#[no_mangle]
pub extern "C" fn find_fundamental_frequency(signal: *const f32, signal_length: isize, sample_rate: f32) -> f32 {
    use std::slice;
    let signal_slice = unsafe {
        &slice::from_raw_parts(signal, signal_length as usize)
    };
    
    transforms::find_fundamental_frequency(&signal_slice, sample_rate).unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn correlation(signal: *const f32, signal_length: isize) {
    //TODO correlate inline
}

#[no_mangle]
pub extern "C" fn hz_to_cents_error(hz: f32) -> f32 {
    //TODO implement
    0.0
}

use std::os::raw::c_char;
use std::ffi::CStr;
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn hz_to_pitch(hz: f32) -> *mut c_char {
    //TODO implement
    CString::new("C 4")
		.unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn align_to_rising_edge(signal: *const f32, signal_length: isize) {
    //TODO format signal nicely inline
}
