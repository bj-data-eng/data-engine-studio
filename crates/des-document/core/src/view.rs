use crate::{
    Document, DocumentBuilder, DocumentCommandAction, DocumentCommandRegistry, DocumentEngine,
    DocumentEventKind, DocumentInput, DocumentOutput, DocumentProjection, DocumentProjectionReport,
    DocumentResult, DocumentWidget, Size, StyleSheet, TextMeasurer,
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

/// Resolved document output plus typed app actions collected from authored commands.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentActionFrame<Action> {
    pub output: DocumentOutput,
    pub actions: Vec<DocumentCommandAction<Action>>,
}

impl<Action> DocumentActionFrame<Action> {
    /// Returns the resolved document output for rendering and interaction queries.
    pub fn output(&self) -> &DocumentOutput {
        &self.output
    }

    /// Returns the collected typed app actions in document event order.
    pub fn actions(&self) -> &[DocumentCommandAction<Action>] {
        &self.actions
    }

    /// Returns true when this frame collected no typed app actions.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Returns the number of typed app actions collected for this frame.
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Returns the first typed app action, when one was collected.
    pub fn first_action(&self) -> Option<&DocumentCommandAction<Action>> {
        self.actions.first()
    }

    /// Iterates typed app actions emitted by one element.
    pub fn actions_for<'a>(
        &'a self,
        target: &'a str,
    ) -> impl Iterator<Item = &'a DocumentCommandAction<Action>> + 'a {
        self.actions
            .iter()
            .filter(move |action| action.target.as_str() == target)
    }

    /// Iterates typed app actions emitted by one resolved document event kind.
    pub fn actions_of_kind(
        &self,
        kind: DocumentEventKind,
    ) -> impl Iterator<Item = &DocumentCommandAction<Action>> {
        self.actions
            .iter()
            .filter(move |action| action.event == kind)
    }

    /// Iterates typed app actions emitted by click intent.
    pub fn clicked_actions(&self) -> impl Iterator<Item = &DocumentCommandAction<Action>> {
        self.actions_of_kind(DocumentEventKind::Clicked)
    }

    /// Returns true when the frame contains the supplied typed action.
    pub fn contains_action(&self, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.actions
            .iter()
            .any(|candidate| &candidate.action == action)
    }

    /// Consumes the frame into the resolved output and collected app actions.
    pub fn into_parts(self) -> (DocumentOutput, Vec<DocumentCommandAction<Action>>) {
        (self.output, self.actions)
    }

    /// Consumes the frame and returns only the collected app actions.
    pub fn into_actions(self) -> Vec<DocumentCommandAction<Action>> {
        self.actions
    }
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
        stylesheet: StyleSheet,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> Self {
        Self::try_build_widget(viewport, stylesheet, widget)
            .expect("document widget projection targets rendered elements")
    }

    /// Builds a document view around one reusable widget and returns projection
    /// errors instead of panicking when the widget contract is incomplete.
    pub fn try_build_widget(
        viewport: Size,
        mut stylesheet: StyleSheet,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> DocumentResult<Self> {
        widget.push_styles(&mut stylesheet);
        let mut view = Self::new(
            Document::build(viewport, |ui| ui.widget(widget)),
            stylesheet,
        );
        view.project_widget(widget)?;
        Ok(view)
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

    /// Applies a projection, routes input, and resolves the updated document.
    pub fn project_and_update_with_input(
        &mut self,
        projection: &DocumentProjection,
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        let report = self.project(projection)?;
        let output = self.update_with_input(input);
        Ok((report, output))
    }

    /// Builds a projection, applies it, routes input, and resolves the document.
    pub fn project_with_and_update_with_input(
        &mut self,
        input: DocumentInput,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.project_and_update_with_input(&projection, input)
    }

    /// Applies app-state projections declared by a reusable document widget.
    pub fn project_widget(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> DocumentResult<DocumentProjectionReport> {
        self.project_with(|projection| widget.push_projection(projection))
    }

    /// Applies app-state projections declared by a collection of widgets.
    pub fn project_widgets<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
    ) -> DocumentResult<DocumentProjectionReport>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.project_with(|projection| {
            for widget in widgets {
                widget.push_projection(projection);
            }
        })
    }

    /// Applies a widget projection and resolves the updated document.
    pub fn project_widget_and_update(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        let report = self.project_widget(widget)?;
        let output = self.update();
        Ok((report, output))
    }

    /// Applies widget projections and resolves the updated document.
    pub fn project_widgets_and_update<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        let report = self.project_widgets(widgets)?;
        let output = self.update();
        Ok((report, output))
    }

    /// Applies a widget projection, routes input, and resolves the document.
    pub fn project_widget_and_update_with_input(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        let report = self.project_widget(widget)?;
        let output = self.update_with_input(input);
        Ok((report, output))
    }

    /// Applies widget projections, routes input, and resolves the document.
    pub fn project_widgets_and_update_with_input<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        let report = self.project_widgets(widgets)?;
        let output = self.update_with_input(input);
        Ok((report, output))
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
    pub fn push_widget_styles(&mut self, widget: &(impl DocumentWidget + ?Sized)) {
        widget.push_styles(&mut self.stylesheet);
    }

    /// Adds styles declared by a collection of reusable document widgets.
    pub fn push_widget_styles_many<'a, W>(&mut self, widgets: impl IntoIterator<Item = &'a W>)
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        for widget in widgets {
            self.push_widget_styles(widget);
        }
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

    /// Resolves the document and collects typed app actions from authored commands.
    pub fn update_actions<Action>(
        &mut self,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentActionFrame<Action>
    where
        Action: Clone,
    {
        let output = self.update();
        Self::collect_action_frame(registry, output)
    }

    /// Routes input, resolves style/layout, and returns the current document output.
    pub fn update_with_input(&mut self, input: DocumentInput) -> DocumentOutput {
        self.engine
            .update_with_input(&mut self.document, &self.stylesheet, input)
    }

    /// Routes input, resolves the document, and collects typed app actions.
    pub fn update_with_input_actions<Action>(
        &mut self,
        input: DocumentInput,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentActionFrame<Action>
    where
        Action: Clone,
    {
        let output = self.update_with_input(input);
        Self::collect_action_frame(registry, output)
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

    /// Routes input with a host text measurer and collects typed app actions.
    pub fn update_with_input_and_text_measurer_actions<Action>(
        &mut self,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentActionFrame<Action>
    where
        Action: Clone,
    {
        let output = self.update_with_input_and_text_measurer(input, text_measurer);
        Self::collect_action_frame(registry, output)
    }

    /// Splits the view into its owned document, stylesheet, and engine.
    pub fn into_parts(self) -> (Document, StyleSheet, DocumentEngine) {
        (self.document, self.stylesheet, self.engine)
    }

    fn collect_action_frame<Action>(
        registry: &DocumentCommandRegistry<Action>,
        output: DocumentOutput,
    ) -> DocumentActionFrame<Action>
    where
        Action: Clone,
    {
        let actions = registry.collect_actions(&output);
        DocumentActionFrame { output, actions }
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

    pub fn widget_styles(mut self, widget: &(impl DocumentWidget + ?Sized)) -> Self {
        widget.push_styles(&mut self.stylesheet);
        self
    }

    pub fn widget_styles_many<'a, W>(mut self, widgets: impl IntoIterator<Item = &'a W>) -> Self
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        for widget in widgets {
            widget.push_styles(&mut self.stylesheet);
        }
        self
    }

    pub fn widget(self, widget: &(impl DocumentWidget + ?Sized)) -> DocumentView {
        self.try_widget(widget)
            .expect("document widget projection targets rendered elements")
    }

    pub fn try_widget(
        mut self,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> DocumentResult<DocumentView> {
        widget.push_styles(&mut self.stylesheet);
        let mut view = DocumentView::new(
            Document::build(self.viewport, |ui| ui.widget(widget)),
            self.stylesheet,
        );
        view.project_widget(widget)?;
        Ok(view)
    }

    pub fn widgets<'a, W>(self, widgets: impl IntoIterator<Item = &'a W>) -> DocumentView
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.try_widgets(widgets)
            .expect("document widget projection targets rendered elements")
    }

    pub fn try_widgets<'a, W>(
        mut self,
        widgets: impl IntoIterator<Item = &'a W>,
    ) -> DocumentResult<DocumentView>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        let widgets = widgets.into_iter().collect::<Vec<_>>();
        for widget in &widgets {
            widget.push_styles(&mut self.stylesheet);
        }
        let mut view = DocumentView::new(
            Document::build(self.viewport, |ui| {
                for widget in &widgets {
                    ui.widget(*widget);
                }
            }),
            self.stylesheet,
        );
        view.project_widgets(widgets)?;
        Ok(view)
    }

    pub fn build_with_widget(
        self,
        widget: &(impl DocumentWidget + ?Sized),
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentView {
        self.try_build_with_widget(widget, build)
            .expect("document widget projection targets rendered elements")
    }

    pub fn try_build_with_widget(
        mut self,
        widget: &(impl DocumentWidget + ?Sized),
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentResult<DocumentView> {
        widget.push_styles(&mut self.stylesheet);
        let mut view = self.build(build);
        view.project_widget(widget)?;
        Ok(view)
    }

    pub fn build_with_widgets<'a, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentView
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.try_build_with_widgets(widgets, build)
            .expect("document widget projection targets rendered elements")
    }

    pub fn try_build_with_widgets<'a, W>(
        mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentResult<DocumentView>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        let widgets = widgets.into_iter().collect::<Vec<_>>();
        for widget in &widgets {
            widget.push_styles(&mut self.stylesheet);
        }
        let mut view = self.build(build);
        view.project_widgets(widgets)?;
        Ok(view)
    }

    pub fn build(self, build: impl FnOnce(&mut DocumentBuilder)) -> DocumentView {
        DocumentView::new(Document::build(self.viewport, build), self.stylesheet)
    }
}
