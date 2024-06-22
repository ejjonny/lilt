// use std::marker::PhantomData;

// use iced::advanced::layout;
// use iced::advanced::renderer;
// use iced::advanced::widget::Operation;
// use iced::advanced::widget::Tree;
// use iced::advanced::Clipboard;
// use iced::advanced::Layout;
// use iced::advanced::Renderer as IcedRenderer;
// use iced::advanced::Shell;
// use iced::advanced::Widget;
// use iced::event;
// use iced::mouse;
// use iced::window;
// use iced::window::RedrawRequest;
// use iced::Element;
// use iced::Event;
// use iced::Length;
// use iced::Rectangle;

// use crate::animation::*;

// pub struct Animator<'a, Message, Renderer, Redrawable, Time> {
//     redrawable: Redrawable,
//     time: Time,
//     child: Box<dyn Fn() -> Element<'a, Message, Renderer>>,
// }

// pub trait Redrawable<Time>: Copy {
//     fn needs_redraw(self, time: Time) -> bool;
// }

// impl<'a, Message, T, Renderer, Time> Animator<'a, Message, T, Renderer, Time>
// where
//     T: AnimatableValue,
//     Time: AnimationTime
// {
//     pub fn duration(mut self, duration: f32) -> Self {
//         self.animation.duration_ms = duration;
//         self
//     }

//     pub fn timing(mut self, timing: Timing) -> Self {
//         self.animation.timing = timing;
//         self
//     }
// }

// impl<'a, Message, Renderer, Redrawable, Time> Animator<'a, Message, Renderer, Redrawable, Time> {
//     pub fn new<Content>(redrawable: Redrawable, time: Time, child: Content) -> Self
//     where
//         Content: Fn() -> Element<'a, Message, Renderer> + 'static,
//         Time: AnimationTime,
//     {
//         Animator {
//             redrawable,
//             child: Box::new(child),
//             time,
//         }
//     }
// }

// impl<'a, 'b, Message, Renderer, R, Time> Widget<Message, Renderer>
//     for Animator<'a, Message, Renderer, R, Time>
// where
//     Renderer: IcedRenderer,
//     R: Redrawable<Time>,
//     Time: AnimationTime,
// {
//     fn draw(
//         &self,
//         state: &Tree,
//         renderer: &mut Renderer,
//         theme: &<Renderer as iced::advanced::renderer::Renderer>::Theme,
//         style: &renderer::Style,
//         layout: Layout<'_>,
//         cursor: mouse::Cursor,
//         viewport: &Rectangle,
//     ) {
//         (self.child)().as_widget().draw(
//             &state.children[0],
//             renderer,
//             theme,
//             style,
//             layout,
//             cursor,
//             viewport,
//         )
//     }
//     fn mouse_interaction(
//         &self,
//         state: &Tree,
//         layout: Layout<'_>,
//         cursor: mouse::Cursor,
//         viewport: &Rectangle,
//         renderer: &Renderer,
//     ) -> mouse::Interaction {
//         (self.child)()
//             .as_widget()
//             .mouse_interaction(state, layout, cursor, viewport, renderer)
//     }
//     fn on_event(
//         &mut self,
//         tree: &mut Tree,
//         event: Event,
//         layout: Layout<'_>,
//         cursor: mouse::Cursor,
//         renderer: &Renderer,
//         clipboard: &mut dyn Clipboard,
//         shell: &mut Shell<'_, Message>,
//         viewport: &Rectangle,
//     ) -> event::Status {
//         if let Event::Window(window::Event::RedrawRequested(..)) = event {
//             if self.redrawable.needs_redraw(self.time) {
//                 shell.request_redraw(RedrawRequest::NextFrame);
//             }
//         }
//         (self.child)().as_widget_mut().on_event(
//             &mut tree.children[0],
//             event,
//             layout,
//             cursor,
//             renderer,
//             clipboard,
//             shell,
//             viewport,
//         )
//     }
//     fn operate(
//         &self,
//         state: &mut Tree,
//         layout: Layout<'_>,
//         renderer: &Renderer,
//         operation: &mut dyn Operation<Message>,
//     ) {
//         (self.child)()
//             .as_widget()
//             .operate(state, layout, renderer, operation)
//     }
//     fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
//         (self.child)().as_widget().layout(renderer, limits)
//     }
//     fn width(&self) -> Length {
//         (self.child)().as_widget().width()
//     }
//     fn height(&self) -> Length {
//         (self.child)().as_widget().height()
//     }
//     fn diff(&self, tree: &mut Tree) {
//         tree.diff_children(&vec![(self.child)()]);
//     }
//     fn children(&self) -> Vec<Tree> {
//         vec![Tree::new((self.child)())]
//     }
// }

// impl<'a, Message, Renderer, R, Time> From<Animator<'a, Message, Renderer, R, Time>>
//     for Element<'a, Message, Renderer>
// where
//     Message: 'a,
//     Renderer: IcedRenderer + 'a,
//     R: Redrawable<Time> + 'a,
//     Time: AnimationTime + 'a,
// {
//     fn from(animating: Animator<'a, Message, Renderer, R, Time>) -> Self {
//         Self::new(animating)
//     }
// }
