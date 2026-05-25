use des_document::{
    AlignItems, Color, CornerRadii, Direction, Document, DocumentBuilder, DocumentCommandAction,
    DocumentCommandRegistry, DocumentEngine, DocumentEvent, DocumentEventKind, DocumentInput,
    DocumentKey, DocumentProjection, DocumentProjectionOperation, DocumentView, DocumentWidget,
    Element, ElementBehaviorEvent, ElementId, ElementSpec, ElementStateSelector, FlexWrap, Insets,
    JustifyContent, KeyInput, KeyModifiers, Length, Overflow, Point, PointerInput, ScrollAxis,
    Shadow, Size, Style, StyleSelector, StyleSheet, TableCellSpec, TableColumnSpec, TableSpec,
    TableTrackSize, TextLayoutRequest, TextLayoutResult, TextLayoutStyle, TextMeasurer,
    TextMeasurerKey, TextOverflow, TextSelectionGranularity, TextTransform, TextWrapMode,
    Transition, ViewportQuery, VisualCloneOptions, WhiteSpace,
};

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.01,
        "expected {actual} to be close to {expected}"
    );
}

fn pointer_input(
    position: Point,
    primary_down: bool,
    primary_pressed: bool,
    primary_clicked: bool,
    time_seconds: f64,
) -> DocumentInput {
    DocumentInput::pointer(
        PointerInput::new(position, time_seconds)
            .with_primary_down(primary_down)
            .with_primary_pressed(primary_pressed)
            .with_primary_clicked(u8::from(primary_clicked)),
    )
}

#[test]
fn document_view_groups_document_stylesheet_and_engine_update() {
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::class("panel"),
        Style::default()
            .width(Length::Px(240.0))
            .height(Length::Px(80.0)),
    );
    let mut view = DocumentView::build(Size::new(640.0, 480.0), stylesheet, |ui| {
        ui.div("panel").class("panel").children(|ui| {
            ui.text("label", "Ready");
        });
    });

    let output = view.update();

    assert_eq!(output.layout.id.as_str(), "root");
    assert_eq!(
        output
            .snapshot()
            .find("panel")
            .expect("panel should exist")
            .rect()
            .size
            .width,
        240.0
    );
    assert_eq!(view.document().viewport(), Size::new(640.0, 480.0));
}

#[test]
fn document_output_exposes_commands_from_typed_behavior_hooks() {
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("run"),
        Style::default()
            .width(Length::Px(96.0))
            .height(Length::Px(32.0)),
    );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("run")
            .behavior_hooks([
                (ElementBehaviorEvent::Click, "run-query"),
                (ElementBehaviorEvent::ContextMenu, "open-query-menu"),
            ])
            .text("Run");
    });

    let output =
        view.update_with_input(pointer_input(Point::new(8.0, 8.0), true, false, true, 0.0));
    let commands = output.commands();
    let run = output.snapshot().find("run").unwrap();
    let hooks = run.behavior_hooks();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].target, ElementId::new("run"));
    assert_eq!(commands[0].event, DocumentEventKind::Clicked);
    assert_eq!(commands[0].command, "run-query");
    assert_eq!(hooks.len(), 2);
    assert!(hooks.iter().any(|hook| hook.command == "open-query-menu"));
}

#[test]
fn document_output_exposes_interaction_query_helpers() {
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("run"),
        Style::default()
            .width(Length::Px(96.0))
            .height(Length::Px(32.0)),
    );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("run").text("Run");
    });

    let output = view.update_with_input(pointer_input(Point::new(8.0, 8.0), true, true, true, 0.0));
    let run_events = output.events_for("run").collect::<Vec<_>>();

    assert_eq!(output.hit_id().map(ElementId::as_str), Some("run"));
    assert!(output.hit_is("run"));
    assert_eq!(output.hit_element().unwrap().text(), Some("Run".to_owned()));
    assert!(output.has_event("run", DocumentEventKind::Pressed));
    assert!(output.has_event("run", DocumentEventKind::Clicked));
    assert!(output.has_event_kind(DocumentEventKind::Clicked));
    assert_eq!(
        output
            .events_of_kind(DocumentEventKind::Clicked)
            .map(|event| event.target.as_str())
            .collect::<Vec<_>>(),
        vec!["run"]
    );
    assert_eq!(
        output
            .event_targets_of_kind(DocumentEventKind::Clicked)
            .map(ElementId::as_str)
            .collect::<Vec<_>>(),
        vec!["run"]
    );
    assert_eq!(
        output
            .first_event_target(DocumentEventKind::Clicked)
            .map(ElementId::as_str),
        Some("run")
    );
    assert_eq!(
        output
            .clicked_targets()
            .map(ElementId::as_str)
            .collect::<Vec<_>>(),
        vec!["run"]
    );
    assert_eq!(
        output.first_clicked_target().map(ElementId::as_str),
        Some("run")
    );
    assert!(output.was_pressed("run"));
    assert!(output.was_clicked("run"));
    assert!(!output.was_released("run"));
    assert!(run_events.contains(&&DocumentEvent::pressed("run")));
    assert!(run_events.contains(&&DocumentEvent::clicked("run")));
}

#[test]
fn document_output_exposes_context_and_keyboard_query_helpers() {
    let stylesheet = StyleSheet::new()
        .id("menu", Style::default().size(96.0, 32.0))
        .id("search", Style::default().size(160.0, 32.0));
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("menu").text("Menu");
        ui.input("search").focused(true).empty();
    });

    let context_output =
        view.update_with_input(DocumentInput::secondary_click(Point::new(8.0, 8.0)));
    let key_down_output = view.update_with_input(DocumentInput::key_down(DocumentKey::Enter));
    let key_up_output = view.update_with_input(DocumentInput::key_up(DocumentKey::Enter));

    assert_eq!(
        context_output
            .context_requested_targets()
            .map(ElementId::as_str)
            .collect::<Vec<_>>(),
        vec!["menu"]
    );
    assert_eq!(
        context_output
            .first_context_requested_target()
            .map(ElementId::as_str),
        Some("menu")
    );
    assert!(context_output.context_requested_for("menu"));
    assert!(!context_output.context_requested_for("search"));
    assert_eq!(
        key_down_output
            .key_down_events()
            .map(|(target, key)| (target.as_str(), key))
            .collect::<Vec<_>>(),
        vec![("search", KeyInput::down(DocumentKey::Enter))]
    );
    assert_eq!(
        key_down_output.key_down_for("search").collect::<Vec<_>>(),
        vec![KeyInput::down(DocumentKey::Enter)]
    );
    assert!(key_down_output.has_key_down("search", KeyInput::down(DocumentKey::Enter)));
    assert_eq!(
        key_up_output
            .key_up_events()
            .map(|(target, key)| (target.as_str(), key))
            .collect::<Vec<_>>(),
        vec![("search", KeyInput::up(DocumentKey::Enter))]
    );
    assert_eq!(
        key_up_output.key_up_for("search").collect::<Vec<_>>(),
        vec![KeyInput::up(DocumentKey::Enter)]
    );
    assert!(key_up_output.has_key_up("search", KeyInput::up(DocumentKey::Enter)));
}

#[test]
fn document_builder_expresses_default_and_event_scoped_commands() {
    let stylesheet = StyleSheet::new()
        .id("run", Style::default().size(96.0, 32.0))
        .id("search", Style::default().size(160.0, 32.0));
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("run").command("run-query").text("Run");
        ui.input("search")
            .focused(true)
            .command_on(ElementBehaviorEvent::KeyDown, "submit-search")
            .empty();
    });

    let click_output = view.update_with_input(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let key_output = view.update_with_input(DocumentInput::key_down(DocumentKey::Enter));
    let run = click_output.snapshot().find("run").unwrap();
    let search = key_output.snapshot().find("search").unwrap();

    assert!(run.interactive());
    assert_eq!(run.behavior_hooks()[0].event, "click");
    assert_eq!(run.behavior_hooks()[0].command, "run-query");
    assert!(click_output.has_command_intent("run", ElementBehaviorEvent::Click, "run-query"));
    assert!(search.interactive());
    assert_eq!(search.behavior_hooks()[0].event, "keydown");
    assert_eq!(search.behavior_hooks()[0].command, "submit-search");
    assert!(key_output.has_command_intent(
        "search",
        ElementBehaviorEvent::KeyDown,
        "submit-search"
    ));
}

#[test]
fn document_command_registry_maps_hook_commands_to_typed_actions() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum AppAction {
        RunQuery,
        CancelQuery,
    }

    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("run"),
        Style::default()
            .width(Length::Px(96.0))
            .height(Length::Px(32.0)),
    );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("run").on_click("run-query").text("Run");
    });
    let registry = DocumentCommandRegistry::new().bind_many([
        ("run-query", AppAction::RunQuery),
        ("cancel-query", AppAction::CancelQuery),
    ]);

    let output =
        view.update_with_input(pointer_input(Point::new(8.0, 8.0), true, false, true, 0.0));
    let actions = registry.command_actions(&output).collect::<Vec<_>>();
    let clicked_actions = registry.clicked_actions(&output).collect::<Vec<_>>();
    let run_actions = registry
        .command_actions_for(&output, "run")
        .collect::<Vec<_>>();
    let clicked_commands = output
        .commands_of_kind(DocumentEventKind::Clicked)
        .collect::<Vec<_>>();

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].target, &ElementId::new("run"));
    assert_eq!(actions[0].event, DocumentEventKind::Clicked);
    assert_eq!(actions[0].command, "run-query");
    assert_eq!(*actions[0].action, AppAction::RunQuery);
    assert_eq!(clicked_actions.len(), 1);
    assert_eq!(clicked_actions[0].command, "run-query");
    assert_eq!(run_actions.len(), 1);
    assert_eq!(*run_actions[0].action, AppAction::RunQuery);
    assert_eq!(clicked_commands.len(), 1);
    assert_eq!(clicked_commands[0].command, "run-query");
    assert_eq!(
        output
            .commands_for("run")
            .map(|command| command.command)
            .collect::<Vec<_>>(),
        vec!["run-query"]
    );
    assert!(output.has_command("run", "run-query"));
    assert!(output.has_command_kind("run", DocumentEventKind::Clicked, "run-query"));
    assert!(!output.has_command("cancel", "cancel-query"));

    let collected = [
        ("run-query", AppAction::RunQuery),
        ("cancel-query", AppAction::CancelQuery),
    ]
    .into_iter()
    .collect::<DocumentCommandRegistry<_>>();

    assert_eq!(collected.bindings(), registry.bindings());
}

#[test]
fn document_command_registry_can_scope_actions_by_authored_event_intent() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum AppAction {
        CommitByClick,
        CommitByKeyboard,
        Fallback,
    }

    let stylesheet = StyleSheet::new().id(
        "commit",
        Style::default()
            .width(Length::Px(120.0))
            .height(Length::Px(36.0)),
    );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("commit")
            .focused(true)
            .on_click("commit")
            .on_key_down("commit")
            .text("Commit");
    });
    let registry = DocumentCommandRegistry::new()
        .bind("commit", AppAction::Fallback)
        .bind_click("commit", AppAction::CommitByClick)
        .bind_on(
            ElementBehaviorEvent::KeyDown,
            "commit",
            AppAction::CommitByKeyboard,
        );

    let click_output =
        view.update_with_input(pointer_input(Point::new(8.0, 8.0), true, false, true, 0.0));
    let key_output = view.update_with_input(DocumentInput::key_down(DocumentKey::Enter));
    let click_actions = registry.command_actions(&click_output).collect::<Vec<_>>();
    let key_actions = registry.command_actions(&key_output).collect::<Vec<_>>();
    let key_intent_actions = registry
        .command_actions_for_intent(&key_output, ElementBehaviorEvent::KeyDown)
        .collect::<Vec<_>>();

    assert_eq!(registry.action_for("commit"), Some(&AppAction::Fallback));
    assert!(click_output.has_command_intent("commit", ElementBehaviorEvent::Click, "commit"));
    assert!(key_output.has_command_intent("commit", ElementBehaviorEvent::KeyDown, "commit"));
    assert_eq!(
        key_output
            .commands_for_intent(ElementBehaviorEvent::KeyDown)
            .map(|command| command.command)
            .collect::<Vec<_>>(),
        vec!["commit"]
    );
    assert_eq!(click_actions.len(), 1);
    assert_eq!(*click_actions[0].action, AppAction::CommitByClick);
    assert_eq!(click_actions[0].event, DocumentEventKind::Clicked);
    assert_eq!(key_actions.len(), 1);
    assert_eq!(*key_actions[0].action, AppAction::CommitByKeyboard);
    assert_eq!(key_intent_actions.len(), 1);
    assert_eq!(*key_intent_actions[0].action, AppAction::CommitByKeyboard);
    assert_eq!(
        key_actions[0].event,
        DocumentEventKind::KeyDown(KeyInput::down(DocumentKey::Enter))
    );
}

#[test]
fn document_command_registry_collects_owned_app_actions_for_update_loops() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    enum AppAction {
        CommitByClick,
        CommitByKeyboard,
        Fallback,
    }

    let stylesheet = StyleSheet::new().id(
        "commit",
        Style::default()
            .width(Length::Px(120.0))
            .height(Length::Px(36.0)),
    );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("commit")
            .focused(true)
            .on_click("commit")
            .on_key_down("commit")
            .text("Commit");
    });
    let registry = DocumentCommandRegistry::new()
        .bind("commit", AppAction::Fallback)
        .bind_click("commit", AppAction::CommitByClick)
        .bind_on(
            ElementBehaviorEvent::KeyDown,
            "commit",
            AppAction::CommitByKeyboard,
        );

    let click_output =
        view.update_with_input(pointer_input(Point::new(8.0, 8.0), true, false, true, 0.0));
    let key_output = view.update_with_input(DocumentInput::key_down(DocumentKey::Enter));

    let click_actions = registry.collect_actions(&click_output);
    let clicked_actions = registry.collect_clicked_actions(&click_output);
    let key_actions =
        registry.collect_actions_for_intent(&key_output, ElementBehaviorEvent::KeyDown);
    let commit_actions = registry.collect_actions_for(&click_output, "commit");

    assert_eq!(
        click_actions,
        vec![DocumentCommandAction {
            target: ElementId::new("commit"),
            event: DocumentEventKind::Clicked,
            command: "commit".to_owned(),
            action: AppAction::CommitByClick,
        }]
    );
    assert_eq!(clicked_actions, click_actions);
    assert_eq!(commit_actions, click_actions);
    assert_eq!(
        key_actions,
        vec![DocumentCommandAction {
            target: ElementId::new("commit"),
            event: DocumentEventKind::KeyDown(KeyInput::down(DocumentKey::Enter)),
            command: "commit".to_owned(),
            action: AppAction::CommitByKeyboard,
        }]
    );
}

#[test]
fn document_view_can_update_and_collect_typed_actions_in_one_front_door_call() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    enum AppAction {
        Run,
    }

    let stylesheet = StyleSheet::new().id(
        "run",
        Style::default()
            .width(Length::Px(96.0))
            .height(Length::Px(32.0)),
    );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("run").on_click("run").text("Run");
    });
    let registry = DocumentCommandRegistry::new().bind_click("run", AppAction::Run);

    let frame = view.update_with_input_actions(
        DocumentInput::primary_click(Point::new(8.0, 8.0)),
        &registry,
    );
    let run = frame.output.snapshot().find("run").unwrap();

    assert_eq!(run.text(), Some("Run".to_owned()));
    assert_eq!(
        frame.actions,
        vec![DocumentCommandAction {
            target: ElementId::new("run"),
            event: DocumentEventKind::Clicked,
            command: "run".to_owned(),
            action: AppAction::Run,
        }]
    );
}

#[test]
fn document_action_frame_supports_app_update_loop_queries() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    enum AppAction {
        Run,
        Cancel,
    }

    let stylesheet = StyleSheet::new()
        .id("run", Style::default().size(96.0, 32.0))
        .id("cancel", Style::default().size(96.0, 32.0));
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("run").on_click("run").text("Run");
        ui.button("cancel")
            .focused(true)
            .on_key_down("cancel")
            .text("Cancel");
    });
    let registry = DocumentCommandRegistry::new()
        .bind_click("run", AppAction::Run)
        .bind_on(ElementBehaviorEvent::KeyDown, "cancel", AppAction::Cancel);

    let click_frame = view.update_with_input_actions(
        DocumentInput::primary_click(Point::new(8.0, 8.0)),
        &registry,
    );
    assert_eq!(
        click_frame.output().hit_id().map(ElementId::as_str),
        Some("run")
    );
    assert_eq!(click_frame.len(), 1);
    assert!(!click_frame.is_empty());
    assert!(click_frame.contains_action(&AppAction::Run));
    assert_eq!(
        click_frame.first_action().map(|action| &action.action),
        Some(&AppAction::Run)
    );
    assert_eq!(click_frame.actions_for("run").count(), 1);
    assert_eq!(click_frame.clicked_actions().count(), 1);
    assert_eq!(
        click_frame
            .actions_of_kind(DocumentEventKind::Clicked)
            .map(|action| action.command.as_str())
            .collect::<Vec<_>>(),
        vec!["run"]
    );

    let key_frame =
        view.update_with_input_actions(DocumentInput::key_down(DocumentKey::Escape), &registry);
    assert_eq!(key_frame.actions().len(), 1);
    assert!(key_frame.contains_action(&AppAction::Cancel));
    assert_eq!(key_frame.clicked_actions().count(), 0);
    let (output, actions) = key_frame.into_parts();
    assert_eq!(
        output
            .first_event_target(DocumentEventKind::KeyDown(KeyInput::down(
                DocumentKey::Escape
            )))
            .map(ElementId::as_str),
        Some("cancel")
    );
    assert_eq!(actions[0].action, AppAction::Cancel);
}

