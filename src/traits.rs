/// An interface for interacting with time.
pub trait AnimationTime: Copy + std::fmt::Debug + Send {
    type Duration;
    fn elapsed_since(self, time: Self) -> f32;
    fn advanced_by(self, duration_ms: f32) -> Self;
}

impl AnimationTime for std::time::Instant {
    type Duration = std::time::Duration;
    fn elapsed_since(self, time: Self) -> f32 {
        (self - time).as_millis() as f32
    }
    fn advanced_by(self, duration_ms: f32) -> Self {
        let duration = std::time::Duration::from_secs_f32(f32::abs(duration_ms) * 1000.);
        if duration_ms.is_sign_positive() {
            self + duration
        } else {
            self - duration
        }
    }
}

/// Defines a float representation for arbitrary types
///
/// The actual float values are pretty arbitrary - as interpolation from
/// one float to another will usually look the same.
/// This simply correlates values with a "location"
/// that can be interpolated towards.
pub trait FloatRepresentable {
    fn float_value(&self) -> f32;
}

impl FloatRepresentable for bool {
    fn float_value(&self) -> f32 {
        if *self {
            1.
        } else {
            0.
        }
    }
}

/// A type implementing `Interpolable` can be used with `Animated<T>.interpolate(...)`
pub trait Interpolable {
    fn interpolated(self, other: Self, ratio: f32) -> Self;
}

impl Interpolable for f32 {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        self * (1.0 - ratio) + other * ratio
    }
}

impl<T> Interpolable for Option<T>
where
    T: Interpolable + Copy,
{
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.interpolated(b, ratio)),
            _ => other,
        }
    }
}
