#![no_std]
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop{}
}

static RUST_LIB_STATIC_ARR: [u32; 10] = [0,1,2,3,4,5,6,7,8,9];

static mut RUST_LIB_STATIC_MUT_ARR: [f64; 3] = [1.23, 4.56, 7.89];

fn rust_triple_mult_internal(a: i32, b: i32, c: i32) -> i32 {
    a * b * c
}

#[no_mangle]
pub fn rust_triple_mult(a: i32, b: i32, c: i32) -> i32 {
    rust_triple_mult_internal(a,b,c)
}

#[no_mangle]
pub fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub fn rust_return_array_item(i: i32) -> u32 {
    // The Rust compiler cannot optimize the array away because the index is passed as an argument.
    // Indexing the array would contain bounds checking and thus way to many symbols for the
    // potential panic. Thus, it is coerced into an array which then allows an unchecked access
    // using unsafe.
    // Of course, this is VERY UNSAFE AND STUPID but it creates the desired symbols.
    unsafe { *RUST_LIB_STATIC_ARR.get_unchecked(i as usize) }
}

#[no_mangle]
pub fn rust_return_mut_array_item(i: i32) -> f64 {
    // Same as `rust_return_array_item`
    unsafe { *RUST_LIB_STATIC_MUT_ARR.get_unchecked(i as usize) }
}

#[no_mangle]
pub fn rust_set_mut_array_item(i: i32, data: f64) {
    // Same as `rust_return_array_item`
    unsafe {
        let elem = RUST_LIB_STATIC_MUT_ARR.get_unchecked_mut(i as usize);
        *elem = data
    }
}