#[test]
fn document_command_registry_dispatches_typed_actions_with_context() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum AppAction {
        RunQuery,
        CancelQuery,
    }

    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("run"),
            Style::default()
                .width(Length::Px(96.0))
                .height(Length::Px(32.0)),
        )
        .rule(
            StyleSelector::id("cancel"),
            Style::default()
                .width(Length::Px(96.0))
                .height(Length::Px(32.0)),
        );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.button("run").on_click("run-query").text("Run");
        ui.button("cancel")
            .focused(true)
            .on_key_down("cancel-query")
            .text("Cancel");
    });
    let mut unhandled_view =
        DocumentView::build(Size::new(320.0, 180.0), StyleSheet::new(), |ui| {
            ui.button("later")
                .focused(true)
                .on_key_down("unknown-command")
                .text("Later");
        });
    let registry = DocumentCommandRegistry::new()
        .bind("run-query", AppAction::RunQuery)
        .bind("cancel-query", AppAction::CancelQuery);

    let click_output =
        view.update_with_input(pointer_input(Point::new(8.0, 8.0), true, false, true, 0.0));
    let key_output = view.update_with_input(DocumentInput::key_down(DocumentKey::Escape));
    let unknown_output =
        unhandled_view.update_with_input(DocumentInput::key_down(DocumentKey::Escape));
    let mut handled = Vec::new();
    let click_report = registry.dispatch(&click_output, |command| {
        handled.push((
            command.target.clone(),
            command.event,
            command.command.to_owned(),
            *command.action,
        ));
    });
    let key_report = registry.dispatch(&key_output, |command| {
        handled.push((
            command.target.clone(),
            command.event,
            command.command.to_owned(),
            *command.action,
        ));
    });
    let unknown_report = registry.dispatch(&unknown_output, |_| {});
    let mut clicked_only = Vec::new();
    let clicked_only_report = registry.dispatch_clicked(&click_output, |command| {
        clicked_only.push((
            command.target.clone(),
            command.event,
            command.command.to_owned(),
            *command.action,
        ));
    });
    let key_only_report = registry.dispatch_kind(
        &click_output,
        DocumentEventKind::KeyDown(KeyInput::down(DocumentKey::Escape)),
        |_| {},
    );
    let mut key_intent = Vec::new();
    let key_intent_report =
        registry.dispatch_intent(&key_output, ElementBehaviorEvent::KeyDown, |command| {
            key_intent.push((
                command.target.clone(),
                command.event,
                command.command.to_owned(),
                *command.action,
            ));
        });

    assert_eq!(click_report.commands, 1);
    assert_eq!(click_report.handled, 1);
    assert_eq!(click_report.unhandled, 0);
    assert_eq!(key_report.commands, 1);
    assert_eq!(key_report.handled, 1);
    assert_eq!(key_report.unhandled, 0);
    assert_eq!(unknown_report.commands, 1);
    assert_eq!(unknown_report.handled, 0);
    assert_eq!(unknown_report.unhandled, 1);
    assert_eq!(clicked_only_report.commands, 1);
    assert_eq!(clicked_only_report.handled, 1);
    assert_eq!(clicked_only_report.unhandled, 0);
    assert_eq!(key_only_report.commands, 0);
    assert_eq!(key_intent_report.commands, 1);
    assert_eq!(key_intent_report.handled, 1);
    assert_eq!(key_intent_report.unhandled, 0);
    assert_eq!(
        handled,
        vec![
            (
                ElementId::new("run"),
                DocumentEventKind::Clicked,
                "run-query".to_owned(),
                AppAction::RunQuery,
            ),
            (
                ElementId::new("cancel"),
                DocumentEventKind::KeyDown(KeyInput::down(DocumentKey::Escape)),
                "cancel-query".to_owned(),
                AppAction::CancelQuery,
            )
        ]
    );
    assert_eq!(
        clicked_only,
        vec![(
            ElementId::new("run"),
            DocumentEventKind::Clicked,
            "run-query".to_owned(),
            AppAction::RunQuery,
        )]
    );
    assert_eq!(
        key_intent,
        vec![(
            ElementId::new("cancel"),
            DocumentEventKind::KeyDown(KeyInput::down(DocumentKey::Escape)),
            "cancel-query".to_owned(),
            AppAction::CancelQuery,
        )]
    );
}

#[test]
fn keyboard_input_targets_focused_element_and_emits_hook_command() {
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("search"),
        Style::default()
            .width(Length::Px(180.0))
            .height(Length::Px(32.0)),
    );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.input("search")
            .focused(true)
            .on_key_down("submit-search")
            .empty();
    });

    let output = view.update_with_input(DocumentInput::key_down(DocumentKey::Enter));
    let commands = output.commands();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].target, ElementId::new("search"));
    assert_eq!(
        commands[0].event,
        DocumentEventKind::KeyDown(KeyInput {
            key: DocumentKey::Enter,
            modifiers: KeyModifiers::default(),
            pressed: true,
        })
    );
    assert_eq!(commands[0].command, "submit-search");
}

#[test]
fn document_input_builders_express_host_intent_without_struct_literals() {
    let pointer = PointerInput::new(Point::new(16.0, 24.0), 1.5)
        .with_primary_delta(Point::new(2.0, 3.0))
        .primary_down()
        .primary_pressed()
        .with_position(Point::new(18.0, 27.0))
        .with_time(1.75);
    let input = DocumentInput::pointer(pointer)
        .with_scroll(Point::new(0.0, -12.0))
        .with_key(KeyInput::down(DocumentKey::Enter).command().shift())
        .with_key(KeyInput::up(DocumentKey::Escape));
    let modified_down =
        KeyInput::down_with_modifiers(DocumentKey::Character('s'), KeyModifiers::new().command());
    let modified_up =
        KeyInput::up_with_modifiers(DocumentKey::Character('s'), KeyModifiers::new().command());
    let shortcut_input = DocumentInput::key_down_with_modifiers(
        DocumentKey::Character('s'),
        KeyModifiers::new().command(),
    )
    .with_key(KeyInput::up(DocumentKey::Character('s')).command());
    let shortcut_release = DocumentInput::key_up_with_modifiers(
        DocumentKey::Character('s'),
        KeyModifiers::new().command(),
    );

    assert_eq!(input.pointer, Some(pointer));
    assert_eq!(pointer.position, Point::new(18.0, 27.0));
    assert_eq!(pointer.primary_delta, Point::new(2.0, 3.0));
    assert_eq!(pointer.time_seconds, 1.75);
    assert!(pointer.primary_down);
    assert!(pointer.primary_pressed);
    assert_eq!(input.scroll_delta, Point::new(0.0, -12.0));
    assert_eq!(input.keys.len(), 2);
    assert_eq!(input.keys[0].key, DocumentKey::Enter);
    assert!(input.keys[0].pressed);
    assert!(input.keys[0].modifiers.command);
    assert!(input.keys[0].modifiers.shift);
    assert_eq!(input.keys[1], KeyInput::up(DocumentKey::Escape));
    assert_eq!(
        modified_down,
        KeyInput::down(DocumentKey::Character('s')).command()
    );
    assert_eq!(
        modified_up,
        KeyInput::up(DocumentKey::Character('s')).command()
    );
    assert_eq!(shortcut_input.keys, vec![modified_down, modified_up]);
    assert_eq!(shortcut_release.keys, vec![modified_up]);

    let click = PointerInput::at(Point::new(4.0, 5.0)).primary_clicked();
    let press = PointerInput::at(Point::new(4.0, 5.0)).primary_press();
    let double_click = PointerInput::at(Point::new(4.0, 5.0)).primary_double_clicked();
    let triple_click = PointerInput::at(Point::new(4.0, 5.0)).primary_triple_clicked();
    let context_click = PointerInput::at(Point::new(4.0, 5.0)).secondary_clicked();

    assert_eq!(click.primary_click_count, 1);
    assert!(press.primary_down);
    assert!(press.primary_pressed);
    assert_eq!(double_click.primary_click_count, 2);
    assert_eq!(triple_click.primary_click_count, 3);
    assert!(click.primary_clicked);
    assert!(context_click.secondary_clicked);

    assert_eq!(
        DocumentInput::pointer_at(Point::new(1.0, 2.0))
            .pointer
            .unwrap()
            .position,
        Point::new(1.0, 2.0)
    );
    assert_eq!(
        DocumentInput::pointer_at_time(Point::new(1.0, 2.0), 3.5)
            .pointer
            .unwrap()
            .time_seconds,
        3.5
    );
    assert_eq!(
        DocumentInput::primary_click(Point::new(1.0, 2.0))
            .pointer
            .unwrap()
            .primary_click_count,
        1
    );
    assert!(
        DocumentInput::primary_press(Point::new(1.0, 2.0))
            .pointer
            .unwrap()
            .primary_pressed
    );
    assert!(
        DocumentInput::primary_down(Point::new(1.0, 2.0))
            .pointer
            .unwrap()
            .primary_down
    );
    assert_eq!(
        DocumentInput::primary_drag(Point::new(1.0, 2.0), Point::new(3.0, 4.0))
            .pointer
            .unwrap()
            .primary_delta,
        Point::new(3.0, 4.0)
    );
    assert_eq!(
        DocumentInput::primary_double_click(Point::new(1.0, 2.0))
            .pointer
            .unwrap()
            .primary_click_count,
        2
    );
    assert_eq!(
        DocumentInput::primary_triple_click(Point::new(1.0, 2.0))
            .pointer
            .unwrap()
            .primary_click_count,
        3
    );
    assert!(
        DocumentInput::secondary_click(Point::new(1.0, 2.0))
            .pointer
            .unwrap()
            .secondary_clicked
    );
}

#[test]
fn document_projection_batches_app_state_updates() {
    let stylesheet = StyleSheet::new()
        .class(
            "ready",
            Style::default().background(Color::rgb(205, 239, 221)),
        )
        .class(
            "pending",
            Style::default().background(Color::rgb(255, 238, 190)),
        )
        .class("active", Style::default().border(Color::rgb(80, 130, 180)));
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.div("status")
            .classes(["pending", "stale"])
            .selected(false)
            .attribute("data-old-status", "pending")
            .children(|ui| {
                ui.text("status-label", "Pending");
            });
    });
    let projection = DocumentProjection::new()
        .set_text("status-label", "Ready")
        .with_element("status", |mut status| {
            status
                .value("ready")
                .attributes([("data-status", "ready"), ("aria-label", "Ready status")])
                .select()
                .focus()
                .remove_attributes(["data-old-status"])
                .remove_classes(["pending", "stale"])
                .add_classes(["ready", "active"]);
        })
        .add_classes("status", ["hydrated"]);

    let report = view.project(&projection).unwrap();
    let output = view.update();
    let status = output.snapshot().find("status").unwrap();

    assert_eq!(report.operations, 12);
    assert_eq!(report.changed, 12);
    assert_eq!(
        output.snapshot().find("status-label").unwrap().text(),
        Some("Ready".to_owned())
    );
    assert_eq!(status.value(), Some("ready"));
    assert_eq!(status.attribute("data-status"), Some("ready"));
    assert_eq!(status.attribute("aria-label"), Some("Ready status"));
    assert_eq!(status.attribute("data-old-status"), None);
    assert!(status.selected());
    assert!(status.focused());
    assert!(status.has_class("ready"));
    assert!(status.has_class("active"));
    assert!(status.has_class("hydrated"));
    assert!(!status.has_class("pending"));
    assert!(!status.has_class("stale"));
    assert_eq!(status.style().background, Some(Color::rgb(205, 239, 221)));
    assert_eq!(status.style().border, Some(Color::rgb(80, 130, 180)));

    let unchanged_report = view.project(&projection).unwrap();

    assert_eq!(unchanged_report.operations, 12);
    assert_eq!(unchanged_report.changed, 0);

    let reset = DocumentProjection::new()
        .deselect("status")
        .blur("status")
        .disable("status")
        .remove_attributes("status", ["data-status", "aria-label"])
        .remove_classes("status", ["ready", "active", "hydrated"]);
    let reset_report = view.project(&reset).unwrap();
    let reset_output = view.update();
    let reset_status = reset_output.snapshot().find("status").unwrap();

    assert_eq!(reset_report.operations, 8);
    assert_eq!(reset_report.changed, 8);
    assert!(!reset_status.selected());
    assert!(!reset_status.focused());
    assert!(reset_status.disabled());
    assert_eq!(reset_status.attribute("data-status"), None);
    assert_eq!(reset_status.attribute("aria-label"), None);
    assert!(!reset_status.has_class("ready"));
    assert!(!reset_status.has_class("active"));
    assert!(!reset_status.has_class("hydrated"));

    let restore = DocumentProjection::new()
        .select("status")
        .focus("status")
        .enable("status");
    let restore_report = view.project(&restore).unwrap();
    let restore_output = view.update();
    let restore_status = restore_output.snapshot().find("status").unwrap();

    assert_eq!(restore_report.operations, 3);
    assert_eq!(restore_report.changed, 3);
    assert!(restore_status.selected());
    assert!(restore_status.focused());
    assert!(!restore_status.disabled());
}

#[test]
fn document_projection_updates_semantic_attributes() {
    let mut view = DocumentView::build(Size::new(320.0, 180.0), StyleSheet::new(), |ui| {
        ui.button("run")
            .aria("label", "Run query")
            .data("state", "idle")
            .data("ephemeral", "yes")
            .text("Run");
    });

    let report = view
        .project_with(|projection| {
            projection
                .element("run")
                .aria("label", "Query running")
                .aria("busy", "true")
                .data("state", "running")
                .remove_data("ephemeral");
        })
        .unwrap();
    let output = view.update();
    let run = output.snapshot().find("run").unwrap();

    assert_eq!(report.operations, 4);
    assert_eq!(report.changed, 4);
    assert_eq!(run.attribute("aria-label"), Some("Query running"));
    assert_eq!(run.attribute("aria-busy"), Some("true"));
    assert_eq!(run.attribute("data-state"), Some("running"));
    assert_eq!(run.attribute("data-ephemeral"), None);

    let unchanged = DocumentProjection::new()
        .set_aria("run", "label", "Query running")
        .set_aria("run", "busy", "true")
        .set_data("run", "state", "running")
        .remove_data("run", "ephemeral");
    let unchanged_report = view.project(&unchanged).unwrap();

    assert_eq!(unchanged_report.operations, 4);
    assert_eq!(unchanged_report.changed, 0);
}

#[test]
fn document_projection_expresses_conditional_app_state_without_branching() {
    let stylesheet = StyleSheet::new()
        .class(
            "is-ready",
            Style::default().background(Color::rgb(205, 239, 221)),
        )
        .class(
            "is-loading",
            Style::default().background(Color::rgb(255, 238, 190)),
        )
        .class(
            "is-stale",
            Style::default().border(Color::rgb(180, 120, 80)),
        );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.div("status")
            .classes(["is-loading", "is-stale"])
            .data("busy", "true")
            .aria("busy", "true")
            .children(|ui| {
                ui.text("status-label", "Loading");
            });
    });

    let report = view
        .project_with(|projection| {
            projection
                .element("status")
                .class_if("is-ready", true)
                .classes_if(["is-loading", "is-stale"], false)
                .data_if("busy", "true", false)
                .aria_if("busy", "false", true);
        })
        .unwrap();
    let output = view.update();
    let status = output.snapshot().find("status").unwrap();

    assert_eq!(report.operations, 5);
    assert_eq!(report.changed, 5);
    assert!(status.has_class("is-ready"));
    assert!(!status.has_class("is-loading"));
    assert!(!status.has_class("is-stale"));
    assert_eq!(status.data("busy"), None);
    assert_eq!(status.aria("busy"), Some("false"));
    assert_eq!(status.style().background, Some(Color::rgb(205, 239, 221)));

    let reset = DocumentProjection::new()
        .class_if("status", "is-ready", false)
        .classes_if("status", ["is-loading", "is-stale"], true)
        .set_data_if("status", "busy", "true", true)
        .set_aria_if("status", "busy", "false", false);
    let reset_report = view.project(&reset).unwrap();
    let reset_output = view.update();
    let reset_status = reset_output.snapshot().find("status").unwrap();

    assert_eq!(reset_report.operations, 5);
    assert_eq!(reset_report.changed, 5);
    assert!(!reset_status.has_class("is-ready"));
    assert!(reset_status.has_class("is-loading"));
    assert!(reset_status.has_class("is-stale"));
    assert_eq!(reset_status.data("busy"), Some("true"));
    assert_eq!(reset_status.aria("busy"), None);
}

