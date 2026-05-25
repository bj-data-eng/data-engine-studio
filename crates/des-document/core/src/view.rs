use crate::{
    Document, DocumentBuilder, DocumentEngine, DocumentInput, DocumentOutput, DocumentProjection,
    DocumentProjectionReport, DocumentResult, DocumentWidget, Size, StyleSheet, TextMeasurer,
};

/// A ready-to-drive retained document surface.
///
/// `DocumentView` groups the three objects app code normally has to keep in
/// sync by hand: the retained document tree, its stylesheet, and the engine
/// state that routes input and produces resolved output.
pub struct DocumentView {
    document: Document,
    stylesheet: StyleSheet,
    engine: DocumentEngine,
}

impl DocumentView {
    /// Creates a document view from already-built document inputs.
    pub fn new(document: Document, stylesheet: StyleSheet) -> Self {
        Self {
            document,
            stylesheet,
            engine: DocumentEngine::default(),
        }
    }

    /// Builds a document view from the fluent Rust document builder.
    pub fn build(
        viewport: Size,
        stylesheet: StyleSheet,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> Self {
        Self::new(Document::build(viewport, build), stylesheet)
    }

    /// Builds a document view around one reusable document widget and collects
    /// that widget's stylesheet contribution.
    pub fn build_widget(
        viewport: Size,
        mut stylesheet: StyleSheet,
        widget: &impl DocumentWidget,
    ) -> Self {
        widget.push_styles(&mut stylesheet);
        Self::new(
            Document::build(viewport, |ui| ui.widget(widget)),
            stylesheet,
        )
    }

    /// Starts a composable document view builder for collecting structure,
    /// stylesheet rules, and widget style contributions through one front door.
    pub fn compose(viewport: Size) -> DocumentViewBuilder {
        DocumentViewBuilder::new(viewport)
    }

    /// Returns the retained document.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Returns the retained document for controlled mutation.
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }

    /// Replaces the retained document while keeping engine UI state.
    pub fn replace_document(&mut self, document: Document) {
        self.document = document;
    }

    /// Applies a batch of app-state projections to the retained document.
    pub fn project(
        &mut self,
        projection: &DocumentProjection,
    ) -> DocumentResult<DocumentProjectionReport> {
        projection.apply_to(&mut self.document)
    }

    /// Builds and applies a projection in one call.
    pub fn project_with(
        &mut self,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<DocumentProjectionReport> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.project(&projection)
    }

    /// Applies a projection and resolves the updated document.
    pub fn project_and_update(
        &mut self,
        projection: &DocumentProjection,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        let report = self.project(projection)?;
        let output = self.update();
        Ok((report, output))
    }

    /// Builds a projection, applies it, and resolves the updated document.
    pub fn project_with_and_update(
        &mut self,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.project_and_update(&projection)
    }

    /// Returns the stylesheet used to resolve this document.
    pub fn stylesheet(&self) -> &StyleSheet {
        &self.stylesheet
    }

    /// Returns the stylesheet for controlled mutation.
    pub fn stylesheet_mut(&mut self) -> &mut StyleSheet {
        &mut self.stylesheet
    }

    /// Replaces the stylesheet used to resolve this document.
    pub fn replace_stylesheet(&mut self, stylesheet: StyleSheet) {
        self.stylesheet = stylesheet;
    }

    /// Adds styles declared by a reusable document widget.
    pub fn push_widget_styles(&mut self, widget: &impl DocumentWidget) {
        widget.push_styles(&mut self.stylesheet);
    }

    /// Returns the retained document engine.
    pub fn engine(&self) -> &DocumentEngine {
        &self.engine
    }

    /// Returns the retained document engine for advanced state access.
    pub fn engine_mut(&mut self) -> &mut DocumentEngine {
        &mut self.engine
    }

    /// Resolves the document using the current stylesheet and no new input.
    pub fn update(&mut self) -> DocumentOutput {
        self.engine.update(&mut self.document, &self.stylesheet)
    }

    /// Routes input, resolves style/layout, and returns the current document output.
    pub fn update_with_input(&mut self, input: DocumentInput) -> DocumentOutput {
        self.engine
            .update_with_input(&mut self.document, &self.stylesheet, input)
    }

    /// Routes input and resolves the document with a host-provided text measurer.
    pub fn update_with_input_and_text_measurer(
        &mut self,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentOutput {
        self.engine.update_with_input_and_text_measurer(
            &mut self.document,
            &self.stylesheet,
            input,
            text_measurer,
        )
    }

    /// Splits the view into its owned document, stylesheet, and engine.
    pub fn into_parts(self) -> (Document, StyleSheet, DocumentEngine) {
        (self.document, self.stylesheet, self.engine)
    }
}

/// Fluent builder for composing a retained document view.
///
/// This keeps the app-facing setup path compact when a surface is assembled
/// from Rust-authored document structure, parsed CSS, and reusable widget
/// style contributions.
pub struct DocumentViewBuilder {
    viewport: Size,
    stylesheet: StyleSheet,
}

impl DocumentViewBuilder {
    pub fn new(viewport: Size) -> Self {
        Self {
            viewport,
            stylesheet: StyleSheet::new(),
        }
    }

    pub fn stylesheet(mut self, stylesheet: StyleSheet) -> Self {
        self.stylesheet = stylesheet;
        self
    }

    pub fn extend_stylesheet(mut self, stylesheet: StyleSheet) -> Self {
        self.stylesheet.extend(stylesheet);
        self
    }

    pub fn css(mut self, css: &str) -> Result<Self, crate::CssParseError> {
        self.stylesheet.extend_css(css)?;
        Ok(self)
    }

    pub fn with_css(self, css: &str) -> Result<Self, crate::CssParseError> {
        self.css(css)
    }

    pub fn css_forgiving(mut self, css: &str) -> Result<Self, crate::CssParseError> {
        self.stylesheet.extend_css_forgiving(css)?;
        Ok(self)
    }

    pub fn with_css_forgiving(self, css: &str) -> Result<Self, crate::CssParseError> {
        self.css_forgiving(css)
    }

    pub fn widget_styles(mut self, widget: &impl DocumentWidget) -> Self {
        widget.push_styles(&mut self.stylesheet);
        self
    }

    pub fn widget(mut self, widget: &impl DocumentWidget) -> DocumentView {
        widget.push_styles(&mut self.stylesheet);
        DocumentView::new(
            Document::build(self.viewport, |ui| ui.widget(widget)),
            self.stylesheet,
        )
    }

    pub fn build(self, build: impl FnOnce(&mut DocumentBuilder)) -> DocumentView {
        DocumentView::new(Document::build(self.viewport, build), self.stylesheet)
    }
}
