use des_document::{DocumentEngine, DocumentOutput, Point};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortableItemId(pub usize);

impl SortableItemId {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DropZoneId(pub usize);

impl DropZoneId {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DropEdge {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortableDropPreview {
    pub zone: DropZoneId,
    pub nearest_item: Option<SortableItemId>,
    pub edge: DropEdge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortableDocumentConfig {
    item_class: String,
    zone_class: String,
    item_id_prefix: String,
    handle_id_prefix: String,
    zone_id_prefix: String,
    item_count: usize,
    zone_count: usize,
}

impl SortableDocumentConfig {
    pub fn new(
        item_class: impl Into<String>,
        zone_class: impl Into<String>,
        item_id_prefix: impl Into<String>,
        handle_id_prefix: impl Into<String>,
        zone_id_prefix: impl Into<String>,
        item_count: usize,
        zone_count: usize,
    ) -> Self {
        Self {
            item_class: item_class.into(),
            zone_class: zone_class.into(),
            item_id_prefix: item_id_prefix.into(),
            handle_id_prefix: handle_id_prefix.into(),
            zone_id_prefix: zone_id_prefix.into(),
            item_count,
            zone_count,
        }
    }

    pub fn item_for_element_id(&self, id: &str) -> Option<SortableItemId> {
        id.strip_prefix(&self.item_id_prefix)
            .or_else(|| id.strip_prefix(&self.handle_id_prefix))
            .and_then(|suffix| suffix.parse::<usize>().ok())
            .filter(|index| *index < self.item_count)
            .map(SortableItemId)
    }

    pub fn item_element_id(&self, item: SortableItemId) -> String {
        format!("{}{}", self.item_id_prefix, item.index())
    }

    pub fn handle_element_id(&self, item: SortableItemId) -> String {
        format!("{}{}", self.handle_id_prefix, item.index())
    }

    pub fn zone_for_element_id(&self, id: &str) -> Option<DropZoneId> {
        id.strip_prefix(&self.zone_id_prefix)
            .and_then(|suffix| suffix.parse::<usize>().ok())
            .filter(|index| *index < self.zone_count)
            .map(DropZoneId)
    }

    pub fn zone_element_id(&self, zone: DropZoneId) -> String {
        format!("{}{}", self.zone_id_prefix, zone.index())
    }

    pub fn drop_zone_at(&self, output: &DocumentOutput, point: Point) -> Option<DropZoneId> {
        output
            .snapshot()
            .elements_with_class(self.zone_class.as_str())
            .into_iter()
            .filter(|element| element.rect().contains(point))
            .find_map(|element| self.zone_for_element_id(element.id().as_str()))
    }

    pub fn item_zone(&self, output: &DocumentOutput, item: SortableItemId) -> Option<DropZoneId> {
        let point = output
            .snapshot()
            .find(format!("{}{}", self.item_id_prefix, item.0).as_str())?
            .rect()
            .origin;
        output
            .snapshot()
            .elements_with_class(self.zone_class.as_str())
            .into_iter()
            .filter(|element| element.rect().contains(point))
            .find_map(|element| self.zone_for_element_id(element.id().as_str()))
    }

    pub fn preview_at(
        &self,
        output: &DocumentOutput,
        point: Point,
        active_item: Option<SortableItemId>,
    ) -> Option<SortableDropPreview> {
        let zone = self.drop_zone_at(output, point)?;
        let nearest = output
            .snapshot()
            .elements_with_class(self.item_class.as_str())
            .into_iter()
            .filter_map(|element| {
                let item = self.item_for_element_id(element.id().as_str())?;
                if active_item == Some(item) {
                    return None;
                }
                if self.item_zone(output, item) != Some(zone) {
                    return None;
                }
                let rect = element.rect();
                let center_y = rect.origin.y + rect.size.height / 2.0;
                Some((item, center_y, (point.y - center_y).abs()))
            })
            .min_by(|left, right| left.2.total_cmp(&right.2));

        let (nearest_item, edge) = nearest
            .map(|(item, center_y, _)| {
                (
                    Some(item),
                    if point.y > center_y {
                        DropEdge::After
                    } else {
                        DropEdge::Before
                    },
                )
            })
            .unwrap_or((None, DropEdge::After));

        Some(SortableDropPreview {
            zone,
            nearest_item,
            edge,
        })
    }