#[test]
fn document_projection_composes_subprojections_for_app_state() {
    let mut view = DocumentView::build(Size::new(320.0, 180.0), StyleSheet::new(), |ui| {
        ui.div("summary").data("state", "empty").children(|ui| {
            ui.text("summary-count", "0 rows");
        });
        ui.button("refresh").text("Refresh");
    });

    let summary_projection = DocumentProjection::new()
        .set_text("summary-count", "42 rows")
        .set_data("summary", "state", "ready");
    let controls_projection = [
        DocumentProjectionOperation::SetDisabled {
            id: ElementId::new("refresh"),
            disabled: true,
        },
        DocumentProjectionOperation::SetClass {
            id: ElementId::new("refresh"),
            class: "is-loading".into(),
            present: true,
        },
    ]
    .into_iter()
    .collect::<DocumentProjection>();
    let mut projection = DocumentProjection::new()
        .with_projection(summary_projection)
        .with_projection(controls_projection);

    assert_eq!(projection.len(), 4);
    assert!(!projection.is_empty());
    assert_eq!(
        projection
            .operations()
            .iter()
            .map(DocumentProjectionOperation::target)
            .map(ElementId::as_str)
            .collect::<Vec<_>>(),
        vec!["summary-count", "summary", "refresh", "refresh"]
    );

    let report = view.project(&projection).unwrap();
    let output = view.update();
    let summary = output.snapshot().find("summary").unwrap();
    let refresh = output.snapshot().find("refresh").unwrap();

    assert_eq!(report.operations, 4);
    assert_eq!(report.changed, 4);
    assert_eq!(
        output.snapshot().find("summary-count").unwrap().text(),
        Some("42 rows".to_owned())
    );
    assert_eq!(summary.data("state"), Some("ready"));
    assert!(refresh.disabled());
    assert!(refresh.has_class("is-loading"));

    projection.clear();
    assert_eq!(projection.len(), 0);
    assert!(projection.is_empty());
}

#[test]
fn document_view_projects_state_and_updates_in_one_fluent_call() {
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("empty"),
            Style::default().background(Color::rgb(245, 245, 245)),
        )
        .rule(
            StyleSelector::class("ready"),
            Style::default().background(Color::rgb(208, 236, 255)),
        );
    let mut view = DocumentView::build(Size::new(320.0, 180.0), stylesheet, |ui| {
        ui.div("summary").class("empty").children(|ui| {
            ui.text("summary-count", "0 rows");
        });
    });

    let (report, output) = view
        .project_with_and_update(|projection| {
            projection.push_text("summary-count", "42 rows");
            projection
                .element("summary")
                .class("empty", false)
                .class("ready", true)
                .disable()
                .enable()
                .deselect()
                .blur();
        })
        .unwrap();
    let summary = output.snapshot().find("summary").unwrap();

    assert_eq!(report.operations, 7);
    assert_eq!(report.changed, 5);
    assert_eq!(
        output.snapshot().find("summary-count").unwrap().text(),
        Some("42 rows".to_owned())
    );
    assert!(summary.has_class("ready"));
    assert!(!summary.has_class("empty"));
    assert!(!summary.selected());
    assert!(!summary.focused());
    assert!(!summary.disabled());
    assert_eq!(summary.style().background, Some(Color::rgb(208, 236, 255)));

    let (report, output) = view
        .project_with_and_update_with_input(
            pointer_input(Point::new(8.0, 8.0), true, false, true, 0.0),
            |projection| {
                projection
                    .element("summary")
                    .data("refresh", "clicked")
                    .select();
            },
        )
        .unwrap();
    let summary = output.snapshot().find("summary").unwrap();

    assert_eq!(report.operations, 2);
    assert_eq!(report.changed, 2);
    assert_eq!(summary.data("refresh"), Some("clicked"));
    assert!(summary.selected());
}

#[test]
fn document_view_compose_collects_css_and_widget_styles() {
    struct BadgeWidget;

    impl DocumentWidget for BadgeWidget {
        fn render(&self, ui: &mut DocumentBuilder) {
            ui.div("badge").class("badge").text("Pending");
        }

        fn push_styles(&self, stylesheet: &mut StyleSheet) {
            stylesheet.push_class(
                "badge",
                Style::default()
                    .height(Length::Px(24.0))
                    .background(Color::rgb(220, 238, 255)),
            );
        }

        fn push_projection(&self, projection: &mut DocumentProjection) {
            projection
                .element("badge")
                .text("Ready")
                .data("state", "ready");
        }
    }

    let widget = BadgeWidget;
    let mut view = DocumentView::compose(Size::new(320.0, 180.0))
        .css(".panel { width: 160px; height: 64px; }")
        .unwrap()
        .build_with_widget(&widget, |ui| {
            ui.div("panel").class("panel").children(|ui| {
                ui.widget(&widget);
            });
        });

    let output = view.update();
    let panel = output.snapshot().find("panel").unwrap();
    let badge = output.snapshot().find("badge").unwrap();

    assert_eq!(panel.rect().size.width, 160.0);
    assert_eq!(badge.text(), Some("Ready".to_owned()));
    assert_eq!(badge.data("state"), Some("ready"));
    assert_eq!(badge.rect().size.height, 24.0);
    assert_eq!(badge.style().background, Some(Color::rgb(220, 238, 255)));
}

#[test]
fn document_view_can_mount_a_widget_with_its_styles() {
    struct BadgeWidget;
    struct MeterWidget;

    impl DocumentWidget for BadgeWidget {
        fn render(&self, ui: &mut DocumentBuilder) {
            ui.div("badge").class("badge").text("Ready");
        }

        fn push_styles(&self, stylesheet: &mut StyleSheet) {
            stylesheet.push_rule(
                StyleSelector::class("badge"),
                Style::default()
                    .width(Length::Px(88.0))
                    .height(Length::Px(24.0))
                    .background(Color::rgb(220, 238, 255)),
            );
        }
    }

    impl DocumentWidget for MeterWidget {
        fn render(&self, ui: &mut DocumentBuilder) {
            ui.div("meter").class("meter").empty();
        }

        fn push_styles(&self, stylesheet: &mut StyleSheet) {
            stylesheet.push_rule(
                StyleSelector::class("meter"),
                Style::default()
                    .width(Length::Px(120.0))
                    .height(Length::Px(8.0))
                    .background(Color::rgb(205, 239, 221)),
            );
        }
    }

    let widget = BadgeWidget;
    let mut direct =
        DocumentView::build_widget(Size::new(320.0, 180.0), StyleSheet::new(), &widget);
    let mut composed = DocumentView::compose(Size::new(320.0, 180.0)).widget(&widget);
    let badge: Box<dyn DocumentWidget> = Box::new(BadgeWidget);
    let meter: Box<dyn DocumentWidget> = Box::new(MeterWidget);
    let widget_refs = [&*badge, &*meter];
    let mut boxed = DocumentView::compose(Size::new(320.0, 180.0)).widgets(widget_refs);
    let mut pushed = DocumentView::build(Size::new(320.0, 180.0), StyleSheet::new(), |ui| {
        ui.widget(&*badge);
        ui.widget(&*meter);
    });
    pushed.push_widget_styles_many(widget_refs);

    for output in [direct.update(), composed.update()] {
        let badge = output.snapshot().find("badge").unwrap();
        assert_eq!(badge.text(), Some("Ready".to_owned()));
        assert_eq!(badge.rect().size, Size::new(88.0, 24.0));
        assert_eq!(badge.style().background, Some(Color::rgb(220, 238, 255)));
    }

    for output in [boxed.update(), pushed.update()] {
        let badge = output.snapshot().find("badge").unwrap();
        let meter = output.snapshot().find("meter").unwrap();

        assert_eq!(badge.rect().size, Size::new(88.0, 24.0));
        assert_eq!(meter.rect().size, Size::new(120.0, 8.0));
        assert_eq!(meter.style().background, Some(Color::rgb(205, 239, 221)));
    }
}

#[test]
fn document_view_widget_composition_can_return_projection_errors() {
    struct BrokenWidget;

    impl DocumentWidget for BrokenWidget {
        fn render(&self, ui: &mut DocumentBuilder) {
            ui.div("rendered").text("Rendered");
        }

        fn push_projection(&self, projection: &mut DocumentProjection) {
            projection.element("missing").text("Projected");
        }
    }

    fn assert_missing_projection_target(result: des_document::DocumentResult<DocumentView>) {
        let Err(error) = result else {
            panic!("widget projection should fail when it targets an unrendered element");
        };
        assert!(
            error.to_string().contains("missing"),
            "expected missing-element projection error, got {error}"
        );
    }

    let widget = BrokenWidget;

    assert_missing_projection_target(DocumentView::try_build_widget(
        Size::new(320.0, 180.0),
        StyleSheet::new(),
        &widget,
    ));
    assert_missing_projection_target(
        DocumentView::compose(Size::new(320.0, 180.0)).try_widget(&widget),
    );
    assert_missing_projection_target(
        DocumentView::compose(Size::new(320.0, 180.0)).try_build_with_widget(&widget, |ui| {
            ui.widget(&widget);
        }),
    );
}

#[test]
fn document_widgets_can_project_retained_state_through_the_view_front_door() {
    struct StatusWidget {
        ready: bool,
        label: &'static str,
    }

    impl DocumentWidget for StatusWidget {
        fn render(&self, ui: &mut DocumentBuilder) {
            ui.button("status")
                .class("status")
                .on_click("status.toggle")
                .text("Pending");
        }

        fn push_styles(&self, stylesheet: &mut StyleSheet) {
            stylesheet.push_class("status", Style::default().size(120.0, 32.0));
            stylesheet.push_class(
                "is-ready",
                Style::default().background(Color::rgb(205, 239, 221)),
            );
        }

        fn push_projection(&self, projection: &mut DocumentProjection) {
            projection
                .element("status")
                .text(self.label)
                .data("state", if self.ready { "ready" } else { "waiting" })
                .class("is-ready", self.ready)
                .disabled(!self.ready);
        }
    }

    let waiting = StatusWidget {
        ready: false,
        label: "Waiting",
    };
    let ready = StatusWidget {
        ready: true,
        label: "Ready",
    };
    let mut view = DocumentView::compose(Size::new(320.0, 180.0)).widget(&waiting);

    let output = view.update();
    let status = output.snapshot().find("status").unwrap();
    assert_eq!(status.text(), Some("Waiting".to_owned()));
    assert_eq!(status.data("state"), Some("waiting"));
    assert!(status.disabled());
    assert!(!status.has_class("is-ready"));

    let (report, output) = view.project_widget_and_update(&ready).unwrap();
    let status = output.snapshot().find("status").unwrap();

    assert_eq!(report.operations, 4);
    assert_eq!(report.changed, 4);
    assert_eq!(status.text(), Some("Ready".to_owned()));
    assert_eq!(status.data("state"), Some("ready"));
    assert!(!status.disabled());
    assert!(status.has_class("is-ready"));
    assert_eq!(status.style().background, Some(Color::rgb(205, 239, 221)));

    let (_, output) = view
        .project_widget_and_update_with_input(
            &ready,
            pointer_input(Point::new(8.0, 8.0), true, false, true, 0.0),
        )
        .unwrap();
    assert!(output.has_command("status", "status.toggle"));
}

#[test]
fn document_builder_and_engine_update_are_front_door_api() {
    let mut document = Document::build(Size::new(320.0, 200.0), |document| {
        document.element("panel", ElementSpec::div().class("panel"), |document| {
            document.text_element("label", ElementSpec::text(), "Hello");
        });
    });
    let stylesheet = StyleSheet::new().class("panel", Style::default().size(120.0, 48.0));
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(output.layout.find("panel").unwrap().rect.size.width, 120.0);
    assert_eq!(
        output
            .layout
            .find("label")
            .unwrap()
            .text
            .as_ref()
            .map(|text| text.semantic_text()),
        Some("Hello")
    );
}

#[test]
fn document_prelude_exposes_common_app_authoring_surface() {
    use des_document::prelude::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum AppAction {
        Run,
    }

    struct RunBadge;

    impl DocumentWidget for RunBadge {
        fn render(&self, ui: &mut DocumentBuilder) {
            ui.button("run")
                .classes(["badge", "primary"])
                .aria("label", "Run")
                .command("run")
                .text("Run");
        }

        fn push_styles(&self, stylesheet: &mut StyleSheet) {
            stylesheet.push_class("badge", Style::default().size(72.0, 28.0));
        }
    }

    let widget = RunBadge;
    let mut view = DocumentView::compose(Size::new(240.0, 120.0))
        .with_css(".primary { background: rgb(220, 238, 255); }")
        .expect("CSS should compose from the prelude")
        .widget(&widget);
    let registry = DocumentCommandRegistry::new().bind("run", AppAction::Run);
    let frame = view.update_with_input_actions(
        DocumentInput::primary_click(Point::new(8.0, 8.0)),
        &registry,
    );
    let run = frame.output.snapshot().find("run").unwrap();
    let inline = InlineTextStyle {
        font_weight: Some(FontWeight::BOLD),
        font_stretch: Some(FontStretch::CONDENSED),
        font_style: Some(FontStyle::Italic),
        ..InlineTextStyle::default()
    };
    let text_style = TextLayoutStyle {
        white_space_collapse: WhiteSpaceCollapse::PreserveBreaks,
        overflow_wrap: OverflowWrap::Anywhere,
        word_break: WordBreak::BreakAll,
        ..TextLayoutStyle::default()
    };
    let _normalized: Option<NormalizedText> = None;
    let _layout_line: Option<TextLayoutLine> = None;
    let _layout_run: Option<TextLayoutRun> = None;
    let _measurer_key: Option<TextMeasurerKey> = None;

    assert_eq!(frame.actions.len(), 1);
    assert_eq!(frame.actions[0].action, AppAction::Run);
    assert!(run.has_all_classes(["badge", "primary"]));
    assert_eq!(run.aria("label"), Some("Run"));
    assert_eq!(run.rect().size, Size::new(72.0, 28.0));
    assert_eq!(run.style().background, Some(Color::rgb(220, 238, 255)));
    assert_eq!(inline.font_weight, Some(FontWeight::BOLD));
    assert_eq!(inline.font_stretch, Some(FontStretch::CONDENSED));
    assert_eq!(inline.font_style, Some(FontStyle::Italic));
    assert_eq!(
        text_style.white_space_collapse,
        WhiteSpaceCollapse::PreserveBreaks
    );
    assert_eq!(text_style.overflow_wrap, OverflowWrap::Anywhere);
    assert_eq!(text_style.word_break, WordBreak::BreakAll);
}

#[test]
fn stylesheet_composes_typed_rules_and_css_fluently() {
    let stylesheet = StyleSheet::new()
        .element(Element::Div, Style::default().radius(4.0))
        .class("panel", Style::default().height(Length::Px(48.0)))
        .class(
            "accent",
            Style::default().background(Color::rgb(220, 238, 255)),
        )
        .id(
            "title",
            Style::default().padding(Insets::symmetric(4.0, 0.0)),
        )
        .with_css(".panel { width: 120px; }")
        .expect("strict CSS should compose")
        .extended(
            StyleSheet::from_css_forgiving(
                ".panel { unknown-property: 1px; } .title { width: 80px; height: 20px; }",
            )
            .expect("forgiving CSS should keep valid rules"),
        );
    let mut view = DocumentView::build(Size::new(320.0, 200.0), stylesheet, |ui| {
        ui.div("panel").classes(["panel", "accent"]).children(|ui| {
            ui.text_element(
                "title",
                ElementSpec::new(Element::Text).class("title"),
                "Ready",
            );
        });
    });

    let output = view.update();
    let panel = output.snapshot().find("panel").unwrap();
    let title = output.snapshot().find("title").unwrap();

    assert_eq!(panel.rect().size, Size::new(120.0, 48.0));
    assert_eq!(panel.style().background, Some(Color::rgb(220, 238, 255)));
    assert_eq!(panel.style().radius, CornerRadii::all(4.0));
    assert_eq!(title.rect().size, Size::new(80.0, 20.0));
    assert_eq!(title.style().padding.left, 4.0);
    assert!(view.stylesheet().rule_count() >= 6);
}

#[test]
fn document_builder_supports_fluent_html_like_elements() {
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.main("app")
            .classes(["workspace", "workspace-shell"])
            .role("application")
            .data("view", "document")
            .aria("label", "Workspace")
            .children(|ui| {
                ui.header("topbar")
                    .classes(["topbar", "primary-region"])
                    .attributes([("data-region", "chrome"), ("aria-label", "Top bar")])
                    .children(|ui| {
                        ui.h1("title").text("Data Engine Studio");
                    });
                ui.div("content").class("content").children(|ui| {
                    ui.button("run").class("primary").text("Run");
                });
            });
    });
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::element(Element::Main),
            Style::default().size(320.0, 200.0),
        )
        .rule(
            StyleSelector::class("primary"),
            Style::default().size(72.0, 28.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let app = output.layout.find("app").unwrap();
    let run = output.layout.find("run").unwrap();

    assert_eq!(app.element, Element::Main);
    assert!(
        app.classes
            .iter()
            .any(|class| class.as_str() == "workspace")
    );
    assert!(
        app.classes
            .iter()
            .any(|class| class.as_str() == "workspace-shell")
    );
    assert_eq!(app.role.as_deref(), Some("application"));
    assert_eq!(
        app.attributes.get("data-view").map(String::as_str),
        Some("document")
    );
    assert_eq!(
        app.attributes.get("aria-label").map(String::as_str),
        Some("Workspace")
    );
    let topbar = output.layout.find("topbar").unwrap();
    assert_eq!(
        topbar.attributes.get("data-region").map(String::as_str),
        Some("chrome")
    );
    assert_eq!(
        topbar.attributes.get("aria-label").map(String::as_str),
        Some("Top bar")
    );
    assert_eq!(run.element, Element::Button);
    assert_eq!(
        run.text.as_ref().map(|text| text.semantic_text()),
        Some("Run")
    );
    assert_eq!(run.rect.size.width, 72.0);
}

#[test]
fn document_builder_supports_fluent_text_nodes() {
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.section("card").class("card").children(|ui| {
            ui.text_node("title")
                .classes(["card-title", "copyable"])
                .aria("label", "Card title")
                .selectable_text()
                .text("Ready");
            ui.text_node("count")
                .class("metric")
                .data("kind", "row-count")
                .text("42 rows");
        });
    });
    let stylesheet = StyleSheet::new()
        .class("card-title", Style::default().size(120.0, 24.0))
        .class("metric", Style::default().size(80.0, 20.0));
    let output = DocumentEngine::default().update(&mut document, &stylesheet);
    let title = output.snapshot().find("title").unwrap();
    let count = output.snapshot().find("count").unwrap();

    assert_eq!(title.element(), Element::Text);
    assert!(title.has_all_classes(["card-title", "copyable"]));
    assert_eq!(title.aria("label"), Some("Card title"));
    assert!(title.selectable_text());
    assert_eq!(title.text(), Some("Ready".to_owned()));
    assert_eq!(title.rect().size, Size::new(120.0, 24.0));
    assert_eq!(count.data("kind"), Some("row-count"));
    assert_eq!(count.text(), Some("42 rows".to_owned()));
    assert_eq!(count.rect().size, Size::new(80.0, 20.0));
}

