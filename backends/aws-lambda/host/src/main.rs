use std::cell::RefCell;
use std::env;
use std::fs;
use std::process;
use std::sync::{Arc, Mutex};

use clap::crate_version;
use crossbeam_utils::atomic::AtomicCell;
use once_cell::sync::Lazy;
use tokio::sync::mpsc;
use wasmer_runtime::Instance;

use assemblylift_core::threader::Threader;
use assemblylift_core::WasmBufferPtr;
use assemblylift_core_iomod::registry;
use runtime::AwsLambdaRuntime;

mod runtime;
mod wasm;

pub static LAMBDA_RUNTIME: Lazy<AwsLambdaRuntime> = Lazy::new(|| AwsLambdaRuntime::new());
pub static LAMBDA_REQUEST_ID: Lazy<Mutex<RefCell<String>>> =
    Lazy::new(|| Mutex::new(RefCell::new(String::new())));

#[inline(always)]
fn write_event_buffer(instance: &Instance, event: String) {
    use wasmer_runtime::Func;

    let wasm_instance_context = instance.context();
    let wasm_instance_memory = wasm_instance_context.memory(0);

    let fn_name = "__asml_guest_get_aws_event_string_buffer_pointer";
    let get_pointer: Func<(), WasmBufferPtr> = instance
        .exports
        .get(fn_name)
        .expect(&*format!("could not find export in wasm named {}", fn_name));

    let event_buffer = get_pointer.call().unwrap();
    let memory_writer: &[AtomicCell<u8>] = event_buffer
        .deref(wasm_instance_memory, 0, 8192u32)
        .unwrap();

    let bytes = event.bytes();
    for (i, b) in bytes.clone().enumerate() {
        memory_writer[i].store(b);
    }
    if 8192 > bytes.clone().len() {
        // FIXME magic number -- equiv to AWS_EVENT_STRING_BUFFER_SIZE
        for i in bytes.len()..8192 {
            memory_writer[i].store('\0' as u8)
        }
    }
}

#[tokio::main]
async fn main() {
    println!(
        "Starting AssemblyLift AWS Lambda runtime {}",
        crate_version!()
    );

    let registry_channel = mpsc::channel(100);
    let tx = registry_channel.0.clone();
    let rx = registry_channel.1;
    registry::spawn_registry(rx).unwrap();

    // load plugins from runtime dir, which should contain merged contents of Lambda layers
    if let Ok(rd) = fs::read_dir("/opt/iomod") {
        for entry in rd {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                process::Command::new(entry.path()).spawn().unwrap();
            }
        }
    }

    let task_set = tokio::task::LocalSet::new();
    task_set
        .run_until(async move {
            let instance = match wasm::build_instance(tx) {
                Ok(instance) => Arc::new(instance),
                Err(why) => panic!("PANIC {}", why.to_string()),
            };

            while let Ok(event) = LAMBDA_RUNTIME.get_next_event().await {
                {
                    let ref_cell = LAMBDA_REQUEST_ID.lock().unwrap();
                    if ref_cell.borrow().clone() == event.request_id.clone() {
                        continue;
                    }
                    ref_cell.replace(event.request_id.clone());
                }

                let instance = instance.clone();
                tokio::task::spawn_local(async move {
                    // handler coordinates are expected to be <file name>.<function name>
                    let handler_coordinates = env::var("_HANDLER").unwrap();
                    let coords = handler_coordinates.split(".").collect::<Vec<&str>>();
                    let handler_name = coords[1];

                    write_event_buffer(&instance, event.event_body);
                    Threader::__reset_memory();

                    match instance.call(handler_name, &[]) {
                        Ok(result) => println!("TRACE: handler returned {:?}", result),
                        Err(error) => println!("ERROR: {}", error.to_string()),
                    }
                })
                .await
                .unwrap();
            }
        })
        .await;
}
