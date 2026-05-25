use des_document::{DocumentInput, DocumentKey, KeyInput, KeyModifiers, Point, PointerInput};
use eframe::egui;
pub fn document_input(ui: &egui::Ui, origin: egui::Pos2) -> DocumentInput {
    ui.input_mut(|input| {
        let scroll_delta = input.smooth_scroll_delta;
        if scroll_delta != egui::Vec2::ZERO {
            input.smooth_scroll_delta = egui::Vec2::ZERO;
        }
        DocumentInput {
            pointer: input
                .pointer
                .interact_pos()
                .or_else(|| input.pointer.hover_pos())
                .map(|position| PointerInput {
                    position: Point::new(position.x - origin.x, position.y - origin.y),
                    primary_delta: Point::new(input.pointer.delta().x, input.pointer.delta().y),
                    primary_down: input.pointer.primary_down(),
                    primary_pressed: input.pointer.primary_pressed(),
                    primary_clicked: input.pointer.primary_clicked(),
                    primary_click_count: if input
                        .pointer
                        .button_triple_clicked(egui::PointerButton::Primary)
                    {
                        3
                    } else if input
                        .pointer
                        .button_double_clicked(egui::PointerButton::Primary)
                    {
                        2
                    } else if input.pointer.primary_clicked() {
                        1
                    } else {
                        0
                    },
                    secondary_clicked: input.pointer.secondary_clicked(),
                    time_seconds: input.time,
                }),
            scroll_delta: Point::new(scroll_delta.x, scroll_delta.y),
            keys: input.events.iter().filter_map(document_key_input).collect(),
        }
    })
}

fn document_key_input(event: &egui::Event) -> Option<KeyInput> {
    let egui::Event::Key {
        key,
        pressed,
        modifiers,
        ..
    } = event
    else {
        return None;
    };
    let input = if *pressed {
        KeyInput::down(document_key(*key))
    } else {
        KeyInput::up(document_key(*key))
    };
    Some(input.with_modifiers(KeyModifiers {
        alt: modifiers.alt,
        ctrl: modifiers.ctrl,
        shift: modifiers.shift,
        command: modifiers.command,
    }))
}

fn document_key(key: egui::Key) -> DocumentKey {
    match key {
        egui::Key::Enter => DocumentKey::Enter,
        egui::Key::Escape => DocumentKey::Escape,
        egui::Key::Tab => DocumentKey::Tab,
        egui::Key::Space => DocumentKey::Space,
        egui::Key::Backspace => DocumentKey::Backspace,
        egui::Key::Delete => DocumentKey::Delete,
        egui::Key::ArrowUp => DocumentKey::ArrowUp,
        egui::Key::ArrowDown => DocumentKey::ArrowDown,
        egui::Key::ArrowLeft => DocumentKey::ArrowLeft,
        egui::Key::ArrowRight => DocumentKey::ArrowRight,
        egui::Key::Home => DocumentKey::Home,
        egui::Key::End => DocumentKey::End,
        egui::Key::PageUp => DocumentKey::PageUp,
        egui::Key::PageDown => DocumentKey::PageDown,
        _ => DocumentKey::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_input_subtracts_origin_and_preserves_time() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
            time: Some(7.5),
            events: vec![egui::Event::PointerMoved(egui::pos2(42.0, 65.0))],
            ..Default::default()
        };
        let mut input = None;

        let _ = ctx.run_ui(raw, |ui| {
            input = Some(document_input(ui, egui::pos2(10.0, 20.0)));
        });

        let pointer = input.unwrap().pointer.unwrap();
        assert_eq!(pointer.position, Point::new(32.0, 45.0));
        assert_eq!(pointer.time_seconds, 7.5);
        assert!(!pointer.primary_down);
    }

    #[test]
    fn document_input_maps_smooth_scroll_delta() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
            events: vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(5.0, -7.0),
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let mut input = None;

        let _ = ctx.run_ui(raw, |ui| {
            input = Some(document_input(ui, egui::Pos2::ZERO));
        });

        let scroll_delta = input.unwrap().scroll_delta;
        assert!(scroll_delta.x > 0.0);
        assert!(scroll_delta.y < 0.0);
    }

    #[test]
    fn document_input_maps_keyboard_intent() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
            events: vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    command: true,
                    ..Default::default()
                },
            }],
            ..Default::default()
        };
        let mut input = None;

        let _ = ctx.run_ui(raw, |ui| {
            input = Some(document_input(ui, egui::Pos2::ZERO));
        });

        let key = input.unwrap().keys.into_iter().next().unwrap();
        assert_eq!(key.key, DocumentKey::Enter);
        assert!(key.modifiers.command);
        assert!(key.pressed);
    }
}
