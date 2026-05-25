use crate::{
    Document, DocumentBuilder, DocumentEngine, DocumentInput, DocumentOutput, Size, StyleSheet,
    TextMeasurer,
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