#[test]
fn document_builder_supports_conditional_authoring_helpers() {
    let show_badge = true;
    let disabled = false;
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.button("run")
            .class("control")
            .class_if("is-visible", show_badge)
            .class_if("is-disabled", disabled)
            .classes_if(["has-icon", "is-primary"], true)
            .classes_if(["is-hidden", "is-muted"], false)
            .attribute_if("title", "Run query", true)
            .attribute_if("hidden", "hidden", false)
            .data_if("state", "ready", true)
            .data_if("stale", "true", false)
            .aria_if("label", "Run", true)
            .aria_if("disabled", "true", false)
            .text("Run");
    });
    let stylesheet = StyleSheet::new()
        .class("control", Style::default().size(96.0, 32.0))
        .class(
            "is-primary",
            Style::default().background(Color::rgb(220, 238, 255)),
        );
    let output = DocumentEngine::default().update(&mut document, &stylesheet);
    let run = output.snapshot().find("run").unwrap();

    assert!(run.has_all_classes(["control", "is-visible", "has-icon", "is-primary"]));
    assert!(!run.has_class("is-disabled"));
    assert!(!run.has_class("is-hidden"));
    assert!(!run.has_class("is-muted"));
    assert_eq!(run.attribute("title"), Some("Run query"));
    assert_eq!(run.attribute("hidden"), None);
    assert_eq!(run.data("state"), Some("ready"));
    assert_eq!(run.data("stale"), None);
    assert_eq!(run.aria("label"), Some("Run"));
    assert_eq!(run.aria("disabled"), None);
    assert_eq!(run.rect().size, Size::new(96.0, 32.0));
    assert_eq!(run.style().background, Some(Color::rgb(220, 238, 255)));

    let spec = ElementSpec::div()
        .class_if("included", true)
        .class_if("excluded", false)
        .data_if("mode", "demo", true)
        .aria_if("hidden", "true", false);
    assert!(
        spec.classes
            .iter()
            .any(|class| class.as_str() == "included")
    );
    assert!(
        !spec
            .classes
            .iter()
            .any(|class| class.as_str() == "excluded")
    );
    assert_eq!(
        spec.attributes.get("data-mode").map(String::as_str),
        Some("demo")
    );
    assert_eq!(spec.attributes.get("aria-hidden"), None);
}

#[test]
fn document_builder_supports_conditional_behavior_helpers() {
    let can_run = true;
    let can_cancel = false;
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.button("run")
            .interactive_if(can_run)
            .focused(true)
            .command_if("run", can_run)
            .command_on_if(ElementBehaviorEvent::KeyDown, "run-key", can_run)
            .command_if("cancel", can_cancel)
            .text("Run");
    });
    let stylesheet = StyleSheet::new().id("run", Style::default().size(96.0, 32.0));
    let mut engine = DocumentEngine::default();

    let click_output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput::primary_click(Point::new(8.0, 8.0)),
    );
    let key_output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput::key_down(DocumentKey::Enter),
    );
    let run = click_output.snapshot().find("run").unwrap();

    assert!(run.interactive());
    assert!(click_output.has_command("run", "run"));
    assert!(!click_output.has_command("run", "cancel"));
    assert!(key_output.has_command_intent("run", ElementBehaviorEvent::KeyDown, "run-key"));

    let spec = ElementSpec::button()
        .interactive_if(true)
        .command_if("save", true)
        .command_on_if(ElementBehaviorEvent::KeyDown, "save-key", false);

    assert!(spec.interactive);
    assert_eq!(spec.behavior_hooks.len(), 1);
    assert_eq!(spec.behavior_hooks[0].command, "save");
}

#[test]
fn update_reports_created_retained_and_removed_elements() {
    let mut engine = DocumentEngine::default();
    let stylesheet = probe_stylesheet();
    let mut first = catalog_document("Projects");
    let first_output = engine.update(&mut first, &stylesheet);

    assert!(
        first_output
            .changes
            .created
            .contains(&ElementId::new("catalog"))
    );
    assert!(first_output.changes.retained.is_empty());

    engine.element_state_mut("catalog").unwrap().scroll_y = 42.0;

    let mut second = catalog_document("Flows");
    let second_output = engine.update(&mut second, &stylesheet);

    assert!(
        second_output
            .changes
            .retained
            .contains(&ElementId::new("catalog"))
    );
    assert!(
        second_output
            .changes
            .removed
            .contains(&ElementId::new("Projects"))
    );
    assert_eq!(engine.element_state("catalog").unwrap().scroll_y, 42.0);
}

#[test]
fn visual_clone_preserves_visual_subtree_with_rewritten_ids() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("card-source"),
            Style::default().size(120.0, 48.0),
        )
        .rule(
            StyleSelector::class("clone-root"),
            Style::default().size(120.0, 48.0),
        );
    let mut source = Document::build(Size::new(300.0, 180.0), |ui| {
        ui.element(
            "card",
            ElementSpec::new(Element::Div)
                .class("card-source")
                .interactive()
                .value("source-value"),
            |ui| {
                ui.text_element(
                    "card-label",
                    ElementSpec::new(Element::Text).class("label"),
                    "Card label",
                );
                ui.element(
                    "card-icon",
                    ElementSpec::new(Element::Icon).glyph(des_document::Glyph::DragHandle),
                    |_| {},
                );
            },
        );
    });
    let source_output = engine.update(&mut source, &stylesheet);
    let clone = source_output
        .snapshot()
        .find("card")
        .expect("source card exists")
        .visual_clone();

    let mut cloned = Document::build(Size::new(300.0, 180.0), |ui| {
        ui.visual_clone(
            &clone,
            VisualCloneOptions::new("overlay", "overlay/")
                .root_class("clone-root")
                .interactive(false),
        );
    });
    let cloned_output = engine.update(&mut cloned, &stylesheet);

    let overlay = cloned_output.snapshot().find("overlay").unwrap();
    assert!(overlay.has_class("card-source"));
    assert!(overlay.has_class("clone-root"));
    assert_eq!(overlay.value(), Some("source-value"));
    assert!(!overlay.interactive());

    let label = cloned_output.snapshot().find("overlay/card-label").unwrap();
    assert_eq!(label.text(), Some("Card label".to_string()));
    assert!(label.has_class("label"));

    let icon = cloned_output.snapshot().find("overlay/card-icon").unwrap();
    assert_eq!(icon.element(), Element::Icon);
    assert_eq!(
        clone
            .cloned_ids(&VisualCloneOptions::new("overlay", "overlay/").root_class("clone-root"))
            .into_iter()
            .map(|id| id.as_str().to_owned())
            .collect::<Vec<_>>(),
        vec!["overlay", "overlay/card-label", "overlay/card-icon"]
    );
}

#[test]
fn style_rules_resolve_element_class_state_and_id_in_order() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .element(
            Element::Div,
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20)),
        )
        .class(
            "selected",
            Style::default().background(Color::rgb(35, 56, 78)),
        )
        .state(
            ElementStateSelector::Hovered,
            Style::default().background(Color::rgb(40, 70, 95)),
        )
        .id("card", Style::default().radius(7.0))
        .class_state(
            "selected",
            ElementStateSelector::Hovered,
            Style::default().border(Color::rgb(90, 180, 240)),
        )
        .id_state(
            "card",
            ElementStateSelector::Hovered,
            Style::default().border_width(2.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "card",
            ElementSpec::new(Element::Div)
                .class("selected")
                .interactive(),
            |_| {},
        );
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert_eq!(card.style.radius, CornerRadii::all(7.0));
    assert_eq!(card.style.border, Some(Color::rgb(90, 180, 240)));
    assert_eq!(card.style.border_width, Insets::all(2.0));
}

#[test]
fn document_mutation_can_add_remove_and_toggle_classes_before_layout() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .element(
            Element::Div,
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20)),
        )
        .class("expanded", Style::default().size(140.0, 60.0))
        .class(
            "accent",
            Style::default().background(Color::rgb(35, 56, 78)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::div(), |_| {});
    });

    assert_eq!(
        document
            .add_classes("card", ["expanded", "expanded"])
            .unwrap(),
        1
    );
    assert!(document.toggle_class("card", "accent").unwrap());
    assert!(document.add_class("missing", "accent").is_err());

    let output = engine.update(&mut document, &stylesheet);
    let card = output.layout.find("card").unwrap();
    assert_eq!(card.rect.size, Size::new(140.0, 60.0));
    assert_eq!(card.style.background, Some(Color::rgb(35, 56, 78)));

    assert_eq!(
        document
            .remove_classes("card", ["expanded", "missing"])
            .unwrap(),
        1
    );
    assert!(document.toggle_class("card", "accent").unwrap());

    let output = engine.update(&mut document, &stylesheet);
    let card = output.layout.find("card").unwrap();
    assert_eq!(card.rect.size, Size::new(100.0, 40.0));
    assert_eq!(card.style.background, Some(Color::rgb(20, 20, 20)));
}

#[test]
fn document_mutation_can_set_text_value_and_authored_states() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Text),
            Style::default().text_color(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Selected),
            Style::default().background(Color::rgb(35, 56, 78)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Disabled),
            Style::default().text_color(Color::rgb(90, 96, 102)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Focused),
            Style::default().border(Color::rgb(88, 157, 230)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.text("label", "Short");
        ui.element(
            "control",
            ElementSpec::button().interactive().value("initial"),
            |_| {},
        );
    });

    let output = engine.update(&mut document, &stylesheet);
    let label = output.layout.find("label").unwrap();
    assert_eq!(
        label.text.as_ref().map(|text| text.semantic_text()),
        Some("Short")
    );
    assert_eq!(label.rect.size.width, 38.0);

    assert!(document.set_text("label", "Much longer text").unwrap());
    assert!(document.set_value("control", "updated").unwrap());
    assert!(document.set_data("control", "state", "busy").unwrap());
    assert!(document.set_aria("control", "label", "Busy").unwrap());
    assert_eq!(
        document
            .set_attributes(
                "control",
                [("data-phase", "loading"), ("aria-busy", "true")]
            )
            .unwrap(),
        2
    );
    assert!(document.select("control").unwrap());
    assert!(document.disable("control").unwrap());
    assert!(document.focus("control").unwrap());

    let output = engine.update(&mut document, &stylesheet);
    let label = output.layout.find("label").unwrap();
    let control = output.layout.find("control").unwrap();

    assert_eq!(
        label.text.as_ref().map(|text| text.semantic_text()),
        Some("Much longer text")
    );
    assert_eq!(label.rect.size.width, 120.0);
    assert_eq!(control.value.as_deref(), Some("updated"));
    assert_eq!(
        control.attributes.get("data-state").map(String::as_str),
        Some("busy")
    );
    assert_eq!(
        control.attributes.get("data-phase").map(String::as_str),
        Some("loading")
    );
    assert_eq!(
        control.attributes.get("aria-label").map(String::as_str),
        Some("Busy")
    );
    assert_eq!(
        control.attributes.get("aria-busy").map(String::as_str),
        Some("true")
    );
    assert_eq!(control.style.background, Some(Color::rgb(35, 56, 78)));
    assert_eq!(control.style.text_color, Color::rgb(90, 96, 102));
    assert_eq!(control.style.border, Some(Color::rgb(88, 157, 230)));
    assert!(!control.interactive);

    assert_eq!(
        document
            .remove_attributes("control", ["data-phase", "aria-busy", "missing"])
            .unwrap(),
        2
    );
    assert!(document.remove_data("control", "state").unwrap());
    assert!(document.remove_aria("control", "label").unwrap());
    assert!(document.deselect("control").unwrap());
    assert!(document.enable("control").unwrap());
    assert!(document.blur("control").unwrap());

    let output = engine.update(&mut document, &stylesheet);
    let control = output.layout.find("control").unwrap();
    assert!(control.attributes.is_empty());
    assert_eq!(control.style.background, None);
    assert_eq!(control.style.border, None);
    assert!(control.interactive);
}

#[test]
fn document_snapshot_queries_resolved_elements_without_mutation_access() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default().size(80.0, 30.0),
        )
        .rule(
            StyleSelector::class("drop-zone"),
            Style::default().background(Color::rgb(35, 56, 78)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "drop-target",
            ElementSpec::new(Element::Div)
                .classes(["drop-zone", "accepts-files"])
                .data("state", "ready")
                .aria("label", "Drop target")
                .value("target-a")
                .interactive(),
            |ui| {
                ui.text("drop-label", "Drop here");
                ui.element(
                    "drop-helper",
                    ElementSpec::new(Element::Div).class("drop-helper"),
                    |_| {},
                );
            },
        );
        ui.element("plain-card", ElementSpec::new(Element::Div), |_| {});
    });

    let output = engine.update(&mut document, &stylesheet);
    let snapshot = output.snapshot();
    let drop_target = snapshot.require("drop-target").unwrap();

    assert_eq!(snapshot.root().id().as_str(), "root");
    assert!(snapshot.contains("drop-target"));
    assert!(!snapshot.contains("missing"));
    assert_eq!(snapshot.require("missing").unwrap_err().id(), "missing");
    assert_eq!(
        snapshot.require("missing").unwrap_err().to_string(),
        "document element 'missing' was not found"
    );
    assert_eq!(drop_target.element(), Element::Div);
    assert!(drop_target.id_is("drop-target"));
    assert!(drop_target.is_element(Element::Div));
    assert!(drop_target.has_class("drop-zone"));
    assert!(drop_target.has_all_classes(["drop-zone", "accepts-files"]));
    assert!(drop_target.has_any_class(["missing", "accepts-files"]));
    assert_eq!(drop_target.data("state"), Some("ready"));
    assert_eq!(drop_target.aria("label"), Some("Drop target"));
    assert!(drop_target.has_data("state", "ready"));
    assert!(drop_target.has_aria("label", "Drop target"));
    assert_eq!(drop_target.value(), Some("target-a"));
    assert!(drop_target.interactive());
    assert_eq!(drop_target.rect().size, Size::new(80.0, 30.0));
    assert_eq!(drop_target.child_count(), 2);
    assert!(!drop_target.is_empty());
    assert!(drop_target.contains("drop-label"));
    assert!(drop_target.require("missing-child").is_err());
    assert_eq!(
        drop_target
            .children()
            .into_iter()
            .map(|child| child.id().as_str().to_owned())
            .collect::<Vec<_>>(),
        vec!["drop-label", "drop-helper"]
    );
    assert!(
        drop_target
            .find("drop-helper")
            .unwrap()
            .has_class("drop-helper")
    );
    assert_eq!(drop_target.elements_with_class("drop-helper").len(), 1);
    assert!(drop_target.contains_class("drop-zone"));
    assert_eq!(drop_target.count_by_element(Element::Text), 1);
    assert!(
        drop_target
            .first_by_element(Element::Text)
            .is_some_and(|element| element.id_is("drop-label"))
    );
    assert_eq!(
        snapshot.require("drop-label").unwrap().text(),
        Some("Drop here".to_string())
    );
    assert_eq!(snapshot.elements_with_class("drop-zone").len(), 1);
    assert!(snapshot.contains_class("drop-zone"));
    assert!(
        snapshot
            .first_with_class("drop-zone")
            .is_some_and(|element| element.id_is("drop-target"))
    );
    assert_eq!(snapshot.count_with_class("drop-zone"), 1);
    assert_eq!(snapshot.elements_by_element(Element::Div).len(), 3);
    assert!(snapshot.contains_element(Element::Div));
    assert!(
        snapshot
            .first_by_element(Element::Div)
            .is_some_and(|element| element.id_is("drop-target"))
    );
    assert_eq!(snapshot.count_by_element(Element::Div), 3);
}

