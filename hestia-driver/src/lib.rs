#![no_std]

mod callbacks;
mod driver;
mod macros;
mod status;
mod strings;
mod wdf;

use wdk::println;
use wdk_sys::{DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING};

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

use crate::driver::initialize_driver;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

pub use status::Result;

#[export_name = "DriverEntry"] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    println!("[driver_entry] Begin");
    unsafe { initialize_driver(driver, registry_path) }
}
