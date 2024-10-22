//! A easy to use decoder that takes a `std::io::Read` and outputs `i16` as an iterator.
use crate::Dsp;
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
    /// `total_frames` is the total amount of frames in the channel.
    pub fn mono(reader: R, state: Dsp, total_frames: u32) -> Self {
        Self {
            left_reader: reader,
            right_reader: None,
            left_state: state,
            right_state: None,
            frames_remaing: total_frames,
            buffer: Vec::with_capacity(14),
            _phantom_data: PhantomData,
        }
    }
}

impl<R: Read> Decoder<R, Stereo> {
    /// Decode a stereo audio stream where each channel has their own buffer.
    ///
    /// `total_frames` is the total amount of frames in one channel.
    pub fn stereo(
        left_reader: R,
        left_state: Dsp,
        right_reader: R,
        right_state: Dsp,
        total_frames: u32,
    ) -> Self {
        Self {
            left_reader,
            right_reader: Some(right_reader),
            left_state,
            right_state: Some(right_state),
            frames_remaing: total_frames,
            buffer: Vec::with_capacity(28),
            _phantom_data: PhantomData,
        }
    }
}

impl<R: Read> Decoder<R, StereoInterleaved> {
    /// Decode a stereo audio stream interleaved per frame.
    ///
    /// `total_frames` is the total amount of frames for both channels.
    ///
    /// # Panics
    /// Will panic if `total_frames` is not a multiple of 2.
    pub fn interleaved_stereo(
        reader: R,
        left_state: Dsp,
        right_state: Dsp,
        total_frames: u32,
    ) -> Self {
        assert_eq!(
            total_frames % 2,
            0,
            "Total frames needs to be a multiple of 2"
        );
        Self {
            left_reader: reader,
            right_reader: None,
            left_state,
            right_state: Some(right_state),
            frames_remaing: total_frames,
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
            let samples = self.left_state.decode_frame(frame);
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
            let l = self.left_state.decode_frame(left_frame);
            let r = self
                .right_state
                .as_mut()
                .unwrap_or_else(|| unreachable!())
                .decode_frame(right_frame);
            self.buffer.extend_from_slice(&[
                l[0], r[0], l[1], r[1], l[2], r[2], l[3], r[3], l[4], r[4], l[5], r[5], l[6], r[6],
                l[7], r[7], l[8], r[8], l[9], r[9], l[10], r[10], l[11], r[11], l[12], r[12],
                l[13], r[13],
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
            let l = self.left_state.decode_frame(left_frame);
            let r = self
                .right_state
                .as_mut()
                .unwrap_or_else(|| unreachable!())
                .decode_frame(right_frame);
            self.buffer.extend_from_slice(&[
                l[0], r[0], l[1], r[1], l[2], r[2], l[3], r[3], l[4], r[4], l[5], r[5], l[6], r[6],
                l[7], r[7], l[8], r[8], l[9], r[9], l[10], r[10], l[11], r[11], l[12], r[12],
                l[13], r[13],
            ]);
            self.frames_remaing -= 2;
        }
        self.buffer.pop().map(Ok)
    }
}
