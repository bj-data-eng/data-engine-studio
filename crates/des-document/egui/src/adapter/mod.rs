mod input;
mod paint;
mod text;

pub use input::document_input;
pub use paint::{
    CosmicTextPaintResources, TextPaintStats, paint_frame, paint_frame_with_text_renderer,
    paint_frame_with_text_resources, paint_scroll_chrome, paint_surface,
};
pub use text::{EguiTextMeasurer, configure_text_selection_input, copy_selected_text_on_command};
