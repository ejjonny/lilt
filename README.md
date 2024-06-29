# Lilt

<img src="https://github.com/ejjonny/lilt/assets/17223924/2c5c1971-f10e-4766-9252-0ff8194e3e5d" width="100%">

![rust](https://github.com/ejjonny/lilt/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/lilt.svg)](https://crates.io/crates/lilt)
[![Downloads](https://img.shields.io/crates/d/lilt.svg)](https://crates.io/crates/lilt)
[![License](https://img.shields.io/crates/l/lilt.svg)](https://github.com/lilt/blob/master/LICENSE)

A simple, dependency free library for running interruptable, transition based animations as a function of time.

This library only implements animations & would be most useful along with a GUI library that can do GUI things (like [iced](https://github.com/iced-rs/iced)).

## Getting Started

### Define

Embed the state you want to animate in an `Animated` struct.

```rust
struct MyViewState {
    animated_toggle: Animated<bool, Instant>,
}
```

When you initialize your view state - define the initial state & configure the animation to your liking.

```rust
let mut state = MyViewState {
    animated_toggle: Animated::new(false)
        .duration(300.)
        .easing(Easing::EaseOut)
        .delay(30.)
        .repeat(3),
};
```

### Transition

When your state needs an update, call the `transition` function on your animated state, passing the current time.

```rust
let now = std::time::Instant::now();
state
    .animated_toggle
    .transition(!state.animated_toggle.value, now);
```

### Render

While rendering a view based on your state - use the `animate` function on your state to get the interpolated value for the current frame.

```rust
let now = std::time::Instant::now();
// Use the animated float for something like width, height, offset
let animated_width = self.animated_toggle.animate(100., 500., now)
// Or add an `Interpolable` implementation to an object of your choice, like a color
let animated_color = self.animated_toggle.animate(my_color_a, my_color_b, now)
```

## [Examples](examples/)
