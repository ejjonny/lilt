# Lilt

![rust](https://github.com/ejjonny/lilt/actions/workflows/rust.yml/badge.svg)

A simple, dependency free library for running interruptable, transition based animations as a function of time.

This library only implements animations & would be most useful along with a GUI library that can do GUI things (like [iced](https://github.com/iced-rs/iced)).

## Getting Started

Embed the state you want to animate in an `Animated` struct.

```rust
struct MyViewState {
    animated_toggle: Animated<bool, Instant>,
}
```

When you initialize your view state - choose the duration & easing you want.

```rust
let mut state = MyViewState {
    animated_toggle: Animated::new(false),
};
```

When your state needs an update, call the `transition` function on your animated state, passing the current time.

```rust
let now = std::time::Instant::now();
state
    .animated_toggle
    .transition(!state.animated_toggle.value, now);
```

While rendering a view based on your state - use the `interpolate` function on your animated state to get the in-between value for the current frame.

```rust
let now = std::time::Instant::now();
// Use the animated float for something like width, height, offset
let animated_width = self.animated_toggle.animate(100., 500., now)
// Or add an `Interpolable` implementation to an object of your choice, like a color
let animated_color = self.animated_toggle.animate(my_color_a, my_color_b, now)
```

## [Examples](examples/)
