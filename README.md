# Lilt

<img src="https://github.com/ejjonny/lilt/assets/17223924/2c5c1971-f10e-4766-9252-0ff8194e3e5d" width="100%">

![rust](https://github.com/ejjonny/lilt/actions/workflows/rust.yml/badge.svg)
[![coverage](https://codecov.io/github/ejjonny/lilt/main/graph/badge.svg?token=4XJNXRSQNX)](https://codecov.io/github/ejjonny/lilt)
[![crates.io](https://img.shields.io/crates/v/lilt.svg)](https://crates.io/crates/lilt)
[![downloads](https://img.shields.io/crates/d/lilt.svg)](https://crates.io/crates/lilt)
[![license](https://img.shields.io/crates/l/lilt.svg)](https://github.com/lilt/blob/master/LICENSE)

A simple, dependency free library for running interruptable, transition based animations as a function of time.

This library only implements animations & would be most useful along with a GUI library that can do GUI things (like [iced](https://github.com/iced-rs/iced)).

## Getting Started

### Define

Embed the state you want to animate in an `Animated` struct.

```rust
struct MyViewState {
    toggle: Animated<bool, Instant>,
}
```

When you initialize your view state - define the initial state & configure the animation to your liking.

```rust
let mut state = MyViewState {
    toggle: Animated::new(false)
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
    .toggle
    .transition(!state.animated_toggle.value, now);
```

### Render

While rendering a view based on your state - use the `animate` function on your state to get the interpolated value for the current frame.

```rust
let now = std::time::Instant::now();

// The wrapped value can be used to interpolate any values that implement `Interpolable`
let animated_width = self.toggle.animate_bool(100., 500., now);

// If the wrapped value itself is `Interpolable`, it can easily be interpolated in place
let animated_width = self.width.animate_wrapped(now);

// There are plenty of `animate` methods for interpolating things based on the wrapped value.
```

### What's the point?

lilt emerged from the need for ELM compatible / reactive animations.

The animations modeled by this library don't require periodic mutation like a 'tick' function - all interim states of the animation are predefined when 'transition' is called, & then accessed while rendering based on the current time.

lilt animations are fully independent of frame rate or tick frequency & only need to be computed if they're used during rendering.

## [Examples](examples/)

![indicator](https://github.com/ejjonny/lilt/assets/17223924/e4f81d63-67a4-4586-a2cf-309c687fd59d)
