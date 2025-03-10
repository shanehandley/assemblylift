use std::cell::Cell;
use std::error::Error;
use std::io;
use std::io::ErrorKind;

use wasmer_runtime::memory::MemoryView;
use wasmer_runtime_core::vm;

use crate::threader::Threader;
use crate::{invoke_io, WasmBufferPtr};
use std::time::{SystemTime, UNIX_EPOCH};

pub type AsmlAbiFn = fn(&mut vm::Ctx, WasmBufferPtr, WasmBufferPtr, u32) -> i32;

fn to_io_error<E: Error>(err: E) -> io::Error {
    io::Error::new(ErrorKind::Other, err.to_string())
}

pub fn asml_abi_invoke(
    ctx: &mut vm::Ctx,
    mem: WasmBufferPtr,
    name_ptr: u32,
    name_len: u32,
    input: u32,
    input_len: u32,
) -> i32 {
    if let Ok(method_path) = ctx_ptr_to_string(ctx, name_ptr, name_len) {
        if let Ok(input) = ctx_ptr_to_bytes(ctx, input, input_len) {
            return invoke_io(ctx, mem, &*method_path, input);
        }
    }

    -1i32 // error
}

pub fn asml_abi_poll(ctx: &mut vm::Ctx, id: u32) -> i32 {
    let threader = get_threader(ctx);
    unsafe { threader.as_mut().unwrap().poll(id) as i32 }
}

pub fn asml_abi_io_ptr(ctx: &mut vm::Ctx, id: u32) -> u32 {
    let threader = get_threader(ctx);
    unsafe {
        threader
            .as_mut()
            .unwrap()
            .get_io_memory_document(id)
            .unwrap()
            .start as u32
    }
}

pub fn asml_abi_io_len(ctx: &mut vm::Ctx, id: u32) -> u32 {
    let threader = get_threader(ctx);
    unsafe {
        threader
            .as_mut()
            .unwrap()
            .get_io_memory_document(id)
            .unwrap()
            .length as u32
    }
}

pub fn asml_abi_clock_time_get(_ctx: &mut vm::Ctx) -> u64 {
    let start = SystemTime::now();
    let unix_time = start
        .duration_since(UNIX_EPOCH)
        .expect("time is broken");
    unix_time.as_secs() * 1000u64
}

#[inline]
pub fn get_threader(ctx: &mut vm::Ctx) -> *mut Threader {
    let threader: *mut Threader = ctx.data.cast();
    if threader.is_null() {
        panic!("Threader instance is NULL in abi::get_threader")
    }

    threader
}

#[inline]
fn ctx_ptr_to_string(ctx: &mut vm::Ctx, ptr: u32, len: u32) -> Result<String, io::Error> {
    let memory = ctx.memory(0);
    let view: MemoryView<u8> = memory.view();

    let mut str_vec: Vec<u8> = Vec::new();
    for byte in view[ptr as usize..(ptr + len) as usize]
        .iter()
        .map(Cell::get)
    {
        str_vec.push(byte);
    }

    std::str::from_utf8(str_vec.as_slice())
        .map(String::from)
        .map_err(to_io_error)
}

#[inline]
fn ctx_ptr_to_bytes(ctx: &mut vm::Ctx, ptr: u32, len: u32) -> Result<Vec<u8>, io::Error> {
    let memory = ctx.memory(0);
    let view: MemoryView<u8> = memory.view();

    let mut bytes: Vec<u8> = Vec::new();
    for byte in view[ptr as usize..(ptr + len) as usize]
        .iter()
        .map(Cell::get)
    {
        bytes.push(byte);
    }

    Ok(bytes)
}
