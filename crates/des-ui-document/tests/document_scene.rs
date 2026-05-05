use des_ui_document::{DocumentScene, ElementId, ElementRole, ElementSpec, Size};

#[test]
fn scene_reparents_existing_element_without_reallocating_layout_node() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_text(
            "panel",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Retained text",
        )
        .unwrap();

    let original_layout_node = scene.layout_node("label").unwrap();

    scene.reparent("label", "root").unwrap();

    assert_eq!(scene.layout_node("label"), Some(original_layout_node));
    assert_eq!(scene.parent("label").unwrap(), Some(ElementId::new("root")));
    assert!(scene.children("panel").unwrap().is_empty());
    assert_eq!(
        scene.children("root").unwrap(),
        vec![ElementId::new("panel"), ElementId::new("label")]
    );
}

#[test]
fn scene_remove_prunes_descendants_from_model_and_layout_graph() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_text(
            "panel",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Retained text",
        )
        .unwrap();

    scene.remove("panel").unwrap();

    assert_eq!(scene.children("root").unwrap(), Vec::<ElementId>::new());
    assert!(scene.layout_node("panel").is_none());
    assert!(scene.layout_node("label").is_none());
    assert!(scene.parent("label").is_err());
}
