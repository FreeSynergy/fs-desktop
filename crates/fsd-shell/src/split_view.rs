/// SplitView — resizable horizontal master/detail split panel.
use dioxus::prelude::*;

/// State of the split panel.
#[derive(Clone, PartialEq, Debug)]
pub enum SplitState {
    /// Master collapsed — only detail visible.
    Collapsed,
    /// Both panes visible, resizable via drag handle.
    Half,
    /// Master full-width — detail hidden.
    FullRight,
}

impl SplitState {
    /// Cycle: Collapsed → Half → FullRight → Collapsed.
    pub fn next(&self) -> Self {
        match self {
            Self::Collapsed => Self::Half,
            Self::Half      => Self::FullRight,
            Self::FullRight => Self::Collapsed,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SplitViewProps {
    pub state: SplitState,
    pub on_state_change: EventHandler<SplitState>,
    /// Left panel content.
    pub master: Element,
    /// Right panel content (default slot).
    pub children: Element,
}

/// Horizontal split with a draggable resize handle.
/// Double-click the handle to cycle through split states.
#[component]
pub fn SplitView(props: SplitViewProps) -> Element {
    let mut drag_start: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut master_px: Signal<f64>                 = use_signal(|| 280.0);
    let is_half = props.state == SplitState::Half;

    let master_style = match &props.state {
        SplitState::Half => format!(
            "width: {}px; min-width: 160px; max-width: 600px; flex-shrink: 0; overflow: hidden;",
            *master_px.read()
        ),
        _ => "width: 0; min-width: 0; overflow: hidden; flex-shrink: 0;".into(),
    };

    let on_change = props.on_state_change.clone();

    rsx! {
        div {
            style: "display: flex; height: 100%; width: 100%; overflow: hidden;",
            onmousemove: move |evt| {
                if let Some((start_x, start_w)) = *drag_start.read() {
                    let dx = evt.data().client_coordinates().x - start_x;
                    master_px.set((start_w + dx).max(160.0).min(600.0));
                }
            },
            onmouseup:    move |_| drag_start.set(None),
            onmouseleave: move |_| drag_start.set(None),

            // Master pane
            div { class: "fsd-split__master", style: "{master_style}", {props.master} }

            // Resize handle (only visible in Half mode)
            if is_half {
                div {
                    class: "fsd-split__handle",
                    style: "width: 4px; cursor: col-resize; flex-shrink: 0; \
                            background: var(--fsn-color-border-default, #334155);",
                    title: "Drag to resize · Double-click to collapse",
                    onmousedown: move |evt| {
                        evt.stop_propagation();
                        let x = evt.data().client_coordinates().x;
                        drag_start.set(Some((x, *master_px.read())));
                    },
                    ondoubleclick: move |_| on_change.call(SplitState::Collapsed),
                }
            }

            // Detail pane
            div { class: "fsd-split__detail", style: "flex: 1; overflow: hidden;", {props.children} }
        }
    }
}
