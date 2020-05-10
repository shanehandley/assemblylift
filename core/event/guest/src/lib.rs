#[macro_use]
extern crate lazy_static;

use std::collections::hash_map::Entry::Occupied;
use std::collections::HashMap;
use std::ffi::c_void;
use std::future::Future;
use std::io::Read;
use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

use bincode::deserialize;
use futures::FutureExt;
use futures::task::{ArcWake, waker_ref};
use serde::Deserialize;

use assemblylift_core_event_common::EventMemoryDocument;

extern {
    fn __asml_abi_poll(id: u32) -> i32;
    fn __asml_abi_event_ptr(id: u32) -> u32;
    fn __asml_abi_event_len(id: u32) -> u32;
}

const MAX_EVENTS: usize              = 50;
const EVENT_SIZE_BYTES: usize        = 512;
const EVENT_BUFFER_SIZE_BYTES: usize = MAX_EVENTS * EVENT_SIZE_BYTES;

// Raw buffer holding serialized Event-Future's
pub static mut EVENT_BUFFER: [u8; EVENT_BUFFER_SIZE_BYTES] = [0; EVENT_BUFFER_SIZE_BYTES];

#[no_mangle]
pub fn __asml_get_event_buffer_pointer() -> *const u8 {
    unsafe { EVENT_BUFFER.as_ptr() }
}

#[derive(Clone)]
pub struct Event<'a, R> {
    pub id: u32,
    waker: Box<Option<Waker>>,
    _phantom: PhantomData<&'a R>,
}

impl<'a, R: Deserialize<'a>> Event<'_, R> {
    pub fn new(id: u32) -> Self {
        Event { id, waker: Box::new(None), _phantom: PhantomData }
    }
}

impl<'a, R: Deserialize<'a>> Future for Event<'_, R> {
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match unsafe { __asml_abi_poll(self.id) } {
            1 => Poll::Ready(unsafe { read_response::<Self::Output>(self.id).unwrap() }),
            _ => {
                self.waker = Box::new(Some(cx.waker().clone()));
                Poll::Pending
            }
        }
    }
}

unsafe fn read_response<'a, R: Deserialize<'a>>(id: u32) -> Option<R> {
    let ptr = __asml_abi_event_ptr(id) as usize;
    let end = __asml_abi_event_len(id) as usize + ptr;

    if let Ok(response) = deserialize::<R>(&EVENT_BUFFER[ptr..end]) {
        return Some(response);
    }

    None
}