#[test]
fn document_snapshot_hit_test_returns_target_and_path() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default().size(120.0, 80.0),
        )
        .rule(StyleSelector::id("base"), Style::default().size(80.0, 40.0))
        .rule(
            StyleSelector::id("overlay"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(20.0))
                .top(Length::Px(10.0))
                .z_index(5)
                .size(80.0, 40.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("base", ElementSpec::new(Element::Div), |_| {});
            ui.element("overlay", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let hit = output
        .snapshot()
        .hit_test(Point::new(30.0, 20.0))
        .expect("expected hit result");
    let path: Vec<_> = hit
        .path
        .iter()
        .map(|element| element.id().as_str())
        .collect();

    assert_eq!(hit.target.id().as_str(), "overlay");
    assert_eq!(hit.point, Point::new(30.0, 20.0));
    assert_eq!(path, vec!["root", "panel", "overlay"]);
}

#[test]
fn viewport_max_width_rule_applies_when_document_viewport_matches() {
    let mut document = Document::build(Size::new(420.0, 320.0), |ui| {
        ui.element(
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
            |_| {},
        );
    });
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("panel"),
            Style::default().size(320.0, 48.0),
        )
        .viewport_max_width(
            480.0,
            StyleSelector::class("panel"),
            Style::default().size(180.0, 48.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(output.layout.find("panel").unwrap().rect.size.width, 180.0);
}

#[test]
fn viewport_max_width_rule_is_ignored_when_document_viewport_is_wider() {
    let mut document = Document::build(Size::new(640.0, 320.0), |ui| {
        ui.element(
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
            |_| {},
        );
    });
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("panel"),
            Style::default().size(320.0, 48.0),
        )
        .viewport_max_width(
            480.0,
            StyleSelector::class("panel"),
            Style::default().size(180.0, 48.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(output.layout.find("panel").unwrap().rect.size.width, 320.0);
}

#[test]
fn viewport_rule_can_match_width_and_height_ranges() {
    let mut document = Document::build(Size::new(720.0, 520.0), |ui| {
        ui.element(
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
            |_| {},
        );
    });
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("panel"),
            Style::default().size(320.0, 48.0),
        )
        .viewport_rule(
            ViewportQuery::min_width(700.0)
                .with_max_width(760.0)
                .with_min_height(500.0)
                .with_max_height(560.0),
            StyleSelector::class("panel"),
            Style::default().size(480.0, 72.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(output.layout.find("panel").unwrap().rect.size.width, 480.0);
    assert_eq!(output.layout.find("panel").unwrap().rect.size.height, 72.0);
}

#[test]
fn container_max_width_rule_applies_from_parent_resolved_width() {
    let mut document = Document::build(Size::new(800.0, 320.0), |ui| {
        ui.element(
            "container",
            ElementSpec::new(Element::Div).class("container"),
            |ui| {
                ui.element(
                    "panel",
                    ElementSpec::new(Element::Div).class("panel"),
                    |_| {},
                );
            },
        );
    });
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("container"),
            Style::default().size(360.0, 120.0),
        )
        .rule(
            StyleSelector::class("panel"),
            Style::default().size(320.0, 48.0),
        )
        .container_max_width(
            420.0,
            StyleSelector::class("panel"),
            Style::default().size(180.0, 48.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(output.layout.find("panel").unwrap().rect.size.width, 180.0);
}

#[test]
fn container_max_width_rule_is_ignored_when_parent_is_wider() {
    let mut document = Document::build(Size::new(800.0, 320.0), |ui| {
        ui.element(
            "container",
            ElementSpec::new(Element::Div).class("container"),
            |ui| {
                ui.element(
                    "panel",
                    ElementSpec::new(Element::Div).class("panel"),
                    |_| {},
                );
            },
        );
    });
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("container"),
            Style::default().size(520.0, 120.0),
        )
        .rule(
            StyleSelector::class("panel"),
            Style::default().size(320.0, 48.0),
        )
        .container_max_width(
            420.0,
            StyleSelector::class("panel"),
            Style::default().size(180.0, 48.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(output.layout.find("panel").unwrap().rect.size.width, 320.0);
}

#[test]
fn container_rule_can_match_width_and_height_ranges() {
    let mut document = Document::build(Size::new(800.0, 420.0), |ui| {
        ui.element(
            "container",
            ElementSpec::new(Element::Div).class("container"),
            |ui| {
                ui.element(
                    "panel",
                    ElementSpec::new(Element::Div).class("panel"),
                    |_| {},
                );
            },
        );
    });
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("container"),
            Style::default().size(480.0, 220.0),
        )
        .rule(
            StyleSelector::class("panel"),
            Style::default().size(320.0, 48.0),
        )
        .container_rule(
            des_document::ContainerQuery::min_width(460.0)
                .with_max_width(500.0)
                .with_min_height(200.0)
                .with_max_height(240.0),
            StyleSelector::class("panel"),
            Style::default().size(240.0, 72.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(output.layout.find("panel").unwrap().rect.size.width, 240.0);
    assert_eq!(output.layout.find("panel").unwrap().rect.size.height, 72.0);
}

#[test]
fn compound_selectors_require_all_parts_without_specificity_weighting() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("surface"),
            Style::default().background(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::compound()
                .element(Element::Div)
                .class("surface")
                .class("compact")
                .selector(),
            Style::default().background(Color::rgb(35, 56, 78)),
        )
        .rule(
            StyleSelector::compound()
                .class("surface")
                .class("compact")
                .state(ElementStateSelector::Selected)
                .selector(),
            Style::default().border(Color::rgb(90, 180, 240)),
        )
        .rule(
            StyleSelector::class("surface"),
            Style::default().radius(3.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "matching",
            ElementSpec::new(Element::Div)
                .class("surface")
                .class("compact")
                .selected(true)
                .interactive(),
            |_| {},
        );
        ui.element(
            "missing-compact",
            ElementSpec::new(Element::Div)
                .class("surface")
                .interactive(),
            |_| {},
        );
    });

    let output = engine.update(&mut document, &stylesheet);
    let matching = output.layout.find("matching").unwrap();
    let missing = output.layout.find("missing-compact").unwrap();

    assert_eq!(matching.style.background, Some(Color::rgb(35, 56, 78)));
    assert_eq!(matching.style.border, Some(Color::rgb(90, 180, 240)));
    assert_eq!(matching.style.radius, CornerRadii::all(3.0));
    assert_eq!(missing.style.background, Some(Color::rgb(20, 20, 20)));
    assert_eq!(missing.style.border, None);
}

#[test]
fn structural_selectors_match_first_last_and_nth_children() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default().size(20.0, 20.0),
        )
        .rule(
            StyleSelector::first_child(),
            Style::default().background(Color::rgb(10, 20, 30)),
        )
        .rule(
            StyleSelector::nth_child(2),
            Style::default().background(Color::rgb(40, 50, 60)),
        )
        .rule(
            StyleSelector::last_child(),
            Style::default().border(Color::rgb(70, 80, 90)),
        )
        .rule(
            StyleSelector::compound()
                .class("item")
                .nth_child(3)
                .selector(),
            Style::default().radius(9.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "first",
                ElementSpec::new(Element::Div).class("item"),
                |_| {},
            );
            ui.element(
                "second",
                ElementSpec::new(Element::Div).class("item"),
                |_| {},
            );
            ui.element(
                "third",
                ElementSpec::new(Element::Div).class("item"),
                |_| {},
            );
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let first = output.layout.find("first").unwrap();
    let second = output.layout.find("second").unwrap();
    let third = output.layout.find("third").unwrap();

    assert_eq!(first.style.background, Some(Color::rgb(10, 20, 30)));
    assert_eq!(first.style.border, None);
    assert_eq!(second.style.background, Some(Color::rgb(40, 50, 60)));
    assert_eq!(third.style.border, Some(Color::rgb(70, 80, 90)));
    assert_eq!(third.style.radius, CornerRadii::all(9.0));
}

#[test]
fn nth_child_formula_selectors_match_repeating_child_positions() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default().size(20.0, 20.0),
        )
        .rule(
            StyleSelector::nth_child_odd(),
            Style::default().background(Color::rgb(10, 20, 30)),
        )
        .rule(
            StyleSelector::compound()
                .class("item")
                .nth_child_even()
                .selector(),
            Style::default().border(Color::rgb(40, 50, 60)),
        )
        .rule(
            StyleSelector::nth_child_formula(3, 2),
            Style::default().radius(7.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            for index in 1..=6 {
                ui.element(
                    format!("item-{index}"),
                    ElementSpec::new(Element::Div).class("item"),
                    |_| {},
                );
            }
        });
    });

    let output = engine.update(&mut document, &stylesheet);

    for index in [1, 3, 5] {
        assert_eq!(
            output
                .layout
                .find(format!("item-{index}").as_str())
                .unwrap()
                .style
                .background,
            Some(Color::rgb(10, 20, 30))
        );
    }
    for index in [2, 4, 6] {
        assert_eq!(
            output
                .layout
                .find(format!("item-{index}").as_str())
                .unwrap()
                .style
                .border,
            Some(Color::rgb(40, 50, 60))
        );
    }
    for index in [2, 5] {
        assert_eq!(
            output
                .layout
                .find(format!("item-{index}").as_str())
                .unwrap()
                .style
                .radius,
            CornerRadii::all(7.0)
        );
    }
}

#[test]
fn border_and_radius_rules_can_target_individual_sides_and_corners() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("card"),
            Style::default()
                .size(120.0, 80.0)
                .border_width(2.0)
                .radius(4.0),
        )
        .rule(
            StyleSelector::id("card"),
            Style::default()
                .border_left_width(8.0)
                .border_bottom_width(5.0)
                .top_right_radius(14.0)
                .bottom_left_radius(0.0),
        );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div), |_| {});
    });

    let output = engine.update(&mut document, &stylesheet);
    let card = output.layout.find("card").unwrap();

    assert_eq!(
        card.style.border_width,
        Insets {
            top: 2.0,
            right: 2.0,
            bottom: 5.0,
            left: 8.0,
        }
    );
    assert_eq!(
        card.style.radius,
        CornerRadii {
            top_left: 4.0,
            top_right: 14.0,
            bottom_right: 4.0,
            bottom_left: 0.0,
        }
    );
}

#[test]
fn transitioned_state_rules_ease_visual_style_properties() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20))
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div).interactive(), |_| {});
    });

    engine.update(&mut document, &stylesheet);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(31, 48, 62)));
    assert!(output.metrics.reused_input_layout);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let card = output.layout.find("card").unwrap();

    assert!(card.style.background.unwrap().r > 31);
    assert!(output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(!output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);

    let output = (0..28)
        .map(|_| {
            engine.update_with_input(
                &mut document,
                &stylesheet,
                DocumentInput {
                    pointer: Some(PointerInput {
                        position: Point::new(2.0, 2.0),
                        primary_delta: Point::ZERO,
                        primary_down: false,
                        primary_pressed: false,
                        primary_clicked: false,
                        primary_click_count: 0,
                        secondary_clicked: false,
                        time_seconds: 0.0,
                    }),
                    scroll_delta: Point::ZERO,
                    keys: Vec::new(),
                },
            )
        })
        .last()
        .unwrap();
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);
}

#[test]
fn untransitioned_hover_color_reuses_layout_and_updates_paint() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div).interactive(), |_| {});
    });

    engine.update(&mut document, &stylesheet);
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(output.metrics.reused_input_layout);
    assert!(output.metrics.input_changed_state);
    assert!(output.metrics.animation_changed_style);
    assert!(!output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);
}

#[test]
fn untransitioned_hover_layout_change_rebuilds_layout() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default().size(100.0, 40.0),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().size(140.0, 40.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div).interactive(), |_| {});
    });

    engine.update(&mut document, &stylesheet);
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(140.0, 40.0));
    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn transitioned_state_rules_ease_layout_and_box_model_properties() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .min_size(20.0, 20.0)
                .max_size(180.0, 120.0)
                .padding(Insets::all(4.0))
                .margin(Insets::all(2.0))
                .gap(4.0)
                .border_width(2.0)
                .radius(4.0)
                .font_size(12.0)
                .transition(Transition::linear(0.25)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default()
                .size(140.0, 80.0)
                .min_size(40.0, 60.0)
                .max_size(220.0, 160.0)
                .padding(Insets::all(12.0))
                .margin(Insets::all(10.0))
                .gap(20.0)
                .border_width(10.0)
                .radius(20.0)
                .font_size(20.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div).interactive(), |_| {});
    });

    engine.update(&mut document, &stylesheet);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(4.0, 4.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(110.0, 50.0));
    assert_eq!(card.style.min_size, Size::new(25.0, 30.0));
    assert_eq!(card.style.max_size, Size::new(190.0, 130.0));
    assert_eq!(card.style.padding, Insets::all(6.0));
    assert_eq!(card.style.margin, Insets::all(4.0));
    assert_eq!(card.style.gap, Length::Px(8.0));
    assert_eq!(card.style.border_width, Insets::all(4.0));
    assert_eq!(card.style.radius, CornerRadii::all(8.0));
    assert_eq!(card.style.font_size, 14.0);
    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(4.0, 4.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn column_layout_applies_padding_gap_and_margin() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("catalog"),
            Style::default().padding(Insets::all(10.0)).gap(4.0),
        )
        .rule(
            StyleSelector::class("indented"),
            Style::default().margin(Insets::symmetric(3.0, 2.0)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("catalog", ElementSpec::new(Element::Div), |ui| {
            ui.text("one", "One");
            ui.element(
                "two",
                ElementSpec::new(Element::Text).class("indented"),
                |_| {},
            );
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let one = output.layout.find("one").unwrap();
    let two = output.layout.find("two").unwrap();

    assert_eq!(one.rect.origin, Point::new(10.0, 10.0));
    assert_eq!(two.rect.origin, Point::new(13.0, 34.0));
}

#[test]
fn fill_width_uses_parent_content_width_after_box_model() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(200.0, 120.0)
                .border_width(2.0)
                .padding(Insets::symmetric(12.0, 8.0)),
        )
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .width_fill()
                .height(Length::Px(24.0))
                .margin(Insets::symmetric(3.0, 0.0)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("row", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let row = output.layout.find("row").unwrap();

    assert_eq!(row.rect.origin, Point::new(17.0, 10.0));
    assert_eq!(row.rect.size, Size::new(172.0, 24.0));
}

#[test]
fn wrapped_row_layout_rearranges_children_and_expands_container_height() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .flex_direction(des_document::FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width(Length::Px(120.0))
                .height(Length::Auto)
                .gap(10.0),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(50.0, 20.0),
        );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("row", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "item-0",
                ElementSpec::new(Element::Div).class("item"),
                |_| {},
            );
            ui.element(
                "item-1",
                ElementSpec::new(Element::Div).class("item"),
                |_| {},
            );
            ui.element(
                "item-2",
                ElementSpec::new(Element::Div).class("item"),
                |_| {},
            );
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let row = output.layout.find("row").unwrap();
    let item_0 = output.layout.find("item-0").unwrap();
    let item_1 = output.layout.find("item-1").unwrap();
    let item_2 = output.layout.find("item-2").unwrap();

    assert_eq!(row.rect.size, Size::new(120.0, 50.0));
    assert_eq!(item_0.rect.origin, Point::new(0.0, 0.0));
    assert_eq!(item_1.rect.origin, Point::new(60.0, 0.0));
    assert_eq!(item_2.rect.origin, Point::new(0.0, 30.0));
}

#[test]
fn wrapped_fluid_row_layout_expands_around_variable_height_rows() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .flex_direction(des_document::FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width(Length::Px(756.0))
                .height(Length::Auto)
                .padding(Insets::all(10.0))
                .gap(8.0),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default()
                .width_percent(0.48)
                .flex_basis(Length::Percent(0.48))
                .flex_grow(1.0)
                .height(Length::Px(70.0)),
        )
        .rule(
            StyleSelector::class("tall"),
            Style::default().height(Length::Px(73.0)),
        );
    let mut document = Document::build(Size::new(900.0, 520.0), |ui| {
        ui.element("row", ElementSpec::new(Element::Div), |ui| {
            for index in 0..6 {
                let spec = if index % 2 == 0 {
                    ElementSpec::new(Element::Div).class("item").class("tall")
                } else {
                    ElementSpec::new(Element::Div).class("item")
                };
                ui.element(format!("item-{index}"), spec, |_| {});
            }
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let row = output.layout.find("row").unwrap();

    for index in 0..6 {
        let item = output.layout.find(&format!("item-{index}")).unwrap();
        assert!(
            row.rect.bottom() >= item.rect.bottom(),
            "wrapped parent should contain item {index}"
        );
    }
}

#[test]
fn table_layout_resolves_shared_column_tracks_for_header_and_body_cells() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("customers"),
            Style::default()
                .width(Length::Px(240.0))
                .overflow_x(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Element(Element::Td),
            Style::default().border_width(1.0),
        );
    let mut document = table_fixture_document();
    let output = engine.update(&mut document, &stylesheet);

    let header_customer = output.layout.find("customers-header-customer").unwrap();
    let row_customer = output.layout.find("customers-row-0-customer").unwrap();
    let header_orders = output.layout.find("customers-header-orders").unwrap();
    let row_orders = output.layout.find("customers-row-0-orders").unwrap();

    assert_eq!(header_customer.element, Element::Td);
    assert_close(
        header_customer.rect.size.width,
        row_customer.rect.size.width,
    );
    assert_close(header_orders.rect.origin.x, row_orders.rect.origin.x);
    assert_close(header_orders.rect.size.width, 80.0);
    assert!(
        row_customer.rect.origin.y > header_customer.rect.origin.y,
        "body rows should be laid out below the header row"
    );
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id == ElementId::new("customers")
                && chrome.axis == ScrollAxis::Horizontal
                && chrome.max_scroll > 0.0
        }),
        "table content wider than the styled table frame should expose horizontal overflow"
    );
}

#[test]
fn text_layout_uses_document_wrap_and_truncation_styles() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("wrapped"),
            Style::default()
                .width(Length::Px(90.0))
                .text_wrap_mode(TextWrapMode::Wrap),
        )
        .rule(
            StyleSelector::id("truncated"),
            Style::default()
                .width(Length::Px(90.0))
                .text_layout(TextLayoutStyle {
                    max_lines: Some(1),
                    text_overflow: TextOverflow::Ellipsis,
                    ..TextLayoutStyle::default()
                }),
        );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.text_element(
            "wrapped",
            ElementSpec::new(Element::Text),
            "Customer analytics pipeline preview",
        );
        ui.text_element(
            "truncated",
            ElementSpec::new(Element::Text),
            "Customer analytics pipeline preview",
        );
    });

    let output = engine.update(&mut document, &stylesheet);
    let wrapped = output.layout.find("wrapped").unwrap();
    let truncated = output.layout.find("truncated").unwrap();

    assert!(
        wrapped.text_layout.as_ref().unwrap().line_count > 1,
        "wrapped text should report multiple measured lines"
    );
    assert_eq!(truncated.text_layout.as_ref().unwrap().line_count, 1);
    assert!(truncated.text_layout.as_ref().unwrap().elided);
}

