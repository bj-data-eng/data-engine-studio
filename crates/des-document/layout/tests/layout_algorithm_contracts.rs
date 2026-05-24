use des_layout::prelude::{
    auto, length, Display, FlexDirection, LayoutMaxContent, LayoutTree, Size as LayoutSize, Style,
};

#[test]
fn flex_row_distributes_remaining_width_to_growing_child() {
    let mut tree: LayoutTree<()> = LayoutTree::new();
    let fixed = tree
        .new_leaf(Style {
            size: LayoutSize {
                width: length(80.0),
                height: auto(),
            },
            ..Default::default()
        })
        .unwrap();
    let growing = tree
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
                size: LayoutSize {
                    width: length(300.0),
                    height: length(60.0),
                },
                ..Default::default()
            },
            &[fixed, growing],
        )
        .unwrap();

    tree.compute_layout(root, LayoutSize::MAX_CONTENT).unwrap();

    assert_eq!(tree.layout(fixed).unwrap().size.width, 80.0);
    assert_eq!(tree.layout(growing).unwrap().location.x, 80.0);
    assert_eq!(tree.layout(growing).unwrap().size.width, 220.0);
    assert_eq!(tree.layout(growing).unwrap().size.height, 60.0);
}

#[test]
fn flex_row_reverse_positions_children_from_right_edge() {
    let mut tree: LayoutTree<()> = LayoutTree::new();
    let first = tree
        .new_leaf(Style {
            size: LayoutSize {
                width: length(40.0),
                height: length(20.0),
            },
            ..Default::default()
        })
        .unwrap();
    let second = tree
        .new_leaf(Style {
            size: LayoutSize {
                width: length(60.0),
                height: length(20.0),
            },
            ..Default::default()
        })
        .unwrap();
    let root = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::RowReverse,
                size: LayoutSize {
                    width: length(200.0),
                    height: length(40.0),
                },
                ..Default::default()
            },
            &[first, second],
        )
        .unwrap();

    tree.compute_layout(root, LayoutSize::MAX_CONTENT).unwrap();

    assert_eq!(tree.layout(first).unwrap().location.x, 160.0);
    assert_eq!(tree.layout(second).unwrap().location.x, 100.0);
}

#[test]
fn block_layout_stacks_children_and_derives_auto_height() {
    let mut tree: LayoutTree<()> = LayoutTree::new();
    let first = tree
        .new_leaf(Style {
            size: LayoutSize {
                width: length(100.0),
                height: length(20.0),
            },
            ..Default::default()
        })
        .unwrap();
    let second = tree
        .new_leaf(Style {
            size: LayoutSize {
                width: length(100.0),
                height: length(30.0),
            },
            ..Default::default()
        })
        .unwrap();
    let root = tree
        .new_with_children(
            Style {
                display: Display::Block,
                size: LayoutSize {
                    width: length(100.0),
                    height: auto(),
                },
                ..Default::default()
            },
            &[first, second],
        )
        .unwrap();

    tree.compute_layout(root, LayoutSize::MAX_CONTENT).unwrap();

    assert_eq!(tree.layout(first).unwrap().location.y, 0.0);
    assert_eq!(tree.layout(second).unwrap().location.y, 20.0);
    assert_eq!(tree.layout(root).unwrap().size.height, 50.0);
}

#[test]
fn grid_auto_places_children_into_declared_column_tracks() {
    let mut tree: LayoutTree<()> = LayoutTree::new();
    let first = tree.new_leaf(Style::default()).unwrap();
    let second = tree.new_leaf(Style::default()).unwrap();
    let root = tree
        .new_with_children(
            Style {
                display: Display::Grid,
                size: LayoutSize {
                    width: length(200.0),
                    height: length(40.0),
                },
                grid_template_columns: vec![length(80.0), length(120.0)],
                grid_template_rows: vec![length(40.0)],
                ..Default::default()
            },
            &[first, second],
        )
        .unwrap();

    tree.compute_layout(root, LayoutSize::MAX_CONTENT).unwrap();

    assert_eq!(tree.layout(first).unwrap().location.x, 0.0);
    assert_eq!(tree.layout(first).unwrap().size.width, 80.0);
    assert_eq!(tree.layout(second).unwrap().location.x, 80.0);
    assert_eq!(tree.layout(second).unwrap().size.width, 120.0);
}
