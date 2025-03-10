extern crate lazy_static;

use crossbeam_utils::atomic::AtomicCell;
use wasmer_runtime::{Array, WasmPtr};
use wasmer_runtime_core::vm;

use assemblylift_core_io_common::constants::IO_BUFFER_SIZE_BYTES;

use crate::threader::Threader;

pub type WasmBufferPtr = WasmPtr<u8, Array>;

pub mod abi;
pub mod threader;

#[inline(always)]
pub fn invoke_io(
    ctx: &mut vm::Ctx,
    mem: WasmBufferPtr,
    method_path: &str,
    method_input: Vec<u8>,
) -> i32 {
    let threader: *mut Threader = ctx.data.cast();
    if threader.is_null() {
        panic!("Threader instance is NULL in spawn_event")
    }
    let threader = unsafe { threader.as_mut().unwrap() };

    let ioid = threader.next_ioid().unwrap();

    let wasm_instance_memory = ctx.memory(0);
    let memory_writer: &[AtomicCell<u8>] =
        match mem.deref(wasm_instance_memory, 0, IO_BUFFER_SIZE_BYTES as u32) {
            Some(memory) => memory,
            None => panic!("could not dereference WASM guest memory in spawn_event"),
        };

    threader.invoke(method_path, method_input, memory_writer.as_ptr(), ioid);

    ioid as i32
}
