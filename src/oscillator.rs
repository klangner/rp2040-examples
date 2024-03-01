//! Oscillators
//! 
#[allow(unused_imports)]
use num_traits::float::Float;

use core::f32::consts::TAU;


pub trait Oscillator<T> {
    fn next_sample(&mut self) -> T;
    
    fn write_buffer(&mut self, buffer: &mut [T]) {
        for e in buffer.iter_mut() {*e = self.next_sample()}
    }
}


/// Sinusoidal signal
pub struct Sine {
    freq: f32,
    sample_rate: u32,
    step_pos: u32,
}

impl Sine {
    pub fn new(freq: f32, sample_rate: u32) -> Self {
        Self { freq, sample_rate, step_pos: 0 }
    }
}

impl Oscillator<f32> for Sine {
    fn next_sample(&mut self) -> f32 {
        let w = TAU * self.freq / (self.sample_rate as f32);
        let sample = (self.step_pos as f32 * w).cos();
        self.step_pos += 1;
        sample
    }
}


/// Generate triangular signal
pub struct Sawtooth {
    freq: f32,
    sample_rate: u32,
    step_pos: u32,
}

impl Sawtooth {
    /// Create new Triangle generator
    ///   * freq - signal frequency
    ///   * sample_rate - Number of samples/s
    pub fn new(freq: f32, sample_rate: u32) -> Sawtooth {
        Sawtooth { step_pos: 0, freq, sample_rate}
    }
}

// Iterator implementation for f32
impl Oscillator<f32> for Sawtooth {
    fn next_sample(&mut self) -> f32 {
        let sample = 2.0 * ((self.step_pos as f32) * self.freq / (self.sample_rate as f32)).fract() - 1.0;
        self.step_pos += 1;
        if self.step_pos >= self.sample_rate {
            self.step_pos = 0;
        }
        sample
    }
}

// Iterator implementation for u32 (PCM)
impl Oscillator<u32> for Sawtooth {
    fn next_sample(&mut self) -> u32 {
        let i = ((self.step_pos as f32) * self.freq / (self.sample_rate as f32)).fract();
        let sample = (i * u32::MAX as f32) as u32;
        self.step_pos += 1;
        if self.step_pos >= self.sample_rate {
            self.step_pos = 0;
        }
        sample
    }
}

/// Generate square signal
pub struct Square {
    freq: f32,
    sample_rate: u32,
    step_pos: u32,
}

impl Square {
    /// Create new square function generator
    ///   * freq - signal frequency
    ///   * sample_rate - Number of samples/s
    pub fn new(freq: f32, sample_rate: u32) -> Square {
        Square { step_pos: 0, freq, sample_rate}
    }
}

// Iterator implementation for f32
impl Oscillator<f32> for Square {
    fn next_sample(&mut self) -> f32 {
        let sample = if ((self.step_pos as f32) * self.freq/(self.sample_rate as f32)).fract() < 0.5 {
            1.0
        } else {
            -1.0
        };
        self.step_pos += 1;
        if self.step_pos >= self.sample_rate {
            self.step_pos = 0;
        }
        sample
    }
}

// Iterator implementation for PCM data
impl Oscillator<u32> for Square {
    fn next_sample(&mut self) -> u32 {
        let sample = if ((self.step_pos as f32) * self.freq/(self.sample_rate as f32)).fract() < 0.5 {
            u32::MAX
        } else {
            0
        };

        self.step_pos += 1;
        if self.step_pos >= self.sample_rate {
            self.step_pos = 0;
        }
        sample
    }
}