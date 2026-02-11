#![cfg_attr(all(target_arch = "wasm32", not(feature = "std")), no_std)]

// Panic handler for WASM builds (when std feature is not enabled)
#[cfg(all(target_arch = "wasm32", not(feature = "std")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub mod r19_11;
pub use r19_11::*;