#[test]
fn declared_direction_resolves_start_aligned_text_against_rtl_edge() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(100.0))
            .direction(Direction::Rtl),
    );
    let mut document = Document::build(Size::new(160.0, 80.0), |ui| {
        ui.text_element("label", ElementSpec::new(Element::Text), "abcd");
    });

    let output = engine.update(&mut document, &stylesheet);
    let label = output.layout.find("label").unwrap();
    let line = label.text_layout.as_ref().unwrap().lines.first().unwrap();

    assert_eq!(label.style.direction, Direction::Rtl);
    assert!(line.x_offset > 50.0);
}

#[test]
fn text_layout_respects_padding_and_border_box_size() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().padding(Insets::all(4.0)).border_width(2.0),
    );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.text_element("label", ElementSpec::new(Element::Text), "Hi");
    });

    let output = engine.update(&mut document, &stylesheet);
    let label = output.layout.find("label").unwrap();

    assert_close(label.text_layout.as_ref().unwrap().size.width, 15.0);
    assert_close(label.text_layout.as_ref().unwrap().size.height, 18.0);
    assert_close(label.rect.size.width, 27.0);
    assert_close(label.rect.size.height, 30.0);
}

#[test]
fn text_measurer_cache_key_invalidates_cached_layout() {
    struct FixedTextMeasurer {
        key: TextMeasurerKey,
        width: f32,
    }

    impl TextMeasurer for FixedTextMeasurer {
        fn cache_key(&self) -> TextMeasurerKey {
            self.key
        }

        fn measure_text(&mut self, _request: TextLayoutRequest<'_>) -> TextLayoutResult {
            TextLayoutResult::new(Size::new(self.width, 18.0), 1, false)
        }
    }

    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new();
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.text_element("label", ElementSpec::new(Element::Text), "Text");
    });
    let mut narrow = FixedTextMeasurer {
        key: TextMeasurerKey::new("narrow"),
        width: 24.0,
    };
    let mut wide = FixedTextMeasurer {
        key: TextMeasurerKey::new("wide"),
        width: 96.0,
    };

    let first = engine.update_with_input_and_text_measurer(
        &mut document,
        &stylesheet,
        DocumentInput::default(),
        &mut narrow,
    );
    let second = engine.update_with_input_and_text_measurer(
        &mut document,
        &stylesheet,
        DocumentInput::default(),
        &mut wide,
    );

    assert_close(first.layout.find("label").unwrap().rect.size.width, 24.0);
    assert_close(second.layout.find("label").unwrap().rect.size.width, 96.0);
    assert!(!second.metrics.reused_cached_layout);
}

#[test]
fn selectable_text_tracks_pointer_selection_points() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(160.0))
            .text_wrap_mode(TextWrapMode::Wrap),
    );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "Customer analytics pipeline preview",
        );
    });

    let start = Point::new(4.0, 4.0);
    let end = Point::new(86.0, 24.0);
    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: start,
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let dragging = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: end,
                primary_delta: Point::new(end.x - start.x, end.y - start.y),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    let selection = dragging.text_selection.unwrap();
    assert_eq!(selection.target, ElementId::new("label"));
    assert_eq!(selection.anchor, start);
    assert_eq!(selection.focus, end);
    assert!(selection.focus_index > selection.anchor_index);
    assert!(
        selection
            .selected_text_from("Customer analytics pipeline preview")
            .is_some()
    );
    assert!(selection.active);

    let released = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: end,
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(!released.text_selection.unwrap().active);
}

#[test]
fn selectable_text_exposes_selected_text_for_copy() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .white_space(WhiteSpace::Pre),
    );
    let mut document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "Customer analytics",
        );
    });

    let start = Point::new(0.0, 4.0);
    let end = Point::new(60.0, 4.0);
    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: start,
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: end,
                primary_delta: Point::new(end.x - start.x, end.y - start.y),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert_eq!(output.text_selection().unwrap().char_range(), 0..8);
    assert_eq!(output.text_selection_range(), Some(0..8));
    assert_eq!(
        output.text_selection_target().map(ElementId::as_str),
        Some("label")
    );
    assert!(output.has_text_selection());
    assert!(output.text_selection_is_active());
    assert_eq!(output.selected_text().as_deref(), Some("Customer"));
    assert!(output.snapshot().find("label").unwrap().selectable_text());
    assert!(output.snapshot().find("label").unwrap().copyable_text());
}

#[test]
fn declared_text_transform_changes_layout_text_but_copy_uses_semantic_text() {
    struct RecordingLayoutTextMeasurer {
        layout_text: String,
    }

    impl TextMeasurer for RecordingLayoutTextMeasurer {
        fn cache_key(&self) -> TextMeasurerKey {
            TextMeasurerKey::new("recording-layout-text")
        }

        fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
            self.layout_text = request.text.layout_text().to_owned();
            TextLayoutResult::new(Size::new(160.0, 18.0), 1, false)
        }
    }

    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .white_space(WhiteSpace::Pre)
            .text_transform(TextTransform::Uppercase),
    );
    let mut document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "Straße analytics",
        );
    });
    let mut text_measurer = RecordingLayoutTextMeasurer {
        layout_text: String::new(),
    };

    let output = engine.update_with_input_and_text_measurer(
        &mut document,
        &stylesheet,
        DocumentInput::default(),
        &mut text_measurer,
    );
    let label = output.snapshot().find("label").unwrap();

    assert_eq!(text_measurer.layout_text, "STRASSE ANALYTICS");
    assert_eq!(label.text().as_deref(), Some("Straße analytics"));
}

#[test]
fn selectable_text_can_disable_copy_without_disabling_selection() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .white_space(WhiteSpace::Pre),
    );
    let mut document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(Element::Text)
                .selectable_text()
                .copyable_text(false),
            "Customer analytics",
        );
    });

    let start = Point::new(0.0, 4.0);
    let end = Point::new(60.0, 4.0);
    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: start,
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: end,
                primary_delta: Point::new(end.x - start.x, end.y - start.y),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(output.text_selection.is_some());
    assert!(output.snapshot().find("label").unwrap().selectable_text());
    assert!(!output.snapshot().find("label").unwrap().copyable_text());
    assert_eq!(output.selected_text(), None);
}

#[test]
fn selectable_text_double_click_selects_word_and_word_drags() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .white_space(WhiteSpace::Pre),
    );
    let mut document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "alpha beta gamma",
        );
    });
    let beta = Point::new(56.0, 4.0);
    let gamma = Point::new(100.0, 4.0);

    engine.update_with_input(
        &mut document,
        &stylesheet,
        pointer_input(beta, true, true, false, 0.0),
    );
    engine.update_with_input(
        &mut document,
        &stylesheet,
        pointer_input(beta, false, false, true, 0.05),
    );
    let word = engine.update_with_input(
        &mut document,
        &stylesheet,
        pointer_input(beta, true, true, false, 0.20),
    );

    assert_eq!(word.text_selection.as_ref().unwrap().char_range(), 6..10);
    assert_eq!(word.selected_text().as_deref(), Some("beta"));

    let multi_word = engine.update_with_input(
        &mut document,
        &stylesheet,
        pointer_input(gamma, true, false, false, 0.24),
    );

    assert_eq!(
        multi_word.text_selection.as_ref().unwrap().granularity,
        TextSelectionGranularity::Word
    );
    assert_eq!(multi_word.selected_text().as_deref(), Some("beta gamma"));
}

#[test]
fn selectable_text_double_click_drag_left_keeps_original_word() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .white_space(WhiteSpace::Pre),
    );
    let mut document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "alpha beta gamma",
        );
    });
    let beta = Point::new(56.0, 4.0);
    let alpha = Point::new(12.0, 4.0);

    engine.update_with_input(
        &mut document,
        &stylesheet,
        pointer_input(beta, true, true, false, 0.0),
    );
    engine.update_with_input(
        &mut document,
        &stylesheet,
        pointer_input(beta, false, false, true, 0.05),
    );
    engine.update_with_input(
        &mut document,
        &stylesheet,
        pointer_input(beta, true, true, false, 0.20),
    );
    let multi_word = engine.update_with_input(
        &mut document,
        &stylesheet,
        pointer_input(alpha, true, false, false, 0.24),
    );

    assert_eq!(
        multi_word.text_selection.as_ref().unwrap().granularity,
        TextSelectionGranularity::Word
    );
    assert_eq!(multi_word.selected_text().as_deref(), Some("alpha beta"));
}

#[test]
fn selectable_text_triple_click_selects_paragraph() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .white_space(WhiteSpace::PreLine),
    );
    let mut document = Document::build(Size::new(360.0, 160.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "first paragraph\nsecond paragraph",
        );
    });
    let second = Point::new(8.0, 24.0);

    for (index, time_seconds) in [0.0, 0.2, 0.4].into_iter().enumerate() {
        engine.update_with_input(
            &mut document,
            &stylesheet,
            pointer_input(second, true, true, false, time_seconds),
        );
        if index < 2 {
            engine.update_with_input(
                &mut document,
                &stylesheet,
                pointer_input(second, false, false, true, time_seconds + 0.05),
            );
        }
    }
    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(
        output.text_selection.as_ref().unwrap().granularity,
        TextSelectionGranularity::Paragraph
    );
    assert_eq!(output.selected_text().as_deref(), Some("second paragraph"));
}

#[test]
fn selectable_text_secondary_click_requests_context() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new();
    let mut document = Document::build(Size::new(240.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "copy me",
        );
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(8.0, 4.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: true,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(output.events.iter().any(|event| {
        event.target == ElementId::new("label") && event.kind == DocumentEventKind::ContextRequested
    }));
}

#[test]
fn interactive_element_secondary_click_requests_context() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("button"),
        Style::default().size(100.0, 40.0),
    );
    let mut document = Document::build(Size::new(240.0, 120.0), |ui| {
        ui.element(
            "button",
            ElementSpec::new(Element::Button).interactive(),
            |_| {},
        );
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: true,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(output.events.iter().any(|event| {
        event.target == ElementId::new("button")
            && event.kind == DocumentEventKind::ContextRequested
    }));
}

#[test]
fn border_style_resolves_as_paint_only_property() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("dashed"),
        Style::default()
            .size(80.0, 40.0)
            .border(Color::rgba(20, 20, 24, 255))
            .border_width(3.0)
            .border_style(des_document::BorderStyle::Dashed),
    );
    let mut document = Document::build(Size::new(180.0, 120.0), |ui| {
        ui.element("dashed", ElementSpec::new(Element::Div), |_| {});
    });

    let output = engine.update(&mut document, &stylesheet);
    let dashed = output.snapshot().find("dashed").unwrap();

    assert_eq!(
        dashed.style().border_style,
        des_document::BorderStyle::Dashed
    );
    assert_eq!(dashed.rect().size, Size::new(80.0, 40.0));

    let previous = output.layout.clone();
    let output = engine.update(
        &mut document,
        &StyleSheet::new().rule(
            StyleSelector::id("dashed"),
            Style::default()
                .size(80.0, 40.0)
                .border(Color::rgba(20, 20, 24, 255))
                .border_width(3.0)
                .border_style(des_document::BorderStyle::Dotted),
        ),
    );

    assert!(output.metrics.reused_input_layout);
    assert_eq!(output.layout.rect, previous.rect);
    assert_eq!(
        output
            .snapshot()
            .find("dashed")
            .unwrap()
            .style()
            .border_style,
        des_document::BorderStyle::Dotted
    );
}

#[test]
fn style_rules_resolve_shadow_as_paint_only_property() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("card"),
        Style::default().size(100.0, 40.0).shadows([
            Shadow {
                offset: Point::new(0.0, 2.0),
                blur: 7.0,
                spread: -1.0,
                color: Color::rgba(0, 0, 0, 110),
            },
            Shadow {
                offset: Point::new(0.0, 14.0),
                blur: 28.0,
                spread: -5.0,
                color: Color::rgba(0, 0, 0, 78),
            },
        ]),
    );
    let mut document = Document::build(Size::new(180.0, 100.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div), |_| {});
    });

    let output = engine.update(&mut document, &stylesheet);
    let card = output.snapshot().find("card").unwrap();

    assert_eq!(card.rect().size, Size::new(100.0, 40.0));
    assert_eq!(card.style().shadows.len(), 2);
    assert_eq!(card.style().shadows[0].blur, 7.0);
    assert_eq!(card.style().shadows[1].spread, -5.0);
}

#[test]
fn row_layout_applies_main_and_cross_axis_alignment() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .flex_direction(des_document::FlexDirection::Row)
                .size(160.0, 80.0)
                .gap(10.0)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::End),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(40.0, 20.0),
        );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("row", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "item-0",
                ElementSpec::new(Element::Div).class("item"),
                |_| {},
            );
            ui.element(
                "item-1",
                ElementSpec::new(Element::Div).class("item"),
                |_| {},
            );
        });
    });

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(
        output.layout.find("item-0").unwrap().rect.origin,
        Point::new(35.0, 60.0)
    );
    assert_eq!(
        output.layout.find("item-1").unwrap().rect.origin,
        Point::new(85.0, 60.0)
    );
}

#[test]
fn column_layout_applies_main_and_cross_axis_alignment() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("column"),
            Style::default()
                .flex_direction(des_document::FlexDirection::Column)
                .size(120.0, 120.0)
                .gap(5.0)
                .justify_content(JustifyContent::SpaceBetween)
                .align_items(AlignItems::Center),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(30.0, 20.0),
        );
    let mut document = Document::build(Size::new(180.0, 160.0), |ui| {
        ui.element("column", ElementSpec::new(Element::Div), |ui| {
            for index in 0..3 {
                ui.element(
                    format!("item-{index}"),
                    ElementSpec::new(Element::Div).class("item"),
                    |_| {},
                );
            }
        });
    });

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(
        output.layout.find("item-0").unwrap().rect.origin,
        Point::new(45.0, 0.0)
    );
    assert_eq!(
        output.layout.find("item-1").unwrap().rect.origin,
        Point::new(45.0, 50.0)
    );
    assert_eq!(
        output.layout.find("item-2").unwrap().rect.origin,
        Point::new(45.0, 100.0)
    );
}

#[test]
fn fill_size_does_not_inflate_auto_sized_parent_during_intrinsic_measurement() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default().width(Length::Auto).height(Length::Auto),
        )
        .rule(
            StyleSelector::id("child"),
            Style::default()
                .width_fill()
                .height_fill()
                .min_size(24.0, 24.0),
        );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("child", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();

    assert_eq!(panel.rect.size, Size::new(24.0, 24.0));
}

