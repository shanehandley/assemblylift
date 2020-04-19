use assemblylift_core_event::constants::*;
use assemblylift_core_event::Event;
use std::pin::Pin;
use std::ffi::c_void;
use std::io::Read;

// Raw buffer holding serialized Event-Future's
pub static mut EVENT_BUFFER: [u8; EVENT_BUFFER_SIZE_BYTES] = [0; EVENT_BUFFER_SIZE_BYTES];

#[no_mangle]
pub fn __asml_get_event_buffer_pointer() -> *const u8 {
    unsafe { EVENT_BUFFER.as_ptr() }
}
