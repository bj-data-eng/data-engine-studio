#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TableColumnId(String);

impl TableColumnId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TableColumnId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for TableColumnId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableTrackSize {
    Px(f32),
    Flex(f32),
}

impl TableTrackSize {
    pub fn px(width: f32) -> Self {
        Self::Px(width.max(0.0))
    }

    pub fn flex(weight: f32) -> Self {
        Self::Flex(weight.max(0.0))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableColumnSpec {
    pub id: TableColumnId,
    pub title: String,
    pub width: TableTrackSize,
    pub min_width: f32,
    pub max_width: Option<f32>,
}

impl TableColumnSpec {
    pub fn new(id: impl Into<TableColumnId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            width: TableTrackSize::Px(120.0),
            min_width: 40.0,
            max_width: None,
        }
    }

    pub fn width(mut self, width: TableTrackSize) -> Self {
        self.width = width;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width.max(0.0);
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width.max(self.min_width));
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableSpec {
    pub columns: Vec<TableColumnSpec>,
    pub header_height: f32,
    pub row_height: f32,
}

impl TableSpec {
    pub fn new(columns: Vec<TableColumnSpec>) -> Self {
        Self {
            columns,
            header_height: 32.0,
            row_height: 30.0,
        }
    }

    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height.max(0.0);
        self
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(0.0);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableCellSpec {
    pub column_id: TableColumnId,
}

impl TableCellSpec {
    pub fn new(column_id: impl Into<TableColumnId>) -> Self {
        Self {
            column_id: column_id.into(),
        }
    }
}
