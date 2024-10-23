//! An easy-to-use decoder that takes a `std::io::Read` and outputs `i16` as an iterator.
use crate::{Dsp, SAMPLES_PER_FRAME};
use std::io::Read;
use std::marker::PhantomData;

/// Private module to prevent users from implementing [`Channels`] for other types.
mod private {
    use crate::{Mono, Stereo, StereoInterleaved};

    /// Sealed trait to prevent users from implementing [`Channels`] for other types.
    pub trait Sealed {}
    impl Sealed for Mono {}
    impl Sealed for Stereo {}
    impl Sealed for StereoInterleaved {}
}

/// Sealed trait for encoding the channel layout in the type system.
pub trait Channels: private::Sealed {}

/// There is only one channel.
pub enum Mono {}
impl Channels for Mono {}

/// There are two channels in two separate streams.
pub enum Stereo {}
impl Channels for Stereo {}

/// There are two channels interleaved per frame in one stream.
pub enum StereoInterleaved {}
impl Channels for StereoInterleaved {}

/// Wrapper around [`Dsp`] that handles channel layout.
///
/// It takes the initial DSP state and one or two readers for the stream data and
/// outputs a `Result<i16, std::io::Error>` iterator.
pub struct Decoder<R: Read, C: Channels> {
    /// The reader for the left/mono/interleaved audio stream
    left_reader: R,
    /// The reader for the right channel audio stream, only available on [`Stereo`]
    right_reader: Option<R>,
    /// The DSP state of the left/mono channel
    left_state: Dsp,
    /// The DSP state of the right channel, not available when channel is [`Mono`]
    right_state: Option<Dsp>,
    /// The amount of frames that still need to be decoded
    frames_remaing: u32,
    /// Buffer for the decoded frame(s)
    buffer: Vec<i16>,
    /// Fake field for the [`Channels`] typestate
    _phantom_data: PhantomData<C>,
}

impl<R: Read> Decoder<R, Mono> {
    /// Decode a mono audio stream.
    ///
    /// `frames` is the amount of frames in the channel.
    pub fn mono(reader: R, state: Dsp, frames: u32) -> Self {
        Self {
            left_reader: reader,
            right_reader: None,
            left_state: state,
            right_state: None,
            frames_remaing: frames,
            buffer: Vec::with_capacity(14),
            _phantom_data: PhantomData,
        }
    }

    /// Decode a mono audio stream.
    ///
    /// `samples` is the amount of samples in the channel.
    pub fn mono_samples(reader: R, state: Dsp, samples: u32) -> Self {
        Self {
            left_reader: reader,
            right_reader: None,
            left_state: state,
            right_state: None,
            frames_remaing: samples.div_ceil(SAMPLES_PER_FRAME),
            buffer: Vec::with_capacity(14),
            _phantom_data: PhantomData,
        }
    }
}

impl<R: Read> Decoder<R, Stereo> {
    /// Decode a stereo audio stream where each channel has their own buffer.
    ///
    /// `channel_frames` is the amount of frames in *one* channel.
    pub fn stereo(
        left_reader: R,
        left_state: Dsp,
        right_reader: R,
        right_state: Dsp,
        channel_frames: u32,
    ) -> Self {
        Self {
            left_reader,
            right_reader: Some(right_reader),
            left_state,
            right_state: Some(right_state),
            frames_remaing: channel_frames,
            buffer: Vec::with_capacity(28),
            _phantom_data: PhantomData,
        }
    }

    /// Decode a stereo audio stream where each channel has their own buffer.
    ///
    /// `channel_samples` is the amount of samples in *one* channel.
    pub fn stereo_samples(
        left_reader: R,
        left_state: Dsp,
        right_reader: R,
        right_state: Dsp,
        channel_samples: u32,
    ) -> Self {
        Self {
            left_reader,
            right_reader: Some(right_reader),
            left_state,
            right_state: Some(right_state),
            frames_remaing: channel_samples.div_ceil(SAMPLES_PER_FRAME),
            buffer: Vec::with_capacity(28),
            _phantom_data: PhantomData,
        }
    }
}

