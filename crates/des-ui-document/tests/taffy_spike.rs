use taffy::prelude::{
    AvailableSpace, Display, FlexDirection, Size as TaffySize, Style, TaffyMaxContent, TaffyTree,
    auto, length, percent,
};

#[derive(Debug)]
struct TextProbe {
    width: f32,
    height: f32,
}

#[test]
fn taffy_can_layout_a_viewport_sized_flex_shell() {
    let mut tree: TaffyTree<()> = TaffyTree::new();

    let nav = tree
        .new_leaf(Style {
            size: TaffySize {
                width: length(240.0),
                height: auto(),
            },
            ..Default::default()
        })
        .unwrap();
    let stage = tree
        .new_leaf(Style {
            flex_grow: 1.0,
            ..Default::default()
        })
        .unwrap();
    let root = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                size: TaffySize {
                    width: length(1200.0),
                    height: length(800.0),
                },
                ..Default::default()
            },
            &[nav, stage],
        )
        .unwrap();

    tree.compute_layout(root, TaffySize::MAX_CONTENT).unwrap();

    let root_layout = tree.layout(root).unwrap();
    let nav_layout = tree.layout(nav).unwrap();
    let stage_layout = tree.layout(stage).unwrap();

    assert_eq!(root_layout.size.width, 1200.0);
    assert_eq!(root_layout.size.height, 800.0);
    assert_eq!(nav_layout.size.width, 240.0);
    assert_eq!(stage_layout.location.x, 240.0);
    assert_eq!(stage_layout.size.width, 960.0);
    assert_eq!(stage_layout.size.height, 800.0);
}

#[test]
fn taffy_can_measure_text_with_available_width() {
    let mut tree: TaffyTree<TextProbe> = TaffyTree::new();

    let text = tree
        .new_leaf_with_context(
            Style {
                size: TaffySize {
                    width: percent(1.0),
                    height: auto(),
                },
                ..Default::default()
            },
            TextProbe {
                width: 180.0,
                height: 32.0,
            },
        )
        .unwrap();
    let root = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                size: TaffySize {
                    width: length(120.0),
                    height: auto(),
                },
                ..Default::default()
            },
            &[text],
        )
        .unwrap();

    tree.compute_layout_with_measure(
        root,
        TaffySize::MAX_CONTENT,
        |known_dimensions, available_space, _node_id, context, _style| {
            let context = context.expect("text probe context is present");
            let available_width = match available_space.width {
                AvailableSpace::Definite(width) => width,
                AvailableSpace::MinContent | AvailableSpace::MaxContent => context.width,
            };
            TaffySize {
                width: known_dimensions
                    .width
                    .unwrap_or(context.width.min(available_width)),
                height: known_dimensions.height.unwrap_or(context.height),
            }
        },
    )
    .unwrap();

    let text_layout = tree.layout(text).unwrap();
    assert_eq!(text_layout.size.width, 120.0);
    assert_eq!(text_layout.size.height, 32.0);
}
