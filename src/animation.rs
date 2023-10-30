use iced_core::Color;

pub trait AnimatableValue<T = Self> where Self: Clone + std::fmt::Debug + PartialEq + Sized {
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
        f32::sqrt(vec![self.0, self.1].iter().map(|v| f32::powf(*v, 2.0)).sum())
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

#[derive(Default, Debug, Clone, Copy)]
pub struct Animation<Time, Value>
where
    Value: AnimatableValue,
{
    pub position: Value,
    pub duration_ms: f32,
    pub timing: Timing,
    pub animation_state: Option<AnimationState<Time, Value>>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct AnimationState<Time, Value> {
    pub origin: Value,
    pub destination: Value,
    pub started_time: Time,
    pub last_tick_time: Time,
    pub speed_at_interrupt: Option<f32>,
}

pub trait AnimationTime: Copy {
    fn elapsed_since(self, time: Self) -> f32;
}

impl AnimationTime for std::time::Instant {
    fn elapsed_since(self, time: Self) -> f32 {
        (self - time).as_millis() as f32
    }
}

impl<Time, Value> Animation<Time, Value>
where
    Time: AnimationTime + std::fmt::Debug,
    Value: AnimatableValue,
{
    pub fn new(position: Value, duration: f32, timing: Timing) -> Self {
        Animation {
            position,
            duration_ms: duration,
            timing,
            animation_state: None,
        }
    }

    pub fn transition(&mut self, destination: Value, time: Time) {
        let timed_progress = self.timed_progress();
        if let Some(animation) = &mut self.animation_state {
            // Snapshot current state as the new animation origin
            if animation.speed_at_interrupt.is_none() {
                animation.speed_at_interrupt = Some(
                    animation.destination.distance(&animation.origin)
                        / self.duration_ms,
                );
            }
            animation.origin = timed_progress;
            self.position = animation.origin.clone();
            animation.destination = destination;
        } else {
            self.animation_state = Some(AnimationState {
                started_time: time,
                last_tick_time: time,
                origin: self.position.clone(),
                destination,
                speed_at_interrupt: None,
            })
        }
    }

    pub fn tick(&mut self, time: Time) -> bool {
        if let Some(animation) = &mut self.animation_state {
            let elapsed = time.elapsed_since(animation.last_tick_time);
            let position_delta: Value;
            if let Some(speed) = animation.speed_at_interrupt {
                let direction = animation.destination.diff(&self.position).normalized();
                position_delta = direction.scale(elapsed * speed);
            } else {
                let duration = self.duration_ms;
                let delta = elapsed / duration;
                let direction = animation.destination.diff(&animation.origin);
                position_delta = direction.scale(delta);
            }
            let mut finished = false;
            if self.duration_ms == 0.0 {
                finished = true;
            } else {
                if position_delta.magnitude() >= self.position.distance(&animation.destination) {
                    finished = true
                }
                self.position = self.position.sum(&position_delta);
            }
            animation.last_tick_time = time;
            if finished {
                self.position = animation.destination.clone();
                self.animation_state = None;
            }
            return true;
        };
        false
    }

    pub fn timed_progress(&self) -> Value {
        match &self.animation_state {
            Some(animation) if animation.destination != animation.origin => {
                let progress_in_animation = self.position.distance(&animation.origin);
                let range_of_animation = animation.destination.distance(&animation.origin);
                let completion = progress_in_animation / range_of_animation;
                let animation_range = animation.destination.diff(&animation.origin);
                animation.origin.sum(&animation_range.scale(self.timing.timing(completion)))
            }
            _ => return self.position.clone(),
        }
    }

    pub fn animating(&self) -> bool {
        self.animation_state.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Timing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    Custom,
}

impl Timing {
    fn timing(self, linear_progress: f32) -> f32 {
        let x = linear_progress;
        let pi = std::f32::consts::PI;
        match self {
            Timing::Linear => linear_progress,
            Timing::EaseIn => 1.0 - f32::cos((x * pi) / 2.0),
            Timing::EaseOut => f32::sin((x * pi) / 2.0),
            Timing::EaseInOut => -(f32::cos(pi * x) - 1.0) / 2.0,
            Timing::EaseInQuint => x * x * x * x * x,
            Timing::EaseOutQuint => 1.0 - f32::powf(1.0 - x, 5.0),
            Timing::EaseInOutQuint => {
                if x < 0.5 {
                    16.0 * x * x * x * x * x
                } else {
                    1.0 - f32::powf(-2.0 * x + 2.0, 5.0) / 2.0
                }
            }
            _ => linear_progress,
        }
    }
}

pub trait Interpolable {
    fn interpolated(self, other: Self, ratio: f32) -> Self;
}

impl Interpolable for Color {
    fn interpolated(self, other: Self, ratio: f32) -> Self {
        if ratio >= 1.0 {
            return other;
        } else if ratio > 0.0 {
            return Color::new(
                self.r.interpolated(other.r, ratio),
                self.g.interpolated(other.g, ratio),
                self.b.interpolated(other.b, ratio),
                self.a.interpolated(other.a, ratio),
            )
        } else {
            return self
        }
    }
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


#[cfg(test)]
mod animatedvalue_tests {
    use super::*;

    #[test]
    fn test_instant_animation() {
        let mut anim = Animation::<f32, f32>::new(0.0, 1.0, Timing::Linear);
        let clock = 0.0;
        // If animation duration is 0.0 the transition should happen instantly
        // & require a redraw without any time passing
        assert_eq!(anim.position, 0.0);
        anim.transition(10.0, clock);
        assert_eq!(anim.position, 0.0);
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 10.0);
    }

    #[test]
    fn test_progression() {
        let mut anim = Animation::<f32, f32>::new(0.0, 1.0, Timing::Linear);
        let mut clock = 0.0;
        // With a duration of 1.0 & linear timing we should be halfway to our
        // destination at 0.5
        anim.transition(10.0, clock);
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 5.0);
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 10.0);

        // Progression backward
        anim.duration_ms = 0.5;
        anim.transition(0.0, clock);
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 0.0);

        // Progression forward in thirds
        anim.duration_ms = 1.0;
        anim.transition(10.0, clock);
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(approximately_equal(anim.position, 2.0));
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(approximately_equal(anim.position, 4.0));
        clock += 0.4;
        assert!(anim.tick(clock));
        assert!(approximately_equal(anim.position, 8.0));
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(approximately_equal(anim.position, 10.0));
    }

