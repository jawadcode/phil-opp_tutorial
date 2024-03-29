#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(phil_opp_tutorial::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use phil_opp_tutorial::println;

/// This function is the entry point, since the linker looks for a function named `_start` by default
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic
#[cfg(not(test))] // outside of testing mode
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)] // inside testing mode
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    phil_opp_tutorial::test_panic_handler(info)
}

#[test_case]
#[allow(clippy::eq_op)]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
