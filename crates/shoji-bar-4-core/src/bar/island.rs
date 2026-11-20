use std::rc::Rc;

use anti_webui::{
    core::{
        reactive::{memo, signal},
        view::{Div, IntoView, View, div},
    },
    style::{Length, Styled},
};

use crate::{
    context::{Cx, IslandState, Part},
    placement::BarPlacement,
    theme::{colors, elevation},
    ui::pressable,
};

#[derive(Clone)]
pub struct Island {
    pub id: String,
    pub content: Part,
    frame: Option<Frame>,
}

#[derive(Clone, Default)]
struct Frame {
    padding: (f32, f32),
    active: Option<Rc<dyn Fn(&Cx) -> bool>>,
    on_click: Option<Rc<dyn Fn(&Cx)>>,
}

impl Island {
    /// The frame, created the first time one of its fields is set.
    fn frame_mut(&mut self) -> &mut Frame {
        self.frame.get_or_insert_with(Frame::default)
    }

    pub fn new(id: impl Into<String>, build: impl Fn(&Cx) -> View + 'static) -> Self {
        Self {
            id: id.into(),
            content: Rc::new(build),
            frame: None,
        }
    }

    pub fn padding(mut self, horizontal: f32, vertical: f32) -> Self {
        self.frame_mut().padding = (horizontal, vertical);
        self
    }

    pub fn active(mut self, f: impl Fn(&Cx) -> bool + 'static) -> Self {
        self.frame_mut().active = Some(Rc::new(f));
        self
    }

    pub fn on_click(mut self, f: impl Fn(&Cx) + 'static) -> Self {
        self.frame_mut().on_click = Some(Rc::new(f));
        self
    }

    pub fn view(&self, cx: &Cx, index: usize) -> View {
        let Some(frame) = &self.frame else {
            return (self.content)(cx); // no frame: the island draws itselft, as today
        };

        let hovered = signal(false);
        let active = {
            let (f, cx) = (frame.active.clone(), cx.clone());
            memo(move || f.as_ref().is_some_and(|f| f(&cx)))
        };

        let cx = cx.with_island(IslandState {
            hovered,
            active,
            index,
        });

        let (horizontal, vertical) = frame.padding;
        let node = bar_island(cx.placement);
        let node = if cx.placement.is_vertical() {
            node.py(vertical)
        } else {
            node.px(horizontal)
        };
        let node = node
            .on_hover(move |h| hovered.set(h))
            .dyn_style(move |s| {
                let c = colors();
                if active.get() {
                    s.bg(c.primary_container).text_color(c.on_primary_container)
                } else {
                    s
                }
            })
            .child((self.content)(&cx));

        match frame.on_click.clone() {
            Some(on_click) => {
                let cx = cx.clone();
                pressable(node.on_click(move |_| on_click(&cx)), |c| c.on_surface).into_view()
            }
            None => node.into_view(),
        }
    }
}

// Aux
pub const VERTICAL_ISLAND: f32 = 40.0;
/// A floating container of the bar: a pill on `surface_container`.
/// How tall an island of the bar is.
pub const ISLAND: f32 = 32.0;

/// A horizontal pill, or a vertical island with its height set by its contents.
pub fn bar_island(at: BarPlacement) -> Div {
    let node = island();
    if at.is_vertical() {
        node.flex_col()
            .w(VERTICAL_ISLAND)
            .h(Length::Auto)
            .rounded(VERTICAL_ISLAND / 2.0)
    } else {
        node
    }
}

pub fn island() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .flex_none()
        .h(ISLAND)
        .rounded(ISLAND / 2.0)
        .shadows(elevation(1))
        .dyn_style(|s| s.bg(colors().surface_container))
}