    #[test]
    fn test_interrupt() {
        let mut anim = Animation::<f32, f32>::new(0.0, 1.0, Timing::Linear);
        let mut clock = 0.0;
        // Interruptions should continue at the same speed the interrupted
        // animation was progressing at.
        anim.transition(10.0, clock);
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 5.0);
        // If we interrupt exactly halfway through distance & duration we
        // should arrive back at the start with another half of the duration
        anim.transition(0.0, clock);
        clock += 0.5;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 0.0);
        assert!(!anim.animating());

        // Begin an animation
        anim.transition(10.0, clock);
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(anim.animating());
        assert!(approximately_equal(anim.position, 2.0));
        // Interrupt one fifth of the way through
        // The animation is playing at 10 units per time unit
        // The target is only 1.0 away
        // We should arrive at the target after 0.1 time units
        anim.transition(1.0, clock);
        clock += 0.100001;
        dbg!(anim.position);
        assert!(anim.tick(clock));
        dbg!(anim.position);
        assert!(!anim.animating());
        assert!(approximately_equal(anim.position, 1.0));
    }

    #[test]
    fn test_interrupt_nonlinear() {
        let mut anim = Animation::<f32, f32>::new(1.0, 10.0, Timing::EaseIn);
        let mut clock = 0.0;

        // Interrupt halfway through with asymmetrical timing
        anim.transition(0.0, clock);
        assert!(anim.animating());
        assert_eq!(anim.position, 1.0);
        clock += 1.0;
        assert!(anim.tick(clock));
        let progress_at_interrupt = anim.timed_progress();
        assert_eq!(progress_at_interrupt, 1.0 - Timing::EaseIn.timing(0.1));

        // Interrupted animation should begin from wherever the timed function
        // was interrupted, which is different from the linear progress.
        anim.transition(1.0, clock);
        assert_eq!(anim.animation_state.unwrap().destination, 1.0);
        assert_eq!(anim.timed_progress(), progress_at_interrupt);
        assert!(anim.animating());
        assert!(anim.animation_state.unwrap().speed_at_interrupt.is_some());
        // Since we've interrupted at some in-between, non-linear point in
        // the animation, the time it takes to finish won't be as clean.
        // It should take a bit less time to return home because it's an
        // EaseIn timing curve. The animation we interrupted was easing in
        // & therefore closer to where it started.
        clock += 3.0;
        assert!(anim.tick(clock));
        assert_eq!(anim.position, 1.0);
        assert!(!anim.animating());
    }

    #[test]
    fn test_multiple_interrupts_start_forward() {
        let mut anim = Animation::<f32, f32>::new(0.0, 1.0, Timing::EaseInOut);
        let mut clock = 0.0;
        anim.transition(1.0, clock);
        clock += 0.5;
        assert!(anim.tick(clock));
        assert!(anim.animating());
        let progress_at_interrupt = anim.timed_progress();
        assert_eq!(progress_at_interrupt, Timing::EaseInOut.timing(0.5));
        anim.transition(0.0, clock);
        clock += 0.2;
        assert_eq!(anim.timed_progress(), progress_at_interrupt);
        assert!(anim.tick(clock));
        assert!(anim.animating());
        anim.transition(1.0, clock);
        clock += 0.2;
        assert!(anim.tick(clock));
        assert!(anim.animating());
    }

    impl AnimationTime for f32 {
        fn elapsed_since(self, time: Self) -> f32 {
            self - time
        }
    }

    fn approximately_equal(a: f32, b: f32) -> bool {
        let close = f32::abs(a - b) < 1e-5;
        if !close {
            dbg!(a, b);
        }
        close
    }
}
