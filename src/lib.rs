#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

#[cfg(feature = "std")]
mod decoder;

#[cfg(feature = "std")]
#[doc(inline)]
pub use decoder::*;

/// State of the DSP encoder of a single channel
pub struct Dsp {
    /// The initial history
    pub hist1: i16,
    /// The initial history 2
    pub hist2: i16,
    /// Coefficients for the audio
    pub coefficients: [i16; 16],
}

impl Dsp {
    /// Decode a single frame of ADPCM data.
    ///
    /// Note: the frames need to be parsed sequentially as the hist1 and hist2 values
    /// are updated every frame.
    pub fn decode_frame(&mut self, frame: [u8; FRAME_SIZE]) -> [i16; 14] {
        let header = frame[0];

        let scale = 1i32 << (header & 0xF);
        let coef_index = usize::from((header >> 4) & 0xF);
        let coef1 = i32::from(self.coefficients[coef_index * 2]);
        let coef2 = i32::from(self.coefficients[coef_index * 2 + 1]);

        let mut out = [0; 14];
        let mut i = 0;

        // 7 data bytes per frame
        for byte in &frame[1..] {
            let byte = *byte;
            // 2 samples per byte
            for s in 0..2 {
                let mut sample = if s == 0 {
                    get_high_nibble(byte)
                } else {
                    get_low_nibble(byte)
                };
                sample = if sample >= 8 { sample - 16 } else { sample };
                sample = (((scale * sample) << 11)
                    + 1024
                    + (coef1 * i32::from(self.hist1) + coef2 * i32::from(self.hist2)))
                    >> 11;
                let sample = clamp(sample);

                out[i] = sample;
                i += 1;

                self.hist2 = self.hist1;
                self.hist1 = sample;
            }
        }
        out
    }
}

/// The amount of samples in a single frame
pub const SAMPLES_PER_FRAME: u32 = 14;
/// The size of one frame in bytes
pub const FRAME_SIZE: usize = 8;

/// Table to convert a nibble to an [`i32`].
const NIBBLE_TO_S8: [i32; 0x10] = [0, 1, 2, 3, 4, 5, 6, 7, -8, -7, -6, -5, -4, -3, -2, -1];

/// Extract the low nibble from the byte and convert it to a [`i32`].
fn get_low_nibble(byte: u8) -> i32 {
    NIBBLE_TO_S8[usize::from(byte & 0xF)]
}

/// Extract the high nibble from the byte and convert it to a [`i32`].
fn get_high_nibble(byte: u8) -> i32 {
    NIBBLE_TO_S8[usize::from((byte >> 4) & 0xF)]
}

/// Clamp an [`i32`] value to [`i16`].
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "It's clamped to i16 and therefore safe."
)]
fn clamp(val: i32) -> i16 {
    val.clamp(-32768, 32767) as i16
}
