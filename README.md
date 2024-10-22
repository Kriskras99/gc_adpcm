# `gc_adpcm`
A decoder for the ADPCM bitstream used by Nintendo on the GameCube, Wii, and WiiU.

## Usage
There are two ways to use this crate:
1. The `Decoder` API, which takes a `std::io::Read` and is an iterator that produces `i16`s (requires the `std` feature).
   This API hides intricacies of stereo interleaving.
2. The `DspState` API, where you manually push frames. This means you also need to take care of any interleaving yourself.

### `Decoder` example
```rust
use gc_adpcm::{Decoder, DspState};

fn mono() {
    // There is only one audio stream
    let dsp_header: DspState = todo!(); // get the metadata from your file format
    let total_frames: u32 = todo!(); // total number of frames expected
    let reader = todo!(); // something that implements Read
    let decoder = Decoder::mono(reader, dsp_header, total_frames);
    for sample in decoder {
        let sample = sample?;
        // do something with the sample
    }
}

fn stereo() {
    // There are two separate audio streams
    let left_dsp_header: DspState = todo!(); // get the metadata from your file format
    let left_reader = todo!(); // something that implements Read
    let right_dsp_header: DspState = todo!(); // get the metadata from your file format
    let right_reader = todo!(); // something that implements Read
    let total_frames: u32 = todo!(); // total number of frames expected for one channel
    let decoder = Decoder::stereo(left_reader, left_dsp_header, right_reader, right_dsp_header, total_frames);
    // the samples are interleaved per sample, not per frame!
    for sample in decoder {
        let sample = sample?;
        // do something with the sample
    }
}

fn stereo_interleaved() {
    // There is one audio stream with two channels
    let left_dsp_header: DspState = todo!(); // get the metadata from your file format
    let right_dsp_header: DspState = todo!(); // get the metadata from your file format
    let reader = todo!(); // something that implements Read
    let total_frames: u32 = todo!(); // total number of frames expected in the stream (thus total_frames_left + total_frames_right!)
    let decoder = Decoder::stereo_interleaved(reader, left_dsp_header, right_dsp_header, total_frames);
    // the samples are interleaved per sample, not per frame!
    for sample in decoder {
        let sample = sample?;
        // do something with the sample
    }
}
```

### `DspState` example
```rust
use gc_adpcm::DspState;

fn manual() {
    let mut dsp_state: DspState = todo!(); // get the metadata from your file format
    let frame: [u8; 8] = todo!(); // get a frame
    let samples = dsp_state.decode_frame(frame); // decode the frame
    // It is important that frames are processed sequentially!
    // To decode frame n, you need to have decoded frame n-1.
}
```

## Features
### `std`
The `std` enables the `Decoder` API. This feature is enabled by default. Disabling this feature makes this crate `no_std` compatible.