#[test]
fn max_size_clamps_auto_explicit_and_fill_sizes() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(200.0, 120.0)
                .padding(Insets::all(10.0)),
        )
        .rule(
            StyleSelector::id("auto-child"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .max_size(40.0, 30.0),
        )
        .rule(
            StyleSelector::id("fixed-child"),
            Style::default().size(96.0, 70.0).max_size(42.0, 28.0),
        )
        .rule(
            StyleSelector::id("fill-child"),
            Style::default()
                .width_fill()
                .height_fill()
                .max_size(50.0, 34.0),
        )
        .rule(
            StyleSelector::class("wide"),
            Style::default().size(80.0, 20.0),
        );
    let mut document = Document::build(Size::new(260.0, 180.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("auto-child", ElementSpec::new(Element::Div), |ui| {
                ui.element(
                    "wide-child",
                    ElementSpec::new(Element::Div).class("wide"),
                    |_| {},
                );
            });
            ui.element("fixed-child", ElementSpec::new(Element::Div), |_| {});
            ui.element("fill-child", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(
        output.layout.find("auto-child").unwrap().rect.size,
        Size::new(40.0, 20.0)
    );
    assert_eq!(
        output.layout.find("fixed-child").unwrap().rect.size,
        Size::new(42.0, 28.0)
    );
    assert_eq!(
        output.layout.find("fill-child").unwrap().rect.size,
        Size::new(50.0, 34.0)
    );
}

#[test]
fn absolute_parent_position_uses_parent_content_rect_and_leaves_flow_measurement() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(10.0))
                .border_width(2.0),
        )
        .rule(
            StyleSelector::id("flow-child"),
            Style::default().size(50.0, 20.0),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(7.0))
                .top(Length::Px(5.0))
                .size(40.0, 20.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("absolute-child", ElementSpec::new(Element::Div), |_| {});
            ui.element("flow-child", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();
    let absolute_child = output.layout.find("absolute-child").unwrap();
    let flow_child = output.layout.find("flow-child").unwrap();

    assert_eq!(panel.rect.size, Size::new(74.0, 44.0));
    assert_eq!(flow_child.rect.origin, Point::new(12.0, 12.0));
    assert_eq!(absolute_child.rect.origin, Point::new(19.0, 17.0));
}

#[test]
fn absolute_anchor_positions_against_resolved_element_rect() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(10.0))
                .border_width(2.0),
        )
        .rule(
            StyleSelector::id("anchor"),
            Style::default().size(80.0, 30.0),
        )
        .rule(
            StyleSelector::id("popover"),
            Style::default()
                .absolute_parent()
                .anchor_bottom_start("anchor", 0.0, -1.0)
                .size(60.0, 20.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("popover", ElementSpec::new(Element::Div), |_| {});
            ui.element("anchor", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();
    let anchor = output.layout.find("anchor").unwrap();
    let popover = output.layout.find("popover").unwrap();

    assert_eq!(panel.rect.size, Size::new(104.0, 54.0));
    assert_eq!(anchor.rect.origin, Point::new(12.0, 12.0));
    assert_eq!(popover.rect.origin, Point::new(12.0, 41.0));
    assert_eq!(popover.rect.size, Size::new(60.0, 20.0));
}

#[test]
fn floating_anchor_uses_fallbacks_and_viewport_shift() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("anchor"),
            Style::default()
                .absolute_viewport()
                .left(Length::Px(84.0))
                .top(Length::Px(40.0))
                .size(12.0, 12.0),
        )
        .rule(
            StyleSelector::id("popover"),
            Style::default()
                .floating_to("anchor")
                .floating_placement(des_document::FloatingPlacement::Right)
                .floating_fallbacks([des_document::FloatingPlacement::Left])
                .floating_shift(des_document::FloatingShift::main_and_cross_axis())
                .size(34.0, 24.0),
        );
    let mut document = Document::build(Size::new(100.0, 100.0), |ui| {
        ui.element("anchor", ElementSpec::new(Element::Div), |_| {});
        ui.element("popover", ElementSpec::new(Element::Div), |_| {});
    });

    let output = engine.update(&mut document, &stylesheet);
    let popover = output.layout.find("popover").unwrap();

    assert_eq!(popover.rect.origin, Point::new(50.0, 34.0));
    assert_eq!(popover.rect.size, Size::new(34.0, 24.0));
}

#[test]
fn floating_arrow_is_style_opt_in_metadata() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("anchor"),
            Style::default()
                .absolute_viewport()
                .left(Length::Px(40.0))
                .top(Length::Px(40.0))
                .size(20.0, 10.0),
        )
        .rule(
            StyleSelector::id("plain-popover"),
            Style::default()
                .floating_to("anchor")
                .floating_placement(des_document::FloatingPlacement::Bottom)
                .size(60.0, 20.0),
        )
        .rule(
            StyleSelector::id("arrow-popover"),
            Style::default()
                .floating_to("anchor")
                .floating_placement(des_document::FloatingPlacement::Bottom)
                .floating_offset(24.0, 0.0)
                .floating_arrow_size(12.0, 6.0, 3.0)
                .size(60.0, 20.0),
        );
    let mut document = Document::build(Size::new(140.0, 120.0), |ui| {
        ui.element("anchor", ElementSpec::new(Element::Div), |_| {});
        ui.element("plain-popover", ElementSpec::new(Element::Div), |_| {});
        ui.element("arrow-popover", ElementSpec::new(Element::Div), |_| {});
    });

    let output = engine.update(&mut document, &stylesheet);
    let plain = output.snapshot().find("plain-popover").unwrap();
    let arrow = output.snapshot().find("arrow-popover").unwrap();

    assert_eq!(plain.floating().unwrap().arrow_offset, None);
    assert_eq!(plain.floating().unwrap().arrow_size, None);
    assert_eq!(
        arrow.floating().unwrap().arrow_offset,
        Some(Point::new(24.0, 0.0))
    );
    assert_eq!(
        arrow.floating().unwrap().arrow_size,
        Some(Size::new(12.0, 6.0))
    );
}

#[test]
fn floating_anchor_can_shift_inside_scroll_container_boundary() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(120.0, 80.0)
                .border_width(4.0)
                .overflow_x(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("track"),
            Style::default()
                .width(Length::Px(360.0))
                .height(Length::Px(70.0)),
        )
        .rule(
            StyleSelector::id("anchor"),
            Style::default().size(40.0, 40.0).margin(Insets {
                top: 16.0,
                right: 0.0,
                bottom: 0.0,
                left: 210.0,
            }),
        )
        .rule(
            StyleSelector::id("popover"),
            Style::default()
                .size(70.0, 32.0)
                .floating_to("anchor")
                .floating_placement(des_document::FloatingPlacement::Bottom)
                .floating_boundary_to("scroll-panel")
                .floating_shift(des_document::FloatingShift::new(false, true)),
        );
    let mut document = Document::build(Size::new(260.0, 140.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("track", ElementSpec::new(Element::Div), |ui| {
                ui.element("anchor", ElementSpec::new(Element::Div), |_| {});
            });
            ui.element("popover", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let panel = output.layout.find("scroll-panel").unwrap();
    let anchor = output.layout.find("anchor").unwrap();
    let popover = output.layout.find("popover").unwrap();
    let boundary_left = panel.rect.origin.x + 4.0;
    let boundary_right = panel.rect.right() - 4.0;
    assert!(anchor.rect.origin.x > boundary_right);
    assert_close(popover.rect.right(), boundary_right);

    engine.element_state_mut("scroll-panel").unwrap().scroll_x = 256.0;
    let output = engine.update(&mut document, &stylesheet);
    let anchor = output.layout.find("anchor").unwrap();
    let popover = output.layout.find("popover").unwrap();
    assert!(anchor.rect.origin.x < boundary_left);
    assert_close(popover.rect.origin.x, boundary_left);
}

#[test]
fn absolute_viewport_position_uses_window_rect() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(120.0, 80.0)
                .padding(Insets::all(10.0)),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_viewport()
                .right(Length::Px(8.0))
                .bottom(Length::Px(9.0))
                .size(40.0, 20.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("absolute-child", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let absolute_child = output.layout.find("absolute-child").unwrap();

    assert_eq!(absolute_child.rect.origin, Point::new(272.0, 171.0));
}

#[test]
fn pointer_input_can_target_absolute_child_outside_parent_box() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default().size(60.0, 40.0),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_viewport()
                .left(Length::Px(140.0))
                .top(Length::Px(80.0))
                .size(40.0, 20.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "absolute-child",
                ElementSpec::new(Element::Div).interactive(),
                |_| {},
            );
        });
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(150.0, 90.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("absolute-child")));
    assert!(engine.element_state("absolute-child").unwrap().pressed);
}

#[test]
fn absolute_viewport_child_escapes_ancestor_overflow_clip() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("clipper"),
            Style::default().size(60.0, 40.0).overflow(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_viewport()
                .left(Length::Px(140.0))
                .top(Length::Px(80.0))
                .size(40.0, 20.0),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("clipper", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "absolute-child",
                ElementSpec::new(Element::Div).interactive(),
                |_| {},
            );
        });
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(150.0, 90.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert_eq!(
        output
            .snapshot()
            .find("absolute-child")
            .unwrap()
            .clip_rect(),
        des_document::ClipRect::from_rect(des_document::Rect::new(0.0, 0.0, 320.0, 200.0))
    );
    assert_eq!(output.hit_id, Some(ElementId::new("absolute-child")));
    assert!(engine.element_state("absolute-child").unwrap().pressed);
}

#[test]
fn pointer_input_targets_interactive_owner_instead_of_inner_text() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::Element(Element::Div),
        Style::default().size(100.0, 40.0),
    );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div).interactive(), |ui| {
            ui.text("label", "Click target");
        });
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(4.0, 4.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("card")));
    let card_state = engine.element_state("card").unwrap();
    assert!(card_state.hovered);
    assert!(card_state.pressed);
    assert_eq!(card_state.click_count, 1);

    let label_state = engine.element_state("label").unwrap();
    assert!(label_state.hovered);
    assert!(!label_state.pressed);
    assert_eq!(label_state.click_count, 0);
}

#[test]
fn pointer_input_emits_document_interaction_events() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::Element(Element::Div),
        Style::default().size(100.0, 40.0),
    );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div).interactive(), |_| {});
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("card")));
    assert!(
        output
            .events
            .contains(&DocumentEvent::pointer_entered("card"))
    );
    assert!(output.events.contains(&DocumentEvent::pressed("card")));
    assert!(output.events.contains(&DocumentEvent::clicked("card")));

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(output.events.contains(&DocumentEvent::released("card")));

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(180.0, 120.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(
        output
            .events
            .contains(&DocumentEvent::pointer_exited("card"))
    );
}

#[test]
fn document_engine_captures_primary_pointer_drag() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("card"),
        Style::default().size(100.0, 40.0),
    );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::new(Element::Div).interactive(), |_| {});
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(12.0, 10.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(
        output.active_drag.is_none(),
        "pointer down should capture a pending drag without activating it"
    );
    assert!(output.completed_drag.is_none());
    assert!(!output.events.contains(&DocumentEvent::drag_started("card")));
    assert!(!engine.element_state("card").unwrap().dragging);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(180.0, 120.0),
                primary_delta: Point::new(168.0, 110.0),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    let drag = output
        .active_drag
        .expect("movement past activation distance should start drag");
    assert_eq!(drag.target, ElementId::new("card"));
    assert_eq!(drag.origin, Point::new(12.0, 10.0));
    assert_eq!(drag.current, Point::new(180.0, 120.0));
    assert_eq!(drag.delta, Point::new(168.0, 110.0));
    assert_eq!(drag.pointer_offset, Point::new(12.0, 10.0));
    assert!(output.completed_drag.is_none());
    assert_eq!(output.hit_id, Some(ElementId::new("card")));
    assert!(output.events.contains(&DocumentEvent::drag_started("card")));

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(180.0, 120.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    let completed = output
        .completed_drag
        .expect("release should expose completed drag");
    assert!(output.active_drag.is_none());
    assert_eq!(completed.target, ElementId::new("card"));
    assert_eq!(completed.current, Point::new(180.0, 120.0));
    assert_eq!(completed.delta, Point::new(168.0, 110.0));
    assert!(output.events.contains(&DocumentEvent::drag_ended("card")));
    assert!(!engine.element_state("card").unwrap().dragging);
}

#[test]
fn scroll_delta_updates_hovered_scroll_container_state() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(180.0, 80.0)
                .padding(Insets::all(8.0))
                .gap(4.0)
                .border_width(5.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::class("scroll-row"),
            Style::default().size(120.0, 36.0),
        );
    let mut document = overflowing_scroll_document();

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -24.0),
            keys: Vec::new(),
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("row-0")));
    assert!(output.events.contains(&DocumentEvent::scrolled(
        "scroll-panel",
        ScrollAxis::Vertical
    )));
    assert_eq!(engine.element_state("scroll-panel").unwrap().scroll_y, 24.0);

    let output = engine.update(&mut document, &stylesheet);
    let first_row = output.layout.find("row-0").unwrap();
    assert_eq!(first_row.rect.origin.y, -11.0);
}

#[test]
fn horizontal_overflow_scrolls_child_content_on_x_axis() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .flex_direction(des_document::FlexDirection::Row)
                .size(80.0, 70.0)
                .gap(4.0)
                .overflow_x(Overflow::Scroll),
        )
        .rule(
            StyleSelector::class("scroll-item"),
            Style::default().size(50.0, 32.0),
        );
    let mut document = Document::build(Size::new(180.0, 120.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "item-0",
                ElementSpec::new(Element::Div).class("scroll-item"),
                |_| {},
            );
            ui.element(
                "item-1",
                ElementSpec::new(Element::Div).class("scroll-item"),
                |_| {},
            );
            ui.element(
                "item-2",
                ElementSpec::new(Element::Div).class("scroll-item"),
                |_| {},
            );
        });
    });

    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(-30.0, 0.0),
            keys: Vec::new(),
        },
    );

    assert_eq!(engine.element_state("scroll-panel").unwrap().scroll_x, 30.0);
    let output = engine.update(&mut document, &stylesheet);
    assert_eq!(
        output.layout.find("item-0").unwrap().rect.origin,
        Point::new(-30.0, 0.0)
    );
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Horizontal
                && chrome.max_scroll > 0.0
        }),
        "horizontal overflow should emit horizontal scroll chrome"
    );
}

#[test]
fn overflow_clip_chain_is_axis_aware_for_hit_testing() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("clipper"),
            Style::default()
                .size(80.0, 60.0)
                .overflow_x(Overflow::Clip)
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::id("below"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(20.0))
                .top(Length::Px(90.0))
                .size(24.0, 18.0),
        )
        .rule(
            StyleSelector::id("side"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(90.0))
                .top(Length::Px(90.0))
                .size(24.0, 18.0),
        );
    let mut document = Document::build(Size::new(180.0, 180.0), |ui| {
        ui.element("clipper", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "below",
                ElementSpec::new(Element::Div).interactive(),
                |_| {},
            );
            ui.element("side", ElementSpec::new(Element::Div).interactive(), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let below = output.snapshot().find("below").unwrap();

    assert_eq!(below.clip_rect().left, Some(0.0));
    assert_eq!(below.clip_rect().right, Some(80.0));
    assert_eq!(below.clip_rect().top, Some(0.0));
    assert_eq!(below.clip_rect().bottom, Some(180.0));
    assert_eq!(
        output
            .snapshot()
            .hit_test(Point::new(30.0, 100.0))
            .unwrap()
            .target
            .id(),
        &ElementId::new("below")
    );
    assert_ne!(
        output
            .snapshot()
            .hit_test(Point::new(100.0, 100.0))
            .unwrap()
            .target
            .id(),
        &ElementId::new("side")
    );
}

#[test]
fn declared_visible_cross_axis_normalizes_to_auto_clip() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("clipper"),
            Style::default()
                .size(80.0, 60.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::id("below"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(20.0))
                .top(Length::Px(90.0))
                .size(24.0, 18.0),
        )
        .rule(
            StyleSelector::id("side"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(90.0))
                .top(Length::Px(20.0))
                .size(24.0, 18.0),
        );
    let mut document = Document::build(Size::new(180.0, 180.0), |ui| {
        ui.element("clipper", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "below",
                ElementSpec::new(Element::Div).interactive(),
                |_| {},
            );
            ui.element("side", ElementSpec::new(Element::Div).interactive(), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let clipper = output.snapshot().find("clipper").unwrap();
    let below = output.snapshot().find("below").unwrap();

    assert_eq!(clipper.style().overflow_x, Overflow::Scroll);
    assert_eq!(clipper.style().overflow_y, Overflow::Auto);
    assert_eq!(below.clip_rect().left, Some(0.0));
    assert_eq!(below.clip_rect().right, Some(80.0));
    assert_eq!(below.clip_rect().top, Some(0.0));
    assert_eq!(below.clip_rect().bottom, Some(60.0));
    assert_ne!(
        output
            .snapshot()
            .hit_test(Point::new(30.0, 100.0))
            .unwrap()
            .target
            .id(),
        &ElementId::new("below")
    );
    assert_ne!(
        output
            .snapshot()
            .hit_test(Point::new(100.0, 30.0))
            .unwrap()
            .target
            .id(),
        &ElementId::new("side")
    );
}

#[test]
fn normalized_auto_axis_scrolls_overflowing_content() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroller"),
            Style::default()
                .size(100.0, 60.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::class("scroll-row"),
            Style::default().size(80.0, 30.0),
        );
    let mut document = Document::build(Size::new(140.0, 100.0), |ui| {
        ui.element("scroller", ElementSpec::new(Element::Div), |ui| {
            for index in 0..4 {
                ui.element(
                    format!("row-{index}"),
                    ElementSpec::new(Element::Div).class("scroll-row"),
                    |_| {},
                );
            }
        });
    });

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(10.0, 10.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -20.0),
            keys: Vec::new(),
        },
    );

    let scroller = output.snapshot().find("scroller").unwrap();
    assert_eq!(scroller.style().overflow_y, Overflow::Auto);
    assert!(
        output
            .events
            .contains(&DocumentEvent::scrolled("scroller", ScrollAxis::Vertical))
    );
    assert_eq!(engine.element_state("scroller").unwrap().scroll_y, 20.0);
    assert!(output.scroll_chrome.iter().any(|chrome| {
        chrome.element_id == ElementId::new("scroller")
            && chrome.axis == ScrollAxis::Vertical
            && chrome.max_scroll > 0.0
    }));

    let output = engine.update(&mut document, &stylesheet);
    let first_row = output.layout.find("row-0").unwrap();
    assert_eq!(first_row.rect.origin.y, -20.0);
}

#[test]
fn declared_clip_cross_axis_normalizes_to_hidden_when_paired_with_scroll() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("clipper"),
            Style::default()
                .size(80.0, 60.0)
                .overflow_x(Overflow::Clip)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("side"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(90.0))
                .top(Length::Px(20.0))
                .size(24.0, 18.0),
        );
    let mut document = Document::build(Size::new(180.0, 180.0), |ui| {
        ui.element("clipper", ElementSpec::new(Element::Div), |ui| {
            ui.element("side", ElementSpec::new(Element::Div).interactive(), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let clipper = output.snapshot().find("clipper").unwrap();

    assert_eq!(clipper.style().overflow_x, Overflow::Hidden);
    assert_eq!(clipper.style().overflow_y, Overflow::Scroll);
    assert_ne!(
        output
            .snapshot()
            .hit_test(Point::new(100.0, 30.0))
            .unwrap()
            .target
            .id(),
        &ElementId::new("side")
    );
}

#[test]
fn overflow_hidden_clips_without_emitting_scroll_chrome() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("clipper"),
            Style::default().size(80.0, 60.0).overflow(Overflow::Hidden),
        )
        .rule(
            StyleSelector::id("child"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(90.0))
                .top(Length::Px(10.0))
                .size(24.0, 18.0),
        );
    let mut document = Document::build(Size::new(180.0, 180.0), |ui| {
        ui.element("clipper", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "child",
                ElementSpec::new(Element::Div).interactive(),
                |_| {},
            );
        });
    });

    let output = engine.update(&mut document, &stylesheet);

    assert!(output.scroll_chrome.is_empty());
    assert_ne!(
        output
            .snapshot()
            .hit_test(Point::new(100.0, 20.0))
            .unwrap()
            .target
            .id(),
        &ElementId::new("child")
    );
}

