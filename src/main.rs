#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
use core::panic::PanicInfo;
use x86_64::instructions::port::Port;
use x86_64::VirtAddr;
mod gdt;
mod interrupts;
mod memory;
mod serial;
mod vga_buffer;
use bootloader::{entry_point, BootInfo};
extern crate alloc;
use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
mod allocator;
mod task;
use task::{executor::Executor, keyboard, Task};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();

    interrupts::init_idt(); //中断初始化

    unsafe { interrupts::PICS.lock().initialize() }; // 8259 初始化

    //初始化也分配器
    let mut mapper = unsafe { memory::init(VirtAddr::new(boot_info.physical_memory_offset)) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    //初始化堆分配器
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    println!("==========================!!!!!!!!!!========================");
    println!("==========================OS   START========================");
    println!("==========================!!!!!!!!!!========================");
    //异步执行器
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    // hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    hlt_loop();
}

pub unsafe fn exit_qemu() {
    let mut port = Port::<u32>::new(0xf4);
    port.write(54); // exit code is (54 << 1) | 1 = 109
}
