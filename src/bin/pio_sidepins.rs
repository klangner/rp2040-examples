//! PIO programming with side pins
//!
//! More complex example which implements demo protocol similar to I2S
//! It will demonstrate
//! * Usage of side pins
//! * Using DMA to access FIFO queue os PIO processor
//!
#![no_std]
#![no_main]

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock}, dma::{double_buffer, DMAExt}, gpio::FunctionPio0, pac, pio::{Buffers, PIOBuilder, PIOExt, PinDir, ShiftDirection}, sio::Sio, watchdog::Watchdog
};
use cortex_m::singleton;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;


const I2S_PIO_CLOCKDIV_INT: u16 = 400;
const I2S_PIO_CLOCKDIV_FRAC: u8 = 0u8;
const WORD_SIZE: u8 = 32;
const DMA_BUFFER_SIZE: usize = 16;


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

    let data_out_pin = pins.gpio14.into_function::<FunctionPio0>();
    let bclk_pin = pins.gpio16.into_function::<FunctionPio0>();
    let lrclk_pin = pins.gpio17.into_function::<FunctionPio0>();

    // Install a program in instruction memory.
    let program = pio_proc::pio_asm!(" 
        .side_set 2
                    ;                  /----LRCLK
                    ;                  |/---BCLK    
        .wrap_target    
            set y, 30 [2]       side 0b01
        loopLch:
            out pins, 1 [1]     side 0b00; MSB -> LSB
            jmp y-- loopLch [2] side 0b01
            out pins, 1 [1]     side 0b10; LSB
            set y, 30 [2]       side 0b11
        loopRch:
            out pins, 1 [1]     side 0b10; MSB -> LSB
            jmp y-- loopRch [2] side 0b11
            out pins, 1 [1]     side 0b00; LSB
        .wrap
    "
    )
    .program;
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program).unwrap();
    // Configure a state machine to use the program.
    let (mut sm, _rx, tx) = PIOBuilder::from_program(installed)
        .out_pins(data_out_pin.id().num, 1)
        .side_set_pin_base(bclk_pin.id().num)
        .out_shift_direction(ShiftDirection::Left) // I2S MSB first
        .autopull(true)
        .pull_threshold(WORD_SIZE) // Bit-depth: 32bit
        .buffers(Buffers::OnlyTx)
        .clock_divisor_fixed_point(I2S_PIO_CLOCKDIV_INT, I2S_PIO_CLOCKDIV_FRAC)
        .build(sm0);

    // The GPIO pin needs to be configured as an output.
    sm.set_pindirs([
        (data_out_pin.id().num, PinDir::Output),
        (bclk_pin.id().num, PinDir::Output),
        (lrclk_pin.id().num, PinDir::Output),
    ]);
    sm.start();
    delay.delay_ms(1000);

        //=============================DMA===============================
    let dma_channels = pac.DMA.split(&mut pac.RESETS);
    let i2s_tx_buf1 = singleton!(: [u32; DMA_BUFFER_SIZE*2] = [12345; DMA_BUFFER_SIZE*2]).unwrap(); 
    let i2s_tx_buf2 = singleton!(: [u32; DMA_BUFFER_SIZE*2] = [123; DMA_BUFFER_SIZE*2]).unwrap(); 
    let i2s_dma_config =
        double_buffer::Config::new((dma_channels.ch0, dma_channels.ch1), i2s_tx_buf1, tx);
    let i2s_tx_transfer = i2s_dma_config.start(); 
    let mut i2s_tx_transfer = i2s_tx_transfer.read_next(i2s_tx_buf2);


    info!("Clock {=u32}", clocks.system_clock.freq().to_Hz());
    loop {
        if i2s_tx_transfer.is_done() {
            let (next_tx_buf, next_tx_transfer) = i2s_tx_transfer.wait();

            for (i, e) in next_tx_buf.iter_mut().enumerate() {
                if i % 2 == 0 {
                    // Left channel
                    *e = 0xff00ff00;
                } else {
                    *e = 0xff00ff00;
                }
            }

            i2s_tx_transfer = next_tx_transfer.read_next(next_tx_buf);
        }
    }
}