#[test]
fn overflow_clip_clips_without_emitting_scroll_chrome() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("clipper"),
            Style::default().size(80.0, 60.0).overflow(Overflow::Clip),
        )
        .rule(
            StyleSelector::id("child"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(90.0))
                .top(Length::Px(10.0))
                .size(24.0, 18.0),
        );
    let mut document = Document::build(Size::new(180.0, 180.0), |ui| {
        ui.element("clipper", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "child",
                ElementSpec::new(Element::Div).interactive(),
                |_| {},
            );
        });
    });

    let output = engine.update(&mut document, &stylesheet);

    assert!(output.scroll_chrome.is_empty());
    assert_ne!(
        output
            .snapshot()
            .hit_test(Point::new(100.0, 20.0))
            .unwrap()
            .target
            .id(),
        &ElementId::new("child")
    );
}

#[test]
fn two_axis_overflow_keeps_independent_scroll_state_and_chrome() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(70.0, 70.0)
                .overflow(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .scrollbar_pressed_handle_color(Color::rgba(190, 217, 255, 238))
                .scrollbar_pressed_handle_border_color(Color::rgba(255, 255, 255, 120))
                .scrollbar_pressed_handle_border_width(1.0),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(140.0, 140.0),
        );
    let mut document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("content", ElementSpec::new(Element::Div), |_| {});
        });
    });

    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(-16.0, -24.0),
            keys: Vec::new(),
        },
    );

    let state = engine.element_state("scroll-panel").unwrap();
    assert_eq!(state.scroll_x, 16.0);
    assert_eq!(state.scroll_y, 24.0);

    let output = engine.update(&mut document, &stylesheet);
    let content = output.layout.find("content").unwrap();
    assert_eq!(content.rect.origin, Point::new(-16.0, -24.0));
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Horizontal
        }),
        "two-axis overflow should emit horizontal chrome"
    );
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        }),
        "two-axis overflow should emit vertical chrome"
    );

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(64.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .unwrap();
    let horizontal = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Horizontal
        })
        .unwrap();
    assert!(vertical.expanded);
    assert_eq!(vertical.handle_rect.size.width, 10.0);
    assert!(!horizontal.expanded);
    assert_eq!(horizontal.handle_rect.size.height, 2.0);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(64.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .unwrap();
    let horizontal = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Horizontal
        })
        .unwrap();
    assert!(vertical.dragged);
    assert_eq!(vertical.handle_rect.size.width, 10.0);
    assert_eq!(vertical.handle_color.a, 238);
    assert!(vertical.handle_border_color.is_some());
    assert!(!horizontal.dragged);
    assert_eq!(horizontal.handle_rect.size.height, 2.0);
    assert_eq!(horizontal.handle_color.a, 118);
    assert!(horizontal.handle_border_color.is_none());
}

#[test]
fn overflow_scrollbar_can_be_forced_visible_without_hover() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(70.0, 70.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_visible(true),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(70.0, 140.0),
        );
    let mut document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("content", ElementSpec::new(Element::Div), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .expect("overflowing scroll panel should emit vertical scroll chrome");

    assert!(vertical.visible);
    assert!(!vertical.hovered);
    assert!(!vertical.dragged);
}

#[test]
fn scroll_limits_include_child_margin_overflow() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(100.0, 80.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::class("row"),
            Style::default().size(100.0, 40.0),
        )
        .rule(
            StyleSelector::id("tail"),
            Style::default().margin(Insets {
                top: 0.0,
                right: 0.0,
                bottom: 30.0,
                left: 0.0,
            }),
        );
    let mut document = Document::build(Size::new(140.0, 120.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("head", ElementSpec::new(Element::Div).class("row"), |_| {});
            ui.element("tail", ElementSpec::new(Element::Div).class("row"), |_| {});
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .expect("child margin overflow should emit scroll chrome");

    assert_eq!(vertical.max_scroll, 30.0);
}

#[test]
fn scrollbar_hover_transition_reuses_layout() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(70.0, 70.0)
                .overflow(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .transition(Transition::ease_out(0.25)),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(70.0, 140.0),
        );
    let mut document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            ui.element("content", ElementSpec::new(Element::Div), |_| {});
        });
    });

    engine.update(&mut document, &stylesheet);
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(64.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .unwrap();

    assert!(vertical.expanded);
    assert!(vertical.handle_rect.size.width > 2.0);
    assert!(vertical.handle_rect.size.width < 10.0);
    assert!(output.animating);
    assert!(output.metrics.reused_input_layout);
    assert!(!output.metrics.animation_changed_layout);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(64.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(output.metrics.reused_input_layout);
    assert!(!output.metrics.animation_changed_layout);
}

#[test]
fn nested_scroll_chrome_is_clipped_by_ancestor_scroll_viewport() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("horizontal-parent"),
            Style::default()
                .flex_direction(des_document::FlexDirection::Row)
                .size(120.0, 96.0)
                .gap(10.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("nested-list"),
            Style::default()
                .size(70.0, 74.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0),
        )
        .rule(
            StyleSelector::class("nested-row"),
            Style::default().size(54.0, 28.0),
        );
    let mut document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element("horizontal-parent", ElementSpec::new(Element::Div), |ui| {
            for list_index in 0..3 {
                ui.element(
                    format!("nested-list-{list_index}"),
                    ElementSpec::new(Element::Div).class("nested-list"),
                    |ui| {
                        for row_index in 0..5 {
                            ui.element(
                                format!("nested-list-{list_index}-row-{row_index}"),
                                ElementSpec::new(Element::Div).class("nested-row"),
                                |_| {},
                            );
                        }
                    },
                );
            }
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let visible_parent_right = output
        .layout
        .find("horizontal-parent")
        .unwrap()
        .rect
        .right();
    let nested_vertical_chrome: Vec<_> = output
        .scroll_chrome
        .iter()
        .filter(|chrome| {
            chrome.element_id.as_str().starts_with("nested-list-")
                && chrome.axis == ScrollAxis::Vertical
        })
        .collect();

    assert_eq!(
        nested_vertical_chrome.len(),
        1,
        "only the fully visible nested list should expose vertical chrome"
    );
    let chrome = nested_vertical_chrome[0];
    assert_eq!(chrome.element_id, ElementId::new("nested-list-0"));
    assert!(chrome.hit_rect.right() <= visible_parent_right);
    assert!(chrome.track_rect.right() <= visible_parent_right);
    assert!(chrome.handle_rect.right() <= visible_parent_right);
}

#[test]
fn clipped_scroll_chrome_does_not_drive_animation_work() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("horizontal-parent"),
            Style::default()
                .flex_direction(des_document::FlexDirection::Row)
                .size(120.0, 96.0)
                .gap(10.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .transition(Transition::ease_out(0.2)),
        )
        .rule(
            StyleSelector::class("nested-list"),
            Style::default()
                .size(70.0, 74.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .transition(Transition::ease_out(0.2)),
        )
        .rule(
            StyleSelector::class("nested-row"),
            Style::default().size(54.0, 28.0),
        );
    let mut document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element("horizontal-parent", ElementSpec::new(Element::Div), |ui| {
            for list_index in 0..3 {
                ui.element(
                    format!("nested-list-{list_index}"),
                    ElementSpec::new(Element::Div).class("nested-list"),
                    |ui| {
                        for row_index in 0..5 {
                            ui.element(
                                format!("nested-list-{list_index}-row-{row_index}"),
                                ElementSpec::new(Element::Div).class("nested-row"),
                                |_| {},
                            );
                        }
                    },
                );
            }
        });
    });

    engine.update(&mut document, &stylesheet);
    engine
        .element_state_mut("horizontal-parent")
        .unwrap()
        .scroll_x = 110.0;
    let output = engine.update(&mut document, &stylesheet);
    let nested_scrollbar = output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == ElementId::new("nested-list-2"))
        .expect("nested list should be visible after horizontal scroll");
    let pointer = Point::new(
        nested_scrollbar.hit_rect.origin.x + nested_scrollbar.hit_rect.size.width / 2.0,
        nested_scrollbar.hit_rect.origin.y + nested_scrollbar.hit_rect.size.height / 2.0,
    );
    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: pointer,
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    engine
        .element_state_mut("horizontal-parent")
        .unwrap()
        .scroll_x = 0.0;
    let output = engine.update(&mut document, &stylesheet);

    assert!(
        !output.animating,
        "offscreen nested scrollbars should not keep the document animating"
    );
}

#[test]
fn scroll_delta_is_clamped_when_content_does_not_overflow() {
    let mut engine = DocumentEngine::default();
    let stylesheet = scroll_fixture_stylesheet(120.0);
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            ui.element(
                "row-0",
                ElementSpec::new(Element::Div).class("scroll-row"),
                |_| {},
            );
            ui.element(
                "row-1",
                ElementSpec::new(Element::Div).class("scroll-row"),
                |_| {},
            );
        });
    });

    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -240.0),
            keys: Vec::new(),
        },
    );

    assert_eq!(engine.element_state("scroll-panel").unwrap().scroll_y, 0.0);
}

#[test]
fn overflow_scroll_container_emits_draggable_scroll_chrome() {
    let mut engine = DocumentEngine::default();
    let stylesheet = scroll_fixture_stylesheet(80.0);
    let mut document = overflowing_scroll_document();

    let output = engine.update(&mut document, &stylesheet);
    let chrome = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .expect("overflowing panel should emit scroll chrome");
    assert!(chrome.max_scroll > 0.0);
    assert!(chrome.handle_rect.size.height < chrome.track_rect.size.height);

    let grab = Point::new(
        chrome.handle_rect.origin.x + chrome.handle_rect.size.width / 2.0,
        chrome.handle_rect.origin.y + chrome.handle_rect.size.height / 2.0,
    );
    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: grab,
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(grab.x, grab.y + 24.0),
                primary_delta: Point::new(0.0, 24.0),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert!(engine.element_state("scroll-panel").unwrap().scroll_y > 0.0);
}

#[test]
fn active_document_drag_is_not_stolen_by_scrollbar_hitbox() {
    let mut engine = DocumentEngine::default();
    let stylesheet = scroll_fixture_stylesheet(80.0).rule(
        StyleSelector::id("drag-source"),
        Style::default().size(80.0, 32.0),
    );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element(
            "drag-source",
            ElementSpec::new(Element::Div).interactive(),
            |_| {},
        );
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            for index in 0..6 {
                ui.element(
                    format!("row-{index}"),
                    ElementSpec::new(Element::Div).class("scroll-row"),
                    |_| {},
                );
            }
        });
    });

    let output = engine.update(&mut document, &stylesheet);
    let chrome = output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
        .expect("overflowing panel should emit scroll chrome");
    let scrollbar_point = Point::new(
        chrome.hit_rect.origin.x + chrome.hit_rect.size.width / 2.0,
        chrome.hit_rect.origin.y + chrome.hit_rect.size.height / 2.0,
    );

    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 16.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: scrollbar_point,
                primary_delta: Point::new(scrollbar_point.x - 20.0, scrollbar_point.y - 16.0),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );

    assert_eq!(
        output.hit_id.as_ref().map(|id| id.as_str()),
        Some("drag-source")
    );
    assert!(
        output
            .active_drag
            .as_ref()
            .is_some_and(|drag| drag.target == ElementId::new("drag-source")),
        "document drags should continue even while the pointer crosses scrollbar hitboxes"
    );
}

#[test]
fn scroll_chrome_appears_on_container_hover_and_expands_on_hit_strip() {
    let mut engine = DocumentEngine::default();
    let stylesheet = scroll_fixture_stylesheet(80.0);
    let mut document = overflowing_scroll_document();

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let chrome = output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
        .unwrap();
    assert!(chrome.visible);
    assert!(!chrome.expanded);
    assert!(!chrome.hovered);
    assert_eq!(chrome.handle_rect.size.width, 2.0);
    assert!(chrome.track_color.is_some());
    assert_eq!(chrome.hit_rect.size.width, 12.0);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(170.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let chrome = output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
        .unwrap();
    assert!(chrome.visible);
    assert!(chrome.expanded);
    assert!(chrome.hovered);
    assert!(chrome.handle_rect.size.width > 2.0);
    assert!(chrome.handle_rect.size.width < 10.0);
    assert!(chrome.track_color.is_some());
    assert_eq!(chrome.handle_color.a, 118);
    assert!(chrome.handle_border_color.is_none());

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(170.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
            keys: Vec::new(),
        },
    );
    let chrome = output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
        .unwrap();
    assert!(chrome.dragged);
    assert!(chrome.handle_rect.size.width > 2.0);
    assert!(chrome.handle_rect.size.width < 10.0);
    assert_eq!(chrome.track_color, Some(Color::rgba(2, 8, 12, 84)));
    assert!(chrome.handle_color.a > 118);
    assert!(chrome.handle_border_color.is_some());
    assert!(chrome.handle_border_width > 0.0);
}

fn catalog_document(title_id: &str) -> Document {
    Document::build(Size::new(240.0, 480.0), |ui| {
        ui.element(
            "catalog",
            ElementSpec::new(Element::Div).class("catalog"),
            |ui| {
                ui.text(title_id, title_id);
                ui.element(
                    "project-card",
                    ElementSpec::new(Element::Div)
                        .class("catalog-item")
                        .class("selected"),
                    |ui| {
                        ui.text("project-name", "Customer 360");
                    },
                );
            },
        );
    })
}

fn probe_stylesheet() -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::class("catalog"),
            Style::default()
                .size(180.0, 40.0)
                .padding(Insets::all(12.0))
                .gap(8.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::class("catalog-item"),
            Style::default().size(180.0, 48.0),
        )
}

fn scroll_fixture_stylesheet(panel_height: f32) -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(180.0, panel_height)
                .padding(Insets::all(8.0))
                .gap(4.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .scrollbar_handle_color(Color::rgba(232, 236, 240, 118))
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_hover_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_pressed_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_pressed_handle_color(Color::rgba(190, 217, 255, 238))
                .scrollbar_pressed_handle_border_color(Color::rgba(255, 255, 255, 120))
                .scrollbar_pressed_handle_border_width(1.0)
                .scrollbar_radius(6.0)
                .transition(Transition::ease_out(0.2)),
        )
        .rule(
            StyleSelector::class("scroll-row"),
            Style::default().size(140.0, 32.0),
        )
}

fn overflowing_scroll_document() -> Document {
    Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(Element::Div), |ui| {
            for index in 0..6 {
                ui.element(
                    format!("row-{index}"),
                    ElementSpec::new(Element::Div).class("scroll-row"),
                    |_| {},
                );
            }
        });
    })
}

fn table_fixture_document() -> Document {
    let table = TableSpec::new(vec![
        TableColumnSpec::new("customer", "Customer").width(TableTrackSize::px(120.0)),
        TableColumnSpec::new("country", "Country").width(TableTrackSize::px(100.0)),
        TableColumnSpec::new("orders", "Orders").width(TableTrackSize::px(80.0)),
    ])
    .header_height(28.0)
    .row_height(26.0);

    Document::build(Size::new(320.0, 220.0), |ui| {
        ui.element(
            "customers",
            ElementSpec::new(Element::Table).table(table),
            |ui| {
                ui.element("customers-header", ElementSpec::new(Element::Thead), |ui| {
                    table_cell(ui, "customers-header-customer", "customer", "Customer");
                    table_cell(ui, "customers-header-country", "country", "Country");
                    table_cell(ui, "customers-header-orders", "orders", "Orders");
                });
                ui.element("customers-row-0", ElementSpec::new(Element::Tr), |ui| {
                    table_cell(ui, "customers-row-0-customer", "customer", "Acme");
                    table_cell(ui, "customers-row-0-country", "country", "US");
                    table_cell(ui, "customers-row-0-orders", "orders", "42");
                });
            },
        );
    })
}

fn table_cell(
    ui: &mut des_document::DocumentBuilder,
    id: &'static str,
    column_id: &'static str,
    text: &'static str,
) {
    ui.text_element(
        id,
        ElementSpec::new(Element::Td).table_cell(TableCellSpec::new(column_id)),
        text,
    );
}
