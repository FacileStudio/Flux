//! Volume math. PulseAudio volumes are cubic, so percentages already feel even.

use super::{BOOST_PERCENT, NORM};

/// A raw volume as a whole percentage, rounded.
pub fn percent(raw: u32) -> u32 {
    ((u64::from(raw) * 100 + u64::from(NORM) / 2) / u64::from(NORM)) as u32
}

/// A percentage as a raw volume.
pub fn raw(percent: u32) -> u32 {
    ((u64::from(percent) * u64::from(NORM) + 50) / 100) as u32
}

/// The highest percentage the sliders allow.
pub fn max_percent(boost: bool) -> u32 {
    if boost {
        BOOST_PERCENT
    } else {
        100
    }
}

/// `raw` moved by `delta` percent and clamped to what the sliders allow. A volume already
/// above the cap is never raised further, and lowering it is always allowed.
pub fn step(raw_volume: u32, delta: i32, boost: bool) -> u32 {
    let current = i64::from(percent(raw_volume));
    let cap = i64::from(max_percent(boost));
    let delta = i64::from(delta);
    let next = if delta >= 0 {
        (current + delta).min(cap.max(current))
    } else {
        (current + delta).max(0)
    };
    raw(next as u32)
}

/// The amplitude factor a percentage applies. PulseAudio volumes are cubic: 50% is 1/8 the
/// amplitude, which is what makes the slider feel even.
pub fn amplitude(percent: u32) -> f64 {
    (f64::from(percent) / 100.0).powi(3)
}

/// Where a peak (linear, 0 to 1) sits on a meter drawn on the same cubic scale as the slider.
pub fn meter(peak: f32) -> f64 {
    f64::from(peak).clamp(0.0, 1.0).cbrt()
}

/// The next meter position: rise at once, fall back gently, so the bar reads instead of flickers.
pub fn smooth(previous: f64, target: f64) -> f64 {
    if target >= previous {
        target
    } else {
        (previous * 0.8 + target * 0.2).max(target)
    }
}

/// The volume icon for a percentage.
pub fn volume_icon(percent: u32, muted: bool) -> &'static str {
    match percent {
        _ if muted => "audio-volume-muted-symbolic",
        0 => "audio-volume-muted-symbolic",
        1..=33 => "audio-volume-low-symbolic",
        34..=66 => "audio-volume-medium-symbolic",
        _ => "audio-volume-high-symbolic",
    }
}

/// The test sound as interleaved stereo samples: a short tone on the left, a pause, then the
/// same tone on the right, each faded in and out so it never clicks.
pub fn test_tone(rate: u32) -> Vec<f32> {
    let tone = (rate as usize * 7) / 20;
    let gap = rate as usize / 8;
    let fade = (rate as usize / 50).max(1);
    let sample = |frame: usize| {
        let edge = frame.min(tone.saturating_sub(frame + 1)) as f32 / fade as f32;
        let time = frame as f32 / rate as f32;
        0.3 * edge.min(1.0) * (std::f32::consts::TAU * 660.0 * time).sin()
    };
    let mut samples = Vec::with_capacity((tone * 2 + gap) * 2);
    samples.extend((0..tone).flat_map(|frame| [sample(frame), 0.0]));
    samples.extend(std::iter::repeat_n(0.0, gap * 2));
    samples.extend((0..tone).flat_map(|frame| [0.0, sample(frame)]));
    samples
}
