//! Blinks 2 LEDs usinf 2 cores on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
//! And the second core will blink led on GP16
//! 
#![no_std]
#![no_main]

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock}, fugit::HertzU32, gpio::{bank0::Gpio25, FunctionSio, Pin, PullDown, SioOutput}, multicore::{Multicore, Stack}, pac, sio::Sio, watchdog::Watchdog
};
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;
use rp_pico as bsp;


static mut CORE1_STACK: Stack<4096> = Stack::new();

fn core1_task(mut led_pin: Pin<Gpio25, FunctionSio<SioOutput>, PullDown>, sys_freq: HertzU32) {
    let core = unsafe { pac::CorePeripherals::steal() };

    let mut delay = cortex_m::delay::Delay::new(core.SYST, sys_freq.to_Hz());
    
    loop {
        // info!("led_25 on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(700);
        // info!("led_25 off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(700);

    }
}


#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    
    let mut led_pin = pins.gpio16.into_push_pull_output();
    let pin_25 = pins.led.into_push_pull_output();


    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];

    info!("start core 1 task");
    let _test = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        core1_task(pin_25, clocks.system_clock.freq())
    });

    info!("start core 0 task.");
    loop {
        // info!("led_16 on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(1_000);
        // info!("led_16 off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(1_000);
    }
}

// End of file
