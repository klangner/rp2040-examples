//! Implementation of I2S protocol for sending PCM data
//!
//! Look at the code to find which 3 pins should be connected.
//!
#![no_std]
#![no_main]

use bsp::hal::{
    dma::{double_buffer, DMAExt}, gpio::FunctionPio0, pac, pio::{Buffers, PIOBuilder, PIOExt, PinDir, ShiftDirection}, sio::Sio
};
use cortex_m::singleton;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;
#[allow(unused_imports)]
use num_traits::float::Float;
use rp2040_sandbox::oscillator::{Oscillator, Square};


// Sound sample rate
const SAMPLE_RATE: u32 = 48_000;
// Bits per channel
const NUM_BITS: u32 = 32;
// System clock
const RP2040_CLOCK_HZ: u32 = 125_000_000;
// Number of cycles required for sending single sample
const CYCLES_PER_SAMPLE: u32 = 5;
// I2S bit clock
const I2S_PIO_CLOCK_HZ: u32 = SAMPLE_RATE * 2 * NUM_BITS * CYCLES_PER_SAMPLE;
/// int + (frac/256)
const I2S_PIO_CLOCKDIV_INT: u16 = (RP2040_CLOCK_HZ / I2S_PIO_CLOCK_HZ) as u16;
const I2S_PIO_CLOCKDIV_FRAC: u8 = 0u8;

// How many sample can be put into DMA buffer. (Mono)
const DMA_BUFFER_SIZE: usize = 16;

#[entry]
fn main() -> ! {
    info!("Program start");
    info!("SAMPLE_RATE: {=u32}", SAMPLE_RATE);
    info!("I2S_PIO_CLOCKDIV_INT: {=u16}", I2S_PIO_CLOCKDIV_INT);
    let mut peripherals = pac::Peripherals::take().unwrap();
    let sio = Sio::new(peripherals.SIO);

    let pins = bsp::Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    let data_out_pin = pins.gpio13.into_function::<FunctionPio0>();
    let bclk_pin = pins.gpio14.into_function::<FunctionPio0>();
    let lrclk_pin = pins.gpio15.into_function::<FunctionPio0>();

    let (mut pio, sm0, _, _, _) = peripherals.PIO0.split(&mut peripherals.RESETS);
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
    ").program;
    let installed = pio.install(&program).unwrap();
    // Configure a state machine to use the program.
    let (mut sm0, _rx, tx) = PIOBuilder::from_program(installed)
        .out_pins(data_out_pin.id().num, 1)
        .side_set_pin_base(bclk_pin.id().num)
        .out_shift_direction(ShiftDirection::Left) // I2S MSB first
        .autopull(true)
        .pull_threshold(32u8) // Bit-depth: 32bit
        .buffers(Buffers::OnlyTx)
        .clock_divisor_fixed_point(I2S_PIO_CLOCKDIV_INT, I2S_PIO_CLOCKDIV_FRAC)
        .build(sm0);

    // The GPIO pin needs to be configured as an output.
    sm0.set_pindirs([
        (data_out_pin.id().num, PinDir::Output),
        (bclk_pin.id().num, PinDir::Output),
        (lrclk_pin.id().num, PinDir::Output),
    ]);
    sm0.start(); 

    //=============================DMA===============================
    let dma_channels = peripherals.DMA.split(&mut peripherals.RESETS);
    // Static buffers. 2* BUFFER_SIZE for stereo
    let i2s_tx_buf1 = singleton!(: [u32; DMA_BUFFER_SIZE*2] = [12345; DMA_BUFFER_SIZE*2]).unwrap(); //static
    let i2s_tx_buf2 = singleton!(: [u32; DMA_BUFFER_SIZE*2] = [123; DMA_BUFFER_SIZE*2]).unwrap(); //static
    let i2s_dma_config =
        double_buffer::Config::new((dma_channels.ch0, dma_channels.ch1), i2s_tx_buf1, tx);
    let i2s_tx_transfer = i2s_dma_config.start(); 
    let mut i2s_tx_transfer = i2s_tx_transfer.read_next(i2s_tx_buf2);

    let mut square = Square::new(220.0, SAMPLE_RATE);
    loop {
        if i2s_tx_transfer.is_done() {
            let (next_tx_buf, next_tx_transfer) = i2s_tx_transfer.wait();

            for (i, e) in next_tx_buf.iter_mut().enumerate() {
                if i % 2 == 0 {
                    // Left channel only
                    let sample: u32 = square.next_sample();
                    *e = sample / 100;
                }
            }

            i2s_tx_transfer = next_tx_transfer.read_next(next_tx_buf);
        }
    }
}

// End of file
