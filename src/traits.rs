/// An interface for interacting with time.
pub trait AnimationTime: Copy + std::fmt::Debug + Send {
    fn elapsed_since(self, time: Self) -> f32;
}

impl AnimationTime for std::time::Instant {
    fn elapsed_since(self, time: Self) -> f32 {
        (self - time).as_millis() as f32
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

impl FloatRepresentable for f32 {
    fn float_value(&self) -> f32 {
        *self
    }
}

/// A type implementing `Interpolable` can be used with `Animated<T>.animate(...)`
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