    pub fn snap_drop_animation(
        &self,
        engine: &mut DocumentEngine,
        item: SortableItemId,
        preview: Option<SortableDropPreview>,
    ) {
        engine.snap_element_animation(format!("{}{}", self.item_id_prefix, item.0).as_str());
        if let Some(nearest_item) = preview.and_then(|preview| preview.nearest_item) {
            engine.snap_element_animation(
                format!("{}{}", self.item_id_prefix, nearest_item.0).as_str(),
            );
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortableModel {
    item_zones: Vec<DropZoneId>,
    item_order: Vec<usize>,
}

impl SortableModel {
    pub fn new(item_zones: impl Into<Vec<DropZoneId>>, item_order: impl Into<Vec<usize>>) -> Self {
        let item_zones = item_zones.into();
        let item_order = item_order.into();
        assert_eq!(
            item_zones.len(),
            item_order.len(),
            "sortable item zones and order must have matching lengths"
        );
        Self {
            item_zones,
            item_order,
        }
    }

    pub fn item_zone(&self, item: SortableItemId) -> DropZoneId {
        self.item_zones[item.0]
    }

    pub fn item_order(&self, item: SortableItemId) -> usize {
        self.item_order[item.0]
    }

    pub fn item_zones(&self) -> &[DropZoneId] {
        &self.item_zones
    }

    pub fn item_order_values(&self) -> &[usize] {
        &self.item_order
    }

    pub fn set_item_zone(&mut self, item: SortableItemId, zone: DropZoneId) {
        self.item_zones[item.0] = zone;
    }

    pub fn preview_changes_position(
        &self,
        item: SortableItemId,
        preview: SortableDropPreview,
    ) -> bool {
        if self.item_zones[item.0] != preview.zone {
            return true;
        }

        let mut zone_items: Vec<_> = (0..self.item_zones.len())
            .filter(|candidate| self.item_zones[*candidate] == preview.zone)
            .map(SortableItemId)
            .collect();
        zone_items.sort_by_key(|candidate| self.item_order[candidate.0]);

        let Some(current_index) = zone_items.iter().position(|candidate| *candidate == item) else {
            return true;
        };

        let mut target_items = zone_items;
        target_items.retain(|candidate| *candidate != item);
        let target_index = self
            .preview_insert_index(&target_items, preview)
            .min(target_items.len());

        target_index != current_index
    }

    pub fn apply_drop(&mut self, item: SortableItemId, preview: SortableDropPreview) {
        self.item_zones[item.0] = preview.zone;

        let mut ordered_items: Vec<_> = (0..self.item_zones.len()).map(SortableItemId).collect();
        ordered_items.sort_by_key(|candidate| self.item_order[candidate.0]);
        ordered_items.retain(|candidate| *candidate != item);

        let insert_index = self.preview_insert_index(&ordered_items, preview);
        ordered_items.insert(insert_index.min(ordered_items.len()), item);

        for (order, ordered_item) in ordered_items.into_iter().enumerate() {
            self.item_order[ordered_item.0] = order;
        }
    }

    fn preview_insert_index(
        &self,
        ordered_items: &[SortableItemId],
        preview: SortableDropPreview,
    ) -> usize {
        preview
            .nearest_item
            .and_then(|nearest| {
                ordered_items
                    .iter()
                    .position(|candidate| *candidate == nearest)
                    .map(|index| {
                        if preview.edge == DropEdge::After {
                            index + 1
                        } else {
                            index
                        }
                    })
            })
            .unwrap_or(ordered_items.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sortable_model_moves_items_between_zones() {
        let mut model =
            SortableModel::new([DropZoneId(0), DropZoneId(2), DropZoneId(4)], [0, 1, 2]);

        model.apply_drop(
            SortableItemId(0),
            SortableDropPreview {
                zone: DropZoneId(3),
                nearest_item: None,
                edge: DropEdge::After,
            },
        );

        assert_eq!(model.item_zone(SortableItemId(0)), DropZoneId(3));
        assert_eq!(model.item_order(SortableItemId(0)), 2);
    }

    #[test]
    fn sortable_model_reorders_near_existing_item() {
        let mut model =
            SortableModel::new([DropZoneId(0), DropZoneId(0), DropZoneId(2)], [0, 1, 2]);
        let preview = SortableDropPreview {
            zone: DropZoneId(0),
            nearest_item: Some(SortableItemId(1)),
            edge: DropEdge::After,
        };

        assert!(model.preview_changes_position(SortableItemId(0), preview));
        model.apply_drop(SortableItemId(0), preview);

        assert!(model.item_order(SortableItemId(0)) > model.item_order(SortableItemId(1)));
    }

    #[test]
    fn sortable_model_ignores_preview_at_original_position() {
        let model = SortableModel::new([DropZoneId(0), DropZoneId(0), DropZoneId(2)], [0, 1, 2]);
        let preview = SortableDropPreview {
            zone: DropZoneId(0),
            nearest_item: Some(SortableItemId(0)),
            edge: DropEdge::After,
        };

        assert!(!model.preview_changes_position(SortableItemId(1), preview));
    }

    #[test]
    fn document_config_maps_item_and_zone_ids() {
        let config = SortableDocumentConfig::new("item", "zone", "item-", "handle-", "zone-", 3, 6);
        let item = SortableItemId::new(2);
        let handle = SortableItemId::new(1);
        let zone = DropZoneId::new(5);

        assert_eq!(item.index(), 2);
        assert_eq!(handle.index(), 1);
        assert_eq!(zone.index(), 5);
        assert_eq!(config.item_element_id(item), "item-2");
        assert_eq!(config.handle_element_id(handle), "handle-1");
        assert_eq!(config.zone_element_id(zone), "zone-5");
        assert_eq!(config.item_for_element_id("item-2"), Some(item));
        assert_eq!(config.item_for_element_id("handle-1"), Some(handle));
        assert_eq!(config.item_for_element_id("item-3"), None);
        assert_eq!(config.zone_for_element_id("zone-5"), Some(zone));
        assert_eq!(config.zone_for_element_id("zone-6"), None);
    }
}
