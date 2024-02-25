//! Based on:
//! https://how2electronics.com/interfacing-16x2-lcd-display-with-raspberry-pi-pico/
//! Datasheet
//! https://www.sparkfun.com/datasheets/LCD/HD44780.pdf
//!
#![no_std]
#![no_main]

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{DynPinId, FunctionSioOutput, Pin, PullNone},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;
use rp_pico as bsp;

struct Lcd1602 {
    rs: Pin<DynPinId, FunctionSioOutput, PullNone>,
    e: Pin<DynPinId, FunctionSioOutput, PullNone>,
    d4: Pin<DynPinId, FunctionSioOutput, PullNone>,
    d5: Pin<DynPinId, FunctionSioOutput, PullNone>,
    d6: Pin<DynPinId, FunctionSioOutput, PullNone>,
    d7: Pin<DynPinId, FunctionSioOutput, PullNone>,
    delay: Delay,
}

impl Lcd1602 {
    fn init(
        delay: Delay,
        rs: Pin<DynPinId, FunctionSioOutput, PullNone>,
        e: Pin<DynPinId, FunctionSioOutput, PullNone>,
        d4: Pin<DynPinId, FunctionSioOutput, PullNone>,
        d5: Pin<DynPinId, FunctionSioOutput, PullNone>,
        d6: Pin<DynPinId, FunctionSioOutput, PullNone>,
        d7: Pin<DynPinId, FunctionSioOutput, PullNone>,
    ) -> Self {
        Self {
            rs,
            e,
            d4,
            d5,
            d6,
            d7,
            delay,
        }
    }

    fn setup(&mut self) {
        // Write instructions
        self.rs.set_low().unwrap();
        self.send_lcd4(0b0011); // 8 bit
        self.send_lcd4(0b0011); // 8 bit
        self.send_lcd4(0b0011); // 8 bit
        self.send_lcd4(0b0010); // 4 bit
        self.send_lcd8(0b00101000); // 4 bit,2 lines?,5*8 bots
        self.send_lcd8(0b00001100); // lcd on, blink off, cursor off.
        self.send_lcd8(0b00000110); // increment cursor, no display shift
        self.send_lcd8(0b00000001); // clear screen

        self.delay.delay_ms(2); // clear screen needs a long delay
    }

    fn print(&mut self, text: &str) {
        // Write data
        self.rs.set_high().unwrap();
        for x in text.bytes() {
            self.send_lcd8(x);
        }
    }

    fn clear(&mut self) {
        self.rs.set_low().unwrap();
        self.send_lcd8(0b00000001);
    }

    fn pulse_e(&mut self) {
        self.e.set_high().unwrap();
        self.delay.delay_us(40);
        self.e.set_low().unwrap();
        self.delay.delay_us(40);
    }

    fn send_lcd4(&mut self, bin_num: u8) {
        self.d4
            .set_state(((bin_num & 0b00000001) > 0).into())
            .unwrap();
        self.d5
            .set_state(((bin_num & 0b00000010) > 0).into())
            .unwrap();
        self.d6
            .set_state(((bin_num & 0b00000100) > 0).into())
            .unwrap();
        self.d7
            .set_state(((bin_num & 0b00001000) > 0).into())
            .unwrap();
        self.pulse_e();
    }

    fn send_lcd8(&mut self, bin_num: u8) {
        self.d4
            .set_state(((bin_num & 0b00010000) > 0).into())
            .unwrap();
        self.d5
            .set_state(((bin_num & 0b00100000) > 0).into())
            .unwrap();
        self.d6
            .set_state(((bin_num & 0b01000000) > 0).into())
            .unwrap();
        self.d7
            .set_state(((bin_num & 0b10000000) > 0).into())
            .unwrap();
        self.pulse_e();
        self.d4
            .set_state(((bin_num & 0b00000001) > 0).into())
            .unwrap();
        self.d5
            .set_state(((bin_num & 0b00000010) > 0).into())
            .unwrap();
        self.d6
            .set_state(((bin_num & 0b00000100) > 0).into())
            .unwrap();
        self.d7
            .set_state(((bin_num & 0b00001000) > 0).into())
            .unwrap();
        self.pulse_e();
    }
}

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

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();
    let rs = pins
        .gpio16
        .into_push_pull_output()
        .into_pull_type()
        .into_dyn_pin();
    let e = pins
        .gpio17
        .into_push_pull_output()
        .into_pull_type()
        .into_dyn_pin();
    let d4 = pins
        .gpio18
        .into_push_pull_output()
        .into_pull_type()
        .into_dyn_pin();
    let d5 = pins
        .gpio19
        .into_push_pull_output()
        .into_pull_type()
        .into_dyn_pin();
    let d6 = pins
        .gpio20
        .into_push_pull_output()
        .into_pull_type()
        .into_dyn_pin();
    let d7 = pins
        .gpio21
        .into_push_pull_output()
        .into_pull_type()
        .into_dyn_pin();

    // Init led
    let delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut lcd = Lcd1602::init(delay, rs, e, d4, d5, d6, d7);
    lcd.setup();

    loop {
        lcd.print("AAA");
        led_pin.set_high().unwrap();
        lcd.delay.delay_ms(5_000);
        lcd.clear();
        led_pin.set_low().unwrap();
        lcd.delay.delay_ms(5_000);
    }
}
