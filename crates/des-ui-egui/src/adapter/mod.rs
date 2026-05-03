mod input;
mod paint;
mod text;

pub(crate) use input::document_input;
pub(crate) use paint::{paint_frame, paint_scroll_chrome, paint_surface};
pub(crate) use text::{
    EguiTextMeasurer, configure_text_selection_input, copy_selected_text_on_command,
};
