use iced_core::widget;
use iced_core::layout;
use iced_core::event;
use iced_core::mouse;
use iced_core::renderer;
use iced_core::Element;
use iced_core::Length;
use iced_core::Layout;
use iced_core::Widget;
use iced_core::widget::Tree;
use iced_core::widget::tree::State;
use iced_core::Event;
use iced_core::Rectangle;
use iced_core::Shell;
use iced_core::Clipboard;

use crate::animation::*;

pub struct Animator<'a, Message, T, Renderer>
where
    T: AnimatableValue,
{
    child: Box<dyn Fn(T) -> Element<'a, Message, Renderer>>,
    animated_value: T,
    duration: std::time::Duration,
    timing: Timing,
}

pub trait AnimatableConvertible<T>
where
    T: AnimatableValue,
{
    fn animatable(self) -> T;
}

impl AnimatableConvertible<f32> for bool {
    fn animatable(self) -> f32 {
        if self {
            1.0
        } else {
            0.0
        }
    }
}

impl<'a, Message, T, Renderer> Animator<'a, Message, T, Renderer>
where
    T: AnimatableValue,
{
    pub fn new<Content>(
        animated_value: T,
        duration: std::time::Duration,
        timing: Timing,
        child: Content,
    ) -> Self
    where
        Content: Fn(T) -> Element<'a, Message, Renderer> + 'static,
    {
        Animator {
            child: Box::new(child),
            animated_value,
            duration,
            timing,
        }
    }
}

impl<'a, 'b, Message, T, Renderer> Widget<Message, Renderer>
    for Animator<'a, Message, T, Renderer>
where
    T: AnimatableValue + Clone + 'static,
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> iced_renderer::core::widget::tree::Tag {
        widget::tree::Tag::of::<Animation<std::time::Instant, T>>()
    }
    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_renderer::core::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        // let animation = state
        //     .state
        //     .downcast_ref::<Animation<std::time::Instant, T>>()
        //     .timed_progress();
        (self.child)(self.animated_value.clone()).as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }
    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        // let animation = state
        //     .state
        //     .downcast_ref::<Animation<std::time::Instant, T>>()
        //     .timed_progress();
        (self.child)(self.animated_value.clone())
            .as_widget()
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        // let animation = tree
        //     .state
        //     .downcast_mut::<Animation<std::time::Instant, T>>();
        // match event {
        //     Event::Window(window::Event::RedrawRequested(now)) => {
        //         match &animation.animation_state {
        //             Some(animation_state) => {
        //                 if animation_state.destination != self.animated_value {
        //                     animation
        //                         .transition(self.animated_value.clone(), now);
        //                 }
        //             }
        //             _ => {
        //                 if animation.position != self.animated_value {
        //                     animation
        //                         .transition(self.animated_value.clone(), now)
        //                 }
        //             }
        //         }
        //         if animation.animating() {
        //             let needs_redraw = animation.tick(now);
        //             if needs_redraw {
        //                 shell.invalidate_layout();
        //                 shell.request_redraw(window::RedrawRequest::NextFrame);
        //             }
        //         }
        //     }
        //     _ => {}
        // }
        // (self.child)(animation.timed_progress())
        //     .as_widget_mut()
        //     .on_event(
        //         &mut tree.children[0],
        //         event,
        //         layout,
        //         cursor,
        //         renderer,
        //         clipboard,
        //         shell,
        //         viewport,
        //     )
        event::Status::Ignored
    }
    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_renderer::core::widget::Operation<Message>,
    ) {
        // let animation = state
        //     .state
        //     .downcast_ref::<Animation<std::time::Instant, T>>()
        //     .timed_progress();
        (self.child)(self.animated_value.clone())
            .as_widget()
            .operate(state, layout, renderer, operation)
    }
    fn state(&self) -> State {
        let animation = Animation::<std::time::Instant, T>::new(
            self.animated_value.clone(),
            self.duration.as_millis() as f32,
            self.timing,
        );
        State::new(animation)
    }
    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // let animation = tree
        //     .state
        //     .downcast_ref::<Animation<std::time::Instant, T>>()
        //     .timed_progress();
        (self.child)(self.animated_value.clone()).as_widget().layout(
            &mut tree.children[0],
            renderer,
            limits,
        )
    }
    fn width(&self) -> Length {
        (self.child)(self.animated_value.clone())
            .as_widget()
            .width()
    }
    fn height(&self) -> Length {
        (self.child)(self.animated_value.clone())
            .as_widget()
            .height()
    }
    fn diff(&self, tree: &mut Tree) {
        // let animation = tree
        //     .state
        //     .downcast_ref::<Animation<std::time::Instant, T>>()
        //     .timed_progress();
        // tree.diff_children(&vec![(self.child)(animation)]);
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new((self.child)(self.animated_value.clone()))]
    }
}

impl<'a, Message, T, Renderer> From<Animator<'a, Message, T, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    T: AnimatableValue + Copy + 'static,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(animating: Animator<'a, Message, T, Renderer>) -> Self {
        Self::new(animating)
    }
}
