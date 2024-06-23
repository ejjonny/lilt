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

/// Adds vector operations to arbitrary types
pub trait AnimatableValue<T = Self>
where
    Self: Clone + std::fmt::Debug + PartialEq + Sized,
{
    fn distance(&self, other: &Self) -> f32;
    fn diff(&self, other: &Self) -> Self;
    fn sum(&self, other: &Self) -> Self;
    fn scale(&self, amount: f32) -> Self;
    fn magnitude(&self) -> f32;
    fn normalized(&self) -> Self;
}

impl AnimatableValue for (f32, f32) {
    fn distance(&self, other: &Self) -> f32 {
        self.diff(other).magnitude()
    }
    fn diff(&self, other: &Self) -> Self {
        (self.0 - other.0, self.1 - other.1)
    }
    fn sum(&self, other: &Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }
    fn scale(&self, amount: f32) -> Self {
        (self.0 * amount, self.1 * amount)
    }
    fn magnitude(&self) -> f32 {
        f32::sqrt(
            vec![self.0, self.1]
                .iter()
                .map(|v| f32::powf(*v, 2.0))
                .sum(),
        )
    }
    fn normalized(&self) -> Self {
        let magnitude = self.magnitude();
        self.scale(1.0 / magnitude)
    }
}

impl AnimatableValue for f32 {
    fn distance(&self, other: &Self) -> f32 {
        self.diff(other).magnitude()
    }
    fn diff(&self, other: &Self) -> Self {
        self - other
    }
    fn sum(&self, other: &Self) -> Self {
        self + other
    }
    fn scale(&self, amount: f32) -> Self {
        self * amount
    }
    fn magnitude(&self) -> f32 {
        f32::sqrt(f32::powf(*self, 2.0))
    }
    fn normalized(&self) -> Self {
        self / self.magnitude()
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
