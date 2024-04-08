//! Control PWM servo
//! Based on:
//! https://www.codrey.com/raspberry-pi/raspberry-pi-pico-sweeping-servo/
//! https://en.wikipedia.org/wiki/Servo_control
//!
//! Typical servo range: 1_000us - 2_000us. Center at 1_500us
//!
#![no_std]
#![no_main]

use bsp::hal::{
    adc::AdcPin, clocks::{init_clocks_and_plls, Clock}, pac, sio::Sio, watchdog::Watchdog, Adc
};
use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use embedded_hal::{adc::OneShot, digital::v2::OutputPin};
use panic_probe as _;
use rp_pico as bsp;


const PWM_WIDTH:u32 = 20_000; // us
const PULSE_MIN:u32 = 500; // us
const PULSE_MAX:u32 = 2_500; // us
const PULSE_WIDTH:u32 = PULSE_MAX - PULSE_MIN;


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
    let mut servo_pin = pins.gpio15.into_push_pull_output();

    info!("start");
    loop {
        let v: u16 = adc.read(&mut pot_pin).unwrap();
        let pulse: u32 = PULSE_MIN + (PULSE_WIDTH * v as u32) / 4096;
        info!("Pulse: {}", pulse);
        servo_pin.set_high().unwrap();
        delay.delay_us(pulse);
        servo_pin.set_low().unwrap();
        delay.delay_us(PWM_WIDTH - pulse);
    }
}

// End of file
