use gpui::*;
use lilt::*;
use std::time::Instant;

struct AnimationExample {
    animation: Animated<bool, Instant>,
}

impl Render for AnimationExample {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let nd = self.animation.in_progress(Instant::now());
        div().flex().flex_col().size_full().justify_around().child(
            div().flex().flex_row().w_full().justify_around().child(
                div()
                    .flex()
                    .bg(rgb(0x2e7d32))
                    .size(Length::Definite(DefiniteLength::Absolute(
                        AbsoluteLength::Pixels(Pixels(self.animation.animate_bool(
                            100.,
                            200.,
                            Instant::now(),
                        ))),
                    )))
                    .justify_center()
                    .items_center()
                    .shadow_lg()
                    .text_xl()
                    .text_color(black())
                    .redraw_if("my_id", nd),
            ),
        )
    }
}

pub fn main() {
    App::new().run(|cx: &mut AppContext| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(300.), px(300.)),
                cx,
            ))),
            ..Default::default()
        };
        cx.open_window(options, |cx| {
            cx.activate(false);
            cx.new_view(|_cx| AnimationExample {
                animation: Animated::<_, Instant>::new(false)
                    .duration(400.)
                    .auto_start(true, Instant::now())
                    .repeat(2)
                    .auto_reverse(),
            })
        })
        .unwrap();
    });
}

// Custom view to ensure redraw while the animation runs
pub trait RedrawExt {
    fn redraw_if(self, id: impl Into<ElementId>, needs_redraw: bool) -> RedrawingElement<Self>
    where
        Self: Sized,
    {
        RedrawingElement {
            id: id.into(),
            element: Some(self),
            needs_redraw,
        }
    }
}

impl<E> RedrawExt for E {}

pub struct RedrawingElement<E> {
    id: ElementId,
    element: Option<E>,
    needs_redraw: bool,
}

impl<E: IntoElement + 'static> IntoElement for RedrawingElement<E> {
    type Element = RedrawingElement<E>;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E: IntoElement + 'static> Element for RedrawingElement<E> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (LayoutId, Self::RequestLayoutState) {
        cx.with_element_state(global_id.unwrap(), |_, cx| {
            let mut element = self
                .element
                .take()
                .expect("should only be called once")
                .into_any_element();

            if self.needs_redraw {
                let parent_id = cx.parent_view_id();
                cx.on_next_frame(move |cx| {
                    if let Some(parent_id) = parent_id {
                        cx.notify(parent_id)
                    } else {
                        cx.refresh()
                    }
                })
            }

            ((element.request_layout(cx), element), Option::<()>::None)
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        element.prepaint(cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        element.paint(cx);
    }
}
