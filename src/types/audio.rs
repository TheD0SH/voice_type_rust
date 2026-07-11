//! Audio types for Voice Type
//!
//! Contains types related to audio recording state.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::audio::SAMPLE_RATE;

/// Audio recording state
///
/// Shared between the audio callback and the control logic.
/// Uses atomic operations for lock-free coordination.
#[derive(Debug)]
pub struct RecordingState {
    /// Set to false to stop recording
    pub running: AtomicBool,
    /// Collected audio samples (interleaved if stereo)
    pub samples: Mutex<Vec<i16>>,
    /// Total audio frames processed so far (for silence timing).
    pub frames_processed: std::sync::atomic::AtomicU64,
    /// Frames since the last frame above the noise threshold.
    pub silent_frames: std::sync::atomic::AtomicU64,
}

impl RecordingState {
    /// Create a new recording state
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(true),
            samples: Mutex::new(Vec::with_capacity(SAMPLE_RATE as usize * 60)),
            frames_processed: std::sync::atomic::AtomicU64::new(0),
            silent_frames: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Check if recording should continue
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// Stop the recording
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// Record a frame's level for auto-stop purposes.
    /// Call once per audio callback invocation with the peak level (0.0–1.0)
    /// and the number of frames in that invocation.
    pub fn record_level(&self, level: f32, frame_count: u64, noise_threshold: f32) {
        self.frames_processed
            .fetch_add(frame_count, Ordering::Relaxed);
        if level >= noise_threshold {
            self.silent_frames.store(0, Ordering::Relaxed);
        } else {
            self.silent_frames.fetch_add(frame_count, Ordering::Relaxed);
        }
    }

    /// Returns the number of consecutive silent frames so far.
    pub fn silent_frame_count(&self) -> u64 {
        self.silent_frames.load(Ordering::Relaxed)
    }
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::new()
    }
}