impl<R: Read> Decoder<R, StereoInterleaved> {
    /// Decode a stereo audio stream interleaved per frame.
    ///
    /// `channel_frames` is the amount of frames in *one* channel.
    pub fn interleaved_stereo(
        reader: R,
        left_state: Dsp,
        right_state: Dsp,
        channel_frames: u32,
    ) -> Self {
        Self {
            left_reader: reader,
            right_reader: None,
            left_state,
            right_state: Some(right_state),
            frames_remaing: channel_frames * 2,
            buffer: Vec::with_capacity(28),
            _phantom_data: PhantomData,
        }
    }

    /// Decode a stereo audio stream interleaved per frame.
    ///
    /// `channel_samples` is the amount of samples in *one* channel.
    pub fn interleaved_stereo_samples(
        reader: R,
        left_state: Dsp,
        right_state: Dsp,
        channel_samples: u32,
    ) -> Self {
        Self {
            left_reader: reader,
            right_reader: None,
            left_state,
            right_state: Some(right_state),
            frames_remaing: channel_samples.div_ceil(SAMPLES_PER_FRAME) * 2,
            buffer: Vec::with_capacity(28),
            _phantom_data: PhantomData,
        }
    }
}

impl<R: Read> Iterator for Decoder<R, Mono> {
    type Item = Result<i16, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() && self.frames_remaing != 0 {
            let mut frame = [0; 8];
            let result = self.left_reader.read_exact(&mut frame);
            if let Err(e) = result {
                return Some(Err(e));
            };
            let mut samples = self.left_state.decode_frame(frame);
            // Reverse the samples as they are output in the wrong order
            samples.as_mut_slice().reverse();
            self.buffer.extend_from_slice(&samples);
            self.frames_remaing -= 1;
        }
        self.buffer.pop().map(Ok)
    }
}

impl<R: Read> Iterator for Decoder<R, Stereo> {
    type Item = Result<i16, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() && self.frames_remaing != 0 {
            let mut left_frame = [0; 8];
            let result = self.left_reader.read_exact(&mut left_frame);
            if let Err(e) = result {
                return Some(Err(e));
            };
            let mut right_frame = [0; 8];
            let result = self
                .right_reader
                .as_mut()
                .unwrap_or_else(|| unreachable!())
                .read_exact(&mut right_frame);
            if let Err(e) = result {
                return Some(Err(e));
            };
            let left = self.left_state.decode_frame(left_frame);
            let right = self
                .right_state
                .as_mut()
                .unwrap_or_else(|| unreachable!())
                .decode_frame(right_frame);
            // Reverse samples and interleave
            self.buffer.extend_from_slice(&[
                left[13], right[13], left[12], right[12], left[11], right[11], left[10], right[10],
                left[9], right[9], left[8], right[8], left[7], right[7], left[6], right[6],
                left[5], right[5], left[4], right[4], left[3], right[3], left[2], right[2],
                left[1], right[1], left[0], right[0],
            ]);
            self.frames_remaing -= 1;
        }
        self.buffer.pop().map(Ok)
    }
}

impl<R: Read> Iterator for Decoder<R, StereoInterleaved> {
    type Item = Result<i16, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() && self.frames_remaing != 0 {
            let mut left_frame = [0; 8];
            let result = self.left_reader.read_exact(&mut left_frame);
            if let Err(e) = result {
                return Some(Err(e));
            };
            let mut right_frame = [0; 8];
            let result = self.left_reader.read_exact(&mut right_frame);
            if let Err(e) = result {
                return Some(Err(e));
            };
            let left = self.left_state.decode_frame(left_frame);
            let right = self
                .right_state
                .as_mut()
                .unwrap_or_else(|| unreachable!())
                .decode_frame(right_frame);
            // Reverse samples and interleave
            self.buffer.extend_from_slice(&[
                left[13], right[13], left[12], right[12], left[11], right[11], left[10], right[10],
                left[9], right[9], left[8], right[8], left[7], right[7], left[6], right[6],
                left[5], right[5], left[4], right[4], left[3], right[3], left[2], right[2],
                left[1], right[1], left[0], right[0],
            ]);
            self.frames_remaing -= 2;
        }
        self.buffer.pop().map(Ok)
    }
}
