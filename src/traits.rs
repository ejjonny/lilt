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
/// one float to another will look the same, however in the case of
/// asymmetric animations the 'direction' of the animation is determined
/// using these float representations.
/// In general, this defines 'keyframes' & associates animated values on
/// a continuous axis so that transitions & interruptions can be represented.
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
    fn interpolated(&self, other: Self, ratio: f32) -> Self;
}

impl Interpolable for f32 {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        self * (1.0 - ratio) + other * ratio
    }
}

impl<T> Interpolable for Option<T>
where
    T: Interpolable + Copy,
{
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.interpolated(b, ratio)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_interpolation() {
        let start = 0.0f32;
        let end = 10.0f32;

        assert_eq!(start.interpolated(end, 0.0), 0.0);
        assert_eq!(start.interpolated(end, 0.5), 5.0);
        assert_eq!(start.interpolated(end, 1.0), 10.0);
        assert_eq!(start.interpolated(end, 0.25), 2.5);
        assert_eq!(start.interpolated(end, 0.75), 7.5);
    }

    #[test]
    fn test_option_f32_interpolation() {
        let start = Some(0.0f32);
        let end = Some(10.0f32);

        assert_eq!(start.interpolated(end, 0.0), Some(0.0));
        assert_eq!(start.interpolated(end, 0.5), Some(5.0));
        assert_eq!(start.interpolated(end, 1.0), Some(10.0));
        assert_eq!(start.interpolated(end, 0.25), Some(2.5));
        assert_eq!(start.interpolated(end, 0.75), Some(7.5));
    }

    #[test]
    fn test_option_f32_interpolation_with_none() {
        let start = Some(0.0f32);
        let end = None;

        assert_eq!(start.interpolated(end, 0.0), None);
        assert_eq!(start.interpolated(end, 0.5), None);
        assert_eq!(start.interpolated(end, 1.0), None);

        let start = None;
        let end = Some(10.0f32);

        assert_eq!(start.interpolated(end, 0.0), None);
        assert_eq!(start.interpolated(end, 0.5), None);
        assert_eq!(start.interpolated(end, 1.0), None);

        let start: Option<f32> = None;
        let end: Option<f32> = None;
        assert_eq!(start.interpolated(end, 0.0), None);
        assert_eq!(start.interpolated(end, 0.5), None);
        assert_eq!(start.interpolated(end, 1.0), None);
    }
}
