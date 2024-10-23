# Changelog

## 0.2.0
- **Breaking**: `StereoInterleaved`'s total frames is for one channel.
- **Breaking**: Fix the sample order. The order was reversed per frame, causing the decoded audio to be useless.
- Clarify `total_frames` by renaming it to `channel_frames` (and `frames` for `Mono`).
- Add new constructors that take the amount of samples in a channel instead of frames.
- Add this changelog file.

## 0.1.1
- Add `authors` and `rust-version` keys to Cargo.toml.
- Move the `#![forbid(unsafe_code)]` lint to Cargo.toml.

## 0.1.0
- Initial release.
