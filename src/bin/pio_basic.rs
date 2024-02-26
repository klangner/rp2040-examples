//! Basic PIO programming
//!
//! This program will switch on/off defined pin, with the frequency based on
//! Pico clock, clock_div and numer of cycles (Minimal ~29Hz)
//!
#![no_std]
#![no_main]

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock}, gpio::FunctionPio0, pac, pio::{PIOBuilder, PIOExt, PinDir}, sio::Sio, watchdog::Watchdog
};
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;


const I2S_PIO_CLOCKDIV_INT: u16 = u16::MAX;
const I2S_PIO_CLOCKDIV_FRAC: u8 = 0u8;


#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

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

    info!("Clock {=u32}", clocks.system_clock.freq().to_Hz());
    info!("I2S_PIO_CLOCKDIV_INT = {=u16}", I2S_PIO_CLOCKDIV_INT);
    info!("freq = {=u32}", clocks.system_clock.freq().to_Hz() / I2S_PIO_CLOCKDIV_INT as u32 / 64);


    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // let led_pin = pins.led.into_function::<FunctionPio0>();
    let led_pin = pins.gpio16.into_function::<FunctionPio0>();

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    // Install a program in instruction memory.
    let program = pio_proc::pio_asm!(
        " 
        .wrap_target
            set pins, 1 [31]
            set pins, 0 [30]
        .wrap
    "
    )
    .program;
    let installed = pio.install(&program).unwrap();
    // Configure a state machine to use the program.
    let (mut sm0, _rx, _tx) = PIOBuilder::from_program(installed)
        .set_pins(led_pin.id().num, 1)
        .clock_divisor_fixed_point(I2S_PIO_CLOCKDIV_INT, I2S_PIO_CLOCKDIV_FRAC)
        .build(sm0);

    // The GPIO pin needs to be configured as an output.
    sm0.set_pindirs([
        (led_pin.id().num, PinDir::Output),
    ]);
    sm0.start();
    delay.delay_ms(1000);

    loop {
        delay.delay_ms(5_000);
    }
}