//! Read potentiometer value using ADC port (GP26)
//!
#![no_std]
#![no_main]

use bsp::hal::{
    adc::AdcPin, clocks::{init_clocks_and_plls, Clock}, pac, sio::Sio, watchdog::Watchdog, Adc
};
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::{adc::OneShot, digital::v2::OutputPin};
use panic_probe as _;
use rp_pico as bsp;

#[entry]
fn main() -> ! {
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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Enable ADC
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    // Configure GPIO26 as an ADC input
    let mut pot_pin = AdcPin::new(pins.gpio26);
    let mut led_pin = pins.led.into_push_pull_output();

    loop {
        let v: u16 = adc.read(&mut pot_pin).unwrap();
        info!("Pot value {=u16}", v);
        
        let d: u32 = (1_000 * v as u32) / 4096;
        led_pin.set_high().unwrap();
        delay.delay_ms(d);
        led_pin.set_low().unwrap();
        delay.delay_ms(1_000 - d);
    }
}

// End of file
