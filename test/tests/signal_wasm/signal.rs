#![no_std]
#![no_main]

#[no_mangle]
pub static mut __signal_address: i32 = 1;

#[no_mangle]
pub static mut __terminated_address: i32 = 0;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
