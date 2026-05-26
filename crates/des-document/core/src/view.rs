use crate::{
    Document, DocumentActionWidget, DocumentBuilder, DocumentCommandAction,
    DocumentCommandDispatchReport, DocumentCommandRegistry, DocumentEngine, DocumentEventKind,
    DocumentInput, DocumentOutput, DocumentProjection, DocumentProjectionReport, DocumentResult,
    DocumentWidget, ElementBehaviorEvent, Size, StyleSheet, TextMeasurer,
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
    output: DocumentOutput,
    actions: Vec<DocumentCommandAction<Action>>,
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

    /// Iterates only the typed app action values collected for this frame.
    pub fn action_values(&self) -> impl Iterator<Item = &Action> {
        self.actions.iter().map(DocumentCommandAction::action)
    }

    /// Returns only the first typed app action value, when one was collected.
    pub fn first_action_value(&self) -> Option<&Action> {
        self.first_action().map(DocumentCommandAction::action)
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

    /// Returns the first typed app action emitted by one element.
    pub fn first_action_for(&self, target: &str) -> Option<&DocumentCommandAction<Action>> {
        self.actions
            .iter()
            .find(|action| action.target.as_str() == target)
    }

    /// Iterates only typed app action values emitted by one element.
    pub fn action_values_for<'a>(
        &'a self,
        target: &'a str,
    ) -> impl Iterator<Item = &'a Action> + 'a {
        self.actions_for(target).map(DocumentCommandAction::action)
    }

    /// Returns only the first typed app action value emitted by one element.
    pub fn first_action_value_for(&self, target: &str) -> Option<&Action> {
        self.first_action_for(target)
            .map(DocumentCommandAction::action)
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

    /// Returns the first typed app action emitted by one resolved document event kind.
    pub fn first_action_of_kind(
        &self,
        kind: DocumentEventKind,
    ) -> Option<&DocumentCommandAction<Action>> {
        self.actions.iter().find(|action| action.event == kind)
    }

    /// Iterates only typed app action values emitted by one resolved event kind.
    pub fn action_values_of_kind(&self, kind: DocumentEventKind) -> impl Iterator<Item = &Action> {
        self.actions_of_kind(kind)
            .map(DocumentCommandAction::action)
    }

    /// Returns only the first typed app action value emitted by one resolved event kind.
    pub fn first_action_value_of_kind(&self, kind: DocumentEventKind) -> Option<&Action> {
        self.first_action_of_kind(kind)
            .map(DocumentCommandAction::action)
    }

    /// Iterates typed app actions emitted by one authored behavior intent.
    pub fn actions_for_intent(
        &self,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = &DocumentCommandAction<Action>> {
        self.actions
            .iter()
            .filter(move |action| action.matches_intent(intent))
    }

    /// Returns the first typed app action emitted by one authored behavior intent.
    pub fn first_action_for_intent(
        &self,
        intent: ElementBehaviorEvent,
    ) -> Option<&DocumentCommandAction<Action>> {
        self.actions
            .iter()
            .find(|action| action.matches_intent(intent))
    }

    /// Iterates only typed app action values emitted by one behavior intent.
    pub fn action_values_for_intent(
        &self,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = &Action> {
        self.actions_for_intent(intent)
            .map(DocumentCommandAction::action)
    }

    /// Returns only the first typed app action value emitted by one behavior intent.
    pub fn first_action_value_for_intent(&self, intent: ElementBehaviorEvent) -> Option<&Action> {
        self.first_action_for_intent(intent)
            .map(DocumentCommandAction::action)
    }

    /// Iterates typed app actions emitted by one element and authored behavior intent.
    pub fn actions_for_target_intent<'a>(
        &'a self,
        target: &'a str,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = &'a DocumentCommandAction<Action>> + 'a {
        self.actions_for(target)
            .filter(move |action| action.matches_intent(intent))
    }

    /// Returns the first typed app action emitted by one element and behavior intent.
    pub fn first_action_for_target_intent(
        &self,
        target: &str,
        intent: ElementBehaviorEvent,
    ) -> Option<&DocumentCommandAction<Action>> {
        self.actions
            .iter()
            .find(|action| action.target.as_str() == target && action.matches_intent(intent))
    }

    /// Iterates only typed app action values emitted by one element and behavior intent.
    pub fn action_values_for_target_intent<'a>(
        &'a self,
        target: &'a str,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = &'a Action> + 'a {
        self.actions_for_target_intent(target, intent)
            .map(DocumentCommandAction::action)
    }

    /// Returns only the first typed app action value emitted by one element and intent.
    pub fn first_action_value_for_target_intent(
        &self,
        target: &str,
        intent: ElementBehaviorEvent,
    ) -> Option<&Action> {
        self.first_action_for_target_intent(target, intent)
            .map(DocumentCommandAction::action)
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

    /// Returns true when the supplied element emitted the supplied typed action.
    pub fn contains_action_for(&self, target: &str, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.actions_for(target)
            .any(|candidate| &candidate.action == action)
    }

    /// Returns true when the supplied event kind emitted the supplied typed action.
    pub fn contains_action_of_kind(&self, kind: DocumentEventKind, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.actions_of_kind(kind)
            .any(|candidate| &candidate.action == action)
    }

    /// Returns true when the supplied behavior intent emitted the supplied typed action.
    pub fn contains_action_for_intent(&self, intent: ElementBehaviorEvent, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.actions_for_intent(intent)
            .any(|candidate| &candidate.action == action)
    }

    /// Returns true when the supplied element and behavior intent emitted the action.
    pub fn contains_action_for_target_intent(
        &self,
        target: &str,
        intent: ElementBehaviorEvent,
        action: &Action,
    ) -> bool
    where
        Action: PartialEq,
    {
        self.actions_for_target_intent(target, intent)
            .any(|candidate| &candidate.action == action)
    }

    /// Dispatches every collected typed action to a handler.
    ///
    /// Because a `DocumentActionFrame` only stores commands that already mapped
    /// to typed actions, the returned report treats every inspected action as
    /// handled and never reports unhandled commands.
    pub fn dispatch<'a>(
        &'a self,
        handler: impl FnMut(&'a DocumentCommandAction<Action>),
    ) -> DocumentCommandDispatchReport {
        self.dispatch_matching(|_| true, handler)
    }

    /// Dispatches collected typed actions emitted by one element.
    pub fn dispatch_for<'a>(
        &'a self,
        target: &'a str,
        handler: impl FnMut(&'a DocumentCommandAction<Action>),
    ) -> DocumentCommandDispatchReport {
        self.dispatch_matching(|action| action.target.as_str() == target, handler)
    }

    /// Dispatches collected typed actions emitted by one resolved event kind.
    pub fn dispatch_kind<'a>(
        &'a self,
        kind: DocumentEventKind,
        handler: impl FnMut(&'a DocumentCommandAction<Action>),
    ) -> DocumentCommandDispatchReport {
        self.dispatch_matching(|action| action.event == kind, handler)
    }

    /// Dispatches collected typed actions emitted by one authored behavior intent.
    pub fn dispatch_intent<'a>(
        &'a self,
        intent: ElementBehaviorEvent,
        handler: impl FnMut(&'a DocumentCommandAction<Action>),
    ) -> DocumentCommandDispatchReport {
        self.dispatch_matching(|action| action.matches_intent(intent), handler)
    }

    /// Dispatches every collected typed app action value to a handler.
    ///
    /// Use this when app code only needs the domain action and does not need
    /// document target, event, or command metadata.
    pub fn dispatch_action_values<'a>(
        &'a self,
        mut handler: impl FnMut(&'a Action),
    ) -> DocumentCommandDispatchReport {
        self.dispatch(|action| handler(action.action()))
    }

    /// Dispatches typed app action values emitted by one element.
    pub fn dispatch_action_values_for<'a>(
        &'a self,
        target: &'a str,
        mut handler: impl FnMut(&'a Action),
    ) -> DocumentCommandDispatchReport {
        self.dispatch_for(target, |action| handler(action.action()))
    }

    /// Dispatches typed app action values emitted by one resolved event kind.
    pub fn dispatch_action_values_of_kind<'a>(
        &'a self,
        kind: DocumentEventKind,
        mut handler: impl FnMut(&'a Action),
    ) -> DocumentCommandDispatchReport {
        self.dispatch_kind(kind, |action| handler(action.action()))
    }

    /// Dispatches typed app action values emitted by one authored behavior intent.
    pub fn dispatch_action_values_for_intent<'a>(
        &'a self,
        intent: ElementBehaviorEvent,
        mut handler: impl FnMut(&'a Action),
    ) -> DocumentCommandDispatchReport {
        self.dispatch_intent(intent, |action| handler(action.action()))
    }

    /// Consumes the frame into the resolved output and collected app actions.
    pub fn into_parts(self) -> (DocumentOutput, Vec<DocumentCommandAction<Action>>) {
        (self.output, self.actions)
    }

    /// Consumes the frame and returns only the collected app actions.
    pub fn into_actions(self) -> Vec<DocumentCommandAction<Action>> {
        self.actions
    }

    /// Consumes the frame and returns only the typed app action values.
    pub fn into_action_values(self) -> Vec<Action> {
        self.actions
            .into_iter()
            .map(DocumentCommandAction::into_action)
            .collect()
    }

    /// Consumes the frame and returns only the resolved document output.
    pub fn into_output(self) -> DocumentOutput {
        self.output
    }

    fn dispatch_matching<'a>(
        &'a self,
        mut matches: impl FnMut(&DocumentCommandAction<Action>) -> bool,
        mut handler: impl FnMut(&'a DocumentCommandAction<Action>),
    ) -> DocumentCommandDispatchReport {
        let mut handled = 0;
        for action in &self.actions {
            if matches(action) {
                handled += 1;
                handler(action);
            }
        }
        DocumentCommandDispatchReport::new(handled, handled, 0)
    }
}

/// Output from a fluent document update request.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentUpdateFrame {
    projection_report: Option<DocumentProjectionReport>,
    output: DocumentOutput,
}

impl DocumentUpdateFrame {
    /// Returns the projection report when this request applied app state first.
    pub fn projection_report(&self) -> Option<&DocumentProjectionReport> {
        self.projection_report.as_ref()
    }

    /// Returns the resolved document output for rendering and interaction queries.
    pub fn output(&self) -> &DocumentOutput {
        &self.output
    }

    /// Consumes this frame into its optional projection report and output.
    pub fn into_parts(self) -> (Option<DocumentProjectionReport>, DocumentOutput) {
        (self.projection_report, self.output)
    }
}

/// Typed actions collected from a fluent document update request.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentActionUpdateFrame<Action> {
    projection_report: Option<DocumentProjectionReport>,
    frame: DocumentActionFrame<Action>,
}

impl<Action> DocumentActionUpdateFrame<Action> {
    /// Returns the projection report when this request applied app state first.
    pub fn projection_report(&self) -> Option<&DocumentProjectionReport> {
        self.projection_report.as_ref()
    }

    /// Returns the resolved document output and collected typed actions.
    pub fn frame(&self) -> &DocumentActionFrame<Action> {
        &self.frame
    }

    /// Returns the resolved document output for rendering and interaction queries.
    pub fn output(&self) -> &DocumentOutput {
        self.frame.output()
    }

    /// Returns the collected typed app actions in document event order.
    pub fn actions(&self) -> &[DocumentCommandAction<Action>] {
        self.frame.actions()
    }

    /// Returns true when this frame collected no typed app actions.
    pub fn is_empty(&self) -> bool {
        self.frame.is_empty()
    }

    /// Returns the number of typed app actions collected for this frame.
    pub fn len(&self) -> usize {
        self.frame.len()
    }

    /// Returns the first typed app action, when one was collected.
    pub fn first_action(&self) -> Option<&DocumentCommandAction<Action>> {
        self.frame.first_action()
    }

    /// Iterates only the typed app action values collected for this frame.
    pub fn action_values(&self) -> impl Iterator<Item = &Action> {
        self.frame.action_values()
    }

    /// Returns only the first typed app action value, when one was collected.
    pub fn first_action_value(&self) -> Option<&Action> {
        self.frame.first_action_value()
    }

    /// Iterates typed app actions emitted by one element.
    pub fn actions_for<'a>(
        &'a self,
        target: &'a str,
    ) -> impl Iterator<Item = &'a DocumentCommandAction<Action>> + 'a {
        self.frame.actions_for(target)
    }

    /// Returns the first typed app action emitted by one element.
    pub fn first_action_for(&self, target: &str) -> Option<&DocumentCommandAction<Action>> {
        self.frame.first_action_for(target)
    }

    /// Iterates only typed app action values emitted by one element.
    pub fn action_values_for<'a>(
        &'a self,
        target: &'a str,
    ) -> impl Iterator<Item = &'a Action> + 'a {
        self.frame.action_values_for(target)
    }

    /// Returns only the first typed app action value emitted by one element.
    pub fn first_action_value_for(&self, target: &str) -> Option<&Action> {
        self.frame.first_action_value_for(target)
    }

    /// Iterates typed app actions emitted by one resolved document event kind.
    pub fn actions_of_kind(
        &self,
        kind: DocumentEventKind,
    ) -> impl Iterator<Item = &DocumentCommandAction<Action>> {
        self.frame.actions_of_kind(kind)
    }

    /// Returns the first typed app action emitted by one resolved document event kind.
    pub fn first_action_of_kind(
        &self,
        kind: DocumentEventKind,
    ) -> Option<&DocumentCommandAction<Action>> {
        self.frame.first_action_of_kind(kind)
    }

    /// Iterates only typed app action values emitted by one resolved event kind.
    pub fn action_values_of_kind(&self, kind: DocumentEventKind) -> impl Iterator<Item = &Action> {
        self.frame.action_values_of_kind(kind)
    }

    /// Returns only the first typed app action value emitted by one resolved event kind.
    pub fn first_action_value_of_kind(&self, kind: DocumentEventKind) -> Option<&Action> {
        self.frame.first_action_value_of_kind(kind)
    }

    /// Iterates typed app actions emitted by one authored behavior intent.
    pub fn actions_for_intent(
        &self,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = &DocumentCommandAction<Action>> {
        self.frame.actions_for_intent(intent)
    }

    /// Returns the first typed app action emitted by one authored behavior intent.
    pub fn first_action_for_intent(
        &self,
        intent: ElementBehaviorEvent,
    ) -> Option<&DocumentCommandAction<Action>> {
        self.frame.first_action_for_intent(intent)
    }

    /// Iterates only typed app action values emitted by one behavior intent.
    pub fn action_values_for_intent(
        &self,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = &Action> {
        self.frame.action_values_for_intent(intent)
    }

    /// Returns only the first typed app action value emitted by one behavior intent.
    pub fn first_action_value_for_intent(&self, intent: ElementBehaviorEvent) -> Option<&Action> {
        self.frame.first_action_value_for_intent(intent)
    }

    /// Iterates typed app actions emitted by one element and authored behavior intent.
    pub fn actions_for_target_intent<'a>(
        &'a self,
        target: &'a str,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = &'a DocumentCommandAction<Action>> + 'a {
        self.frame.actions_for_target_intent(target, intent)
    }

    /// Returns the first typed app action emitted by one element and behavior intent.
    pub fn first_action_for_target_intent(
        &self,
        target: &str,
        intent: ElementBehaviorEvent,
    ) -> Option<&DocumentCommandAction<Action>> {
        self.frame.first_action_for_target_intent(target, intent)
    }

    /// Iterates only typed app action values emitted by one element and behavior intent.
    pub fn action_values_for_target_intent<'a>(
        &'a self,
        target: &'a str,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = &'a Action> + 'a {
        self.frame.action_values_for_target_intent(target, intent)
    }

    /// Returns only the first typed app action value emitted by one element and intent.
    pub fn first_action_value_for_target_intent(
        &self,
        target: &str,
        intent: ElementBehaviorEvent,
    ) -> Option<&Action> {
        self.frame
            .first_action_value_for_target_intent(target, intent)
    }

    /// Returns true when the frame contains the supplied typed action.
    pub fn contains_action(&self, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.frame.contains_action(action)
    }

    /// Returns true when the supplied element emitted the supplied typed action.
    pub fn contains_action_for(&self, target: &str, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.frame.contains_action_for(target, action)
    }

    /// Returns true when the supplied event kind emitted the supplied typed action.
    pub fn contains_action_of_kind(&self, kind: DocumentEventKind, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.frame.contains_action_of_kind(kind, action)
    }

    /// Returns true when the supplied behavior intent emitted the supplied typed action.
    pub fn contains_action_for_intent(&self, intent: ElementBehaviorEvent, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.frame.contains_action_for_intent(intent, action)
    }

    /// Returns true when the supplied element and behavior intent emitted the action.
    pub fn contains_action_for_target_intent(
        &self,
        target: &str,
        intent: ElementBehaviorEvent,
        action: &Action,
    ) -> bool
    where
        Action: PartialEq,
    {
        self.frame
            .contains_action_for_target_intent(target, intent, action)
    }

    /// Consumes this frame into its optional projection report and action frame.
    pub fn into_parts(
        self,
    ) -> (
        Option<DocumentProjectionReport>,
        DocumentActionFrame<Action>,
    ) {
        (self.projection_report, self.frame)
    }
}

/// Fluent request for uncommon document update combinations.
///
/// Prefer the short `update`/`update_with_input` helpers for the simplest
/// frames. Use this request when a frame needs projection, host input, action
/// collection, or dispatch without adding another long method permutation.
pub struct DocumentUpdateRequest<'view> {
    view: &'view mut DocumentView,
    projection: Option<DocumentProjection>,
    input: DocumentInput,
}

impl<'view> DocumentUpdateRequest<'view> {
    fn new(view: &'view mut DocumentView) -> Self {
        Self {
            view,
            projection: None,
            input: DocumentInput::default(),
        }
    }

    /// Applies this projection before routing input and resolving the frame.
    pub fn projection(mut self, projection: DocumentProjection) -> Self {
        self.projection = Some(projection);
        self
    }

    /// Builds and applies a projection before routing input and resolving the frame.
    pub fn project_with(mut self, project: impl FnOnce(&mut DocumentProjection)) -> Self {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.projection = Some(projection);
        self
    }

    /// Routes host input through this update frame.
    pub fn input(mut self, input: DocumentInput) -> Self {
        self.input = input;
        self
    }

    /// Resolves the requested frame.
    pub fn update(self) -> DocumentResult<DocumentUpdateFrame> {
        let projection_report = if let Some(projection) = self.projection {
            Some(self.view.project(&projection)?)
        } else {
            None
        };
        let output = self.view.update_with_input(self.input);
        Ok(DocumentUpdateFrame {
            projection_report,
            output,
        })
    }

    /// Resolves the requested frame and collects typed app actions.
    pub fn update_actions<Action>(
        self,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<DocumentActionUpdateFrame<Action>>
    where
        Action: Clone,
    {
        let projection_report = if let Some(projection) = self.projection {
            Some(self.view.project(&projection)?)
        } else {
            None
        };
        let output = self.view.update_with_input(self.input);
        let frame = DocumentView::collect_action_frame(registry, output);
        Ok(DocumentActionUpdateFrame {
            projection_report,
            frame,
        })
    }

    /// Resolves the requested frame, collects typed app actions, and dispatches them.
    pub fn dispatch<Action>(
        self,
        registry: &DocumentCommandRegistry<Action>,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> DocumentResult<(
        DocumentActionUpdateFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Action: Clone,
    {
        let frame = self.update_actions(registry)?;
        let report = frame.frame.dispatch(handler);
        Ok((frame, report))
    }

    /// Resolves the requested frame and dispatches only typed app action values.
    pub fn dispatch_action_values<Action>(
        self,
        registry: &DocumentCommandRegistry<Action>,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> DocumentResult<(
        DocumentActionUpdateFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Action: Clone,
    {
        let frame = self.update_actions(registry)?;
        let report = frame.frame.dispatch_action_values(handler);
        Ok((frame, report))
    }
}

/// A retained document view paired with the typed command registry that drives it.
///
/// Action surfaces are the ergonomic app-facing shape for reusable widgets:
/// widget structure, styles, projection, and command bindings are mounted
/// together, then update calls can collect typed app actions without passing the
/// same registry through every frame.
pub struct DocumentActionSurface<Action> {
    view: DocumentView,
    commands: DocumentCommandRegistry<Action>,
}

/// Fluent request for updating an action surface with its bound commands.
pub struct DocumentActionSurfaceUpdateRequest<'surface, Action> {
    surface: &'surface mut DocumentActionSurface<Action>,
    projection: Option<DocumentProjection>,
    input: DocumentInput,
}

impl<'surface, Action> DocumentActionSurfaceUpdateRequest<'surface, Action> {
    fn new(surface: &'surface mut DocumentActionSurface<Action>) -> Self {
        Self {
            surface,
            projection: None,
            input: DocumentInput::default(),
        }
    }

    /// Applies this projection before routing input and collecting actions.
    pub fn projection(mut self, projection: DocumentProjection) -> Self {
        self.projection = Some(projection);
        self
    }

    /// Builds and applies a projection before routing input and collecting actions.
    pub fn project_with(mut self, project: impl FnOnce(&mut DocumentProjection)) -> Self {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.projection = Some(projection);
        self
    }

    /// Routes host input through this update frame.
    pub fn input(mut self, input: DocumentInput) -> Self {
        self.input = input;
        self
    }

    /// Resolves the requested frame and collects typed app actions.
    pub fn update(self) -> DocumentResult<DocumentActionUpdateFrame<Action>>
    where
        Action: Clone,
    {
        let projection_report = if let Some(projection) = self.projection {
            Some(self.surface.project(&projection)?)
        } else {
            None
        };
        let frame = self.surface.update_with_input_actions(self.input);
        Ok(DocumentActionUpdateFrame {
            projection_report,
            frame,
        })
    }

    /// Resolves the requested frame, collects typed app actions, and dispatches them.
    pub fn dispatch(
        self,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> DocumentResult<(
        DocumentActionUpdateFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Action: Clone,
    {
        let frame = self.update()?;
        let report = frame.frame.dispatch(handler);
        Ok((frame, report))
    }

    /// Resolves the requested frame and dispatches only typed app action values.
    pub fn dispatch_action_values(
        self,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> DocumentResult<(
        DocumentActionUpdateFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Action: Clone,
    {
        let frame = self.update()?;
        let report = frame.frame.dispatch_action_values(handler);
        Ok((frame, report))
    }
}

impl<Action> DocumentActionSurface<Action> {
    /// Creates an action surface from an already-built view and command registry.
    pub fn new(view: DocumentView, commands: DocumentCommandRegistry<Action>) -> Self {
        Self { view, commands }
    }

    /// Starts a fluent update request using this surface's bound command registry.
    pub fn update_request(&mut self) -> DocumentActionSurfaceUpdateRequest<'_, Action> {
        DocumentActionSurfaceUpdateRequest::new(self)
    }

    /// Returns the retained document view.
    pub fn view(&self) -> &DocumentView {
        &self.view
    }

    /// Returns the retained document view for app state projection and updates.
    pub fn view_mut(&mut self) -> &mut DocumentView {
        &mut self.view
    }

    /// Returns the stylesheet paired with this action surface.
    pub fn stylesheet(&self) -> &StyleSheet {
        self.view.stylesheet()
    }

    /// Returns the stylesheet for controlled app-specific extension.
    pub fn stylesheet_mut(&mut self) -> &mut StyleSheet {
        self.view.stylesheet_mut()
    }

    /// Replaces the stylesheet used by the paired view.
    pub fn replace_stylesheet(&mut self, stylesheet: StyleSheet) {
        self.view.replace_stylesheet(stylesheet);
    }

    /// Extends the paired stylesheet and returns the surface.
    pub fn with_stylesheet(mut self, stylesheet: StyleSheet) -> Self {
        self.extend_stylesheet(stylesheet);
        self
    }

    /// Extends the paired stylesheet in place.
    pub fn extend_stylesheet(&mut self, stylesheet: StyleSheet) -> &mut Self {
        self.view.extend_stylesheet(stylesheet);
        self
    }

    /// Conditionally extends the paired stylesheet and returns the surface.
    pub fn with_stylesheet_if(mut self, stylesheet: StyleSheet, present: bool) -> Self {
        self.extend_stylesheet_if(stylesheet, present);
        self
    }

    /// Conditionally extends the paired stylesheet in place.
    pub fn extend_stylesheet_if(&mut self, stylesheet: StyleSheet, present: bool) -> &mut Self {
        self.view.extend_stylesheet_if(stylesheet, present);
        self
    }

    /// Parses strict CSS into the paired stylesheet.
    pub fn extend_css(&mut self, css: &str) -> Result<&mut Self, crate::CssParseError> {
        self.view.extend_css(css)?;
        Ok(self)
    }

    /// Parses strict CSS into the paired stylesheet and returns the surface.
    pub fn with_css(mut self, css: &str) -> Result<Self, crate::CssParseError> {
        self.extend_css(css)?;
        Ok(self)
    }

    /// Conditionally parses strict CSS into the paired stylesheet.
    pub fn extend_css_if(
        &mut self,
        present: bool,
        css: &str,
    ) -> Result<&mut Self, crate::CssParseError> {
        self.view.extend_css_if(present, css)?;
        Ok(self)
    }

    /// Conditionally parses strict CSS into the paired stylesheet and returns the surface.
    pub fn with_css_if(mut self, present: bool, css: &str) -> Result<Self, crate::CssParseError> {
        self.extend_css_if(present, css)?;
        Ok(self)
    }

    /// Parses browser-forgiving CSS into the paired stylesheet.
    pub fn extend_css_forgiving(&mut self, css: &str) -> Result<&mut Self, crate::CssParseError> {
        self.view.extend_css_forgiving(css)?;
        Ok(self)
    }

    /// Parses browser-forgiving CSS into the paired stylesheet and returns the surface.
    pub fn with_css_forgiving(mut self, css: &str) -> Result<Self, crate::CssParseError> {
        self.extend_css_forgiving(css)?;
        Ok(self)
    }

    /// Conditionally parses browser-forgiving CSS into the paired stylesheet.
    pub fn extend_css_forgiving_if(
        &mut self,
        present: bool,
        css: &str,
    ) -> Result<&mut Self, crate::CssParseError> {
        self.view.extend_css_forgiving_if(present, css)?;
        Ok(self)
    }

    /// Conditionally parses browser-forgiving CSS and returns the surface.
    pub fn with_css_forgiving_if(
        mut self,
        present: bool,
        css: &str,
    ) -> Result<Self, crate::CssParseError> {
        self.extend_css_forgiving_if(present, css)?;
        Ok(self)
    }

    /// Returns the typed command registry paired with this view.
    pub fn commands(&self) -> &DocumentCommandRegistry<Action> {
        &self.commands
    }

    /// Returns the typed command registry for app-specific extensions.
    pub fn commands_mut(&mut self) -> &mut DocumentCommandRegistry<Action> {
        &mut self.commands
    }

    /// Configures the paired command registry and returns the surface.
    pub fn with_commands(
        mut self,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> Self {
        configure(&mut self.commands);
        self
    }

    /// Adds one command binding and returns the surface.
    pub fn bind(mut self, command: impl Into<String>, action: Action) -> Self {
        self.commands.push(command, action);
        self
    }

    /// Conditionally adds one command binding and returns the surface.
    pub fn bind_if(mut self, command: impl Into<String>, action: Action, present: bool) -> Self {
        self.commands.push_if(command, action, present);
        self
    }

    /// Adds one intent-scoped command binding and returns the surface.
    pub fn bind_on(
        mut self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        action: Action,
    ) -> Self {
        self.commands.push_on(event, command, action);
        self
    }

    /// Conditionally adds one intent-scoped command binding and returns the surface.
    pub fn bind_on_if(
        mut self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.commands.push_on_if(event, command, action, present);
        self
    }

    /// Adds command bindings and returns the surface.
    pub fn bind_many<Command>(
        mut self,
        bindings: impl IntoIterator<Item = (Command, Action)>,
    ) -> Self
    where
        Command: Into<String>,
    {
        self.commands.push_many(bindings);
        self
    }

    /// Conditionally adds command bindings and returns the surface.
    pub fn bind_many_if<Command>(
        mut self,
        bindings: impl IntoIterator<Item = (Command, Action)>,
        present: bool,
    ) -> Self
    where
        Command: Into<String>,
    {
        self.commands.push_many_if(bindings, present);
        self
    }

    /// Adds intent-scoped command bindings and returns the surface.
    pub fn bind_on_many<Command>(
        mut self,
        bindings: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> Self
    where
        Command: Into<String>,
    {
        self.commands.push_on_many(bindings);
        self
    }

    /// Conditionally adds intent-scoped command bindings and returns the surface.
    pub fn bind_on_many_if<Command>(
        mut self,
        bindings: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
        present: bool,
    ) -> Self
    where
        Command: Into<String>,
    {
        self.commands.push_on_many_if(bindings, present);
        self
    }

    /// Adds command bindings declared by a reusable action widget.
    pub fn bind_widget(mut self, widget: &(impl DocumentActionWidget<Action> + ?Sized)) -> Self {
        self.push_widget_commands(widget);
        self
    }

    /// Conditionally adds command bindings declared by a reusable action widget.
    pub fn bind_widget_if(
        mut self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        present: bool,
    ) -> Self {
        self.push_widget_commands_if(widget, present);
        self
    }

    /// Adds command bindings declared by a collection of reusable action widgets.
    pub fn bind_widgets<'a, W>(mut self, widgets: impl IntoIterator<Item = &'a W>) -> Self
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.push_widget_commands_many(widgets);
        self
    }

    /// Conditionally adds command bindings declared by reusable action widgets.
    pub fn bind_widgets_if<'a, W>(
        mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) -> Self
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.push_widget_commands_many_if(widgets, present);
        self
    }

    /// Adds command bindings declared by a reusable action widget.
    pub fn push_widget_commands(&mut self, widget: &(impl DocumentActionWidget<Action> + ?Sized)) {
        self.commands.push_widget_commands(widget);
    }

    /// Conditionally adds command bindings declared by a reusable action widget.
    pub fn push_widget_commands_if(
        &mut self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        present: bool,
    ) {
        self.commands.push_widget_commands_if(widget, present);
    }

    /// Adds command bindings declared by a collection of reusable action widgets.
    pub fn push_widget_commands_many<'a, W>(&mut self, widgets: impl IntoIterator<Item = &'a W>)
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.commands.push_widget_commands_many(widgets);
    }

    /// Conditionally adds command bindings declared by reusable action widgets.
    pub fn push_widget_commands_many_if<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.commands.push_widget_commands_many_if(widgets, present);
    }

    /// Applies a batch of app-state projections to the paired view.
    pub fn project(
        &mut self,
        projection: &DocumentProjection,
    ) -> DocumentResult<DocumentProjectionReport> {
        self.view.project(projection)
    }

    /// Builds and applies a projection to the paired view in one call.
    pub fn project_with(
        &mut self,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<DocumentProjectionReport> {
        self.view.project_with(project)
    }

    /// Applies app-state projections declared by a reusable document widget.
    pub fn project_widget(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> DocumentResult<DocumentProjectionReport> {
        self.view.project_widget(widget)
    }

    /// Applies app-state projections declared by a collection of widgets.
    pub fn project_widgets<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
    ) -> DocumentResult<DocumentProjectionReport>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.view.project_widgets(widgets)
    }

    /// Applies a projection, resolves the paired view, and returns output.
    pub fn project_and_update(
        &mut self,
        projection: &DocumentProjection,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        self.view.project_and_update(projection)
    }

    /// Builds a projection, applies it, resolves the paired view, and returns output.
    pub fn project_with_and_update(
        &mut self,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        self.view.project_with_and_update(project)
    }

    /// Applies a projection, resolves the view, and collects typed app actions.
    pub fn project_and_update_actions(
        &mut self,
        projection: &DocumentProjection,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        self.view
            .project_and_update_actions(projection, &self.commands)
    }

    /// Builds a projection, applies it, resolves the view, and collects typed actions.
    pub fn project_with_and_update_actions(
        &mut self,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        self.view
            .project_with_and_update_actions(project, &self.commands)
    }

    /// Applies a widget projection, resolves the view, and collects typed app actions.
    pub fn project_widget_and_update_actions(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        self.view
            .project_widget_and_update_actions(widget, &self.commands)
    }

    /// Applies widget projections, resolves the view, and collects typed app actions.
    pub fn project_widgets_and_update_actions<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        W: DocumentWidget + ?Sized + 'a,
        Action: Clone,
    {
        self.view
            .project_widgets_and_update_actions(widgets, &self.commands)
    }

    /// Applies a widget projection, resolves the paired view, and returns output.
    pub fn project_widget_and_update(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        self.view.project_widget_and_update(widget)
    }

    /// Applies widget projections, resolves the paired view, and returns output.
    pub fn project_widgets_and_update<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.view.project_widgets_and_update(widgets)
    }

    /// Applies a projection, routes input through the paired view, and returns output.
    pub fn project_and_update_with_input(
        &mut self,
        projection: &DocumentProjection,
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        self.view.project_and_update_with_input(projection, input)
    }

    /// Builds a projection, routes input through the paired view, and returns output.
    pub fn project_with_and_update_with_input(
        &mut self,
        input: DocumentInput,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        self.view.project_with_and_update_with_input(input, project)
    }

    /// Applies a projection, routes input, and collects typed app actions.
    pub fn project_and_update_with_input_actions(
        &mut self,
        projection: &DocumentProjection,
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        self.view
            .project_and_update_with_input_actions(projection, input, &self.commands)
    }

    /// Builds a projection, applies it, routes input, and collects typed actions.
    pub fn project_with_and_update_with_input_actions(
        &mut self,
        input: DocumentInput,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        self.view
            .project_with_and_update_with_input_actions(input, project, &self.commands)
    }

    /// Applies a projection, routes input through a host text measurer, and returns output.
    pub fn project_and_update_with_input_and_text_measurer(
        &mut self,
        projection: &DocumentProjection,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        let report = self.project(projection)?;
        let output = self
            .view
            .update_with_input_and_text_measurer(input, text_measurer);
        Ok((report, output))
    }

    /// Builds a projection, routes input through a host text measurer, and returns output.
    pub fn project_with_and_update_with_input_and_text_measurer(
        &mut self,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.project_and_update_with_input_and_text_measurer(&projection, input, text_measurer)
    }

    /// Applies a projection, routes input through a host text measurer, and collects actions.
    pub fn project_and_update_with_input_and_text_measurer_actions(
        &mut self,
        projection: &DocumentProjection,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        self.view
            .project_and_update_with_input_and_text_measurer_actions(
                projection,
                input,
                text_measurer,
                &self.commands,
            )
    }

    /// Builds a projection, routes input through a host text measurer, and collects actions.
    pub fn project_with_and_update_with_input_and_text_measurer_actions(
        &mut self,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        self.view
            .project_with_and_update_with_input_and_text_measurer_actions(
                input,
                text_measurer,
                project,
                &self.commands,
            )
    }

    /// Applies a widget projection, routes input through the paired view, and returns output.
    pub fn project_widget_and_update_with_input(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)> {
        self.view
            .project_widget_and_update_with_input(widget, input)
    }

    /// Applies a widget projection, routes input, and collects typed app actions.
    pub fn project_widget_and_update_with_input_actions(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        self.view
            .project_widget_and_update_with_input_actions(widget, input, &self.commands)
    }

    /// Applies widget projections, routes input through the paired view, and returns output.
    pub fn project_widgets_and_update_with_input<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentOutput)>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.view
            .project_widgets_and_update_with_input(widgets, input)
    }

    /// Applies widget projections, routes input, and collects typed app actions.
    pub fn project_widgets_and_update_with_input_actions<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        input: DocumentInput,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        W: DocumentWidget + ?Sized + 'a,
        Action: Clone,
    {
        self.view
            .project_widgets_and_update_with_input_actions(widgets, input, &self.commands)
    }

    /// Resolves the view and collects typed app actions with the paired registry.
    pub fn update_actions(&mut self) -> DocumentActionFrame<Action>
    where
        Action: Clone,
    {
        self.view.update_actions(&self.commands)
    }

    /// Routes input, resolves the view, and collects typed app actions.
    pub fn update_with_input_actions(&mut self, input: DocumentInput) -> DocumentActionFrame<Action>
    where
        Action: Clone,
    {
        self.view.update_with_input_actions(input, &self.commands)
    }

    /// Routes input with a host text measurer and collects typed app actions.
    pub fn update_with_input_and_text_measurer_actions(
        &mut self,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentActionFrame<Action>
    where
        Action: Clone,
    {
        self.view
            .update_with_input_and_text_measurer_actions(input, text_measurer, &self.commands)
    }

    /// Splits the surface into its owned view and typed command registry.
    pub fn into_parts(self) -> (DocumentView, DocumentCommandRegistry<Action>) {
        (self.view, self.commands)
    }

    /// Consumes the surface and returns only the retained document view.
    pub fn into_view(self) -> DocumentView {
        self.view
    }

    /// Consumes the surface and returns only the typed command registry.
    pub fn into_commands(self) -> DocumentCommandRegistry<Action> {
        self.commands
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
            Document::build(viewport, |ui| {
                ui.widget(widget);
            }),
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

    /// Consumes the view into an action surface paired with a command registry.
    pub fn action_surface<Action>(
        self,
        commands: DocumentCommandRegistry<Action>,
    ) -> DocumentActionSurface<Action> {
        DocumentActionSurface::new(self, commands)
    }

    /// Consumes the view into an action surface configured in one app-facing hook.
    pub fn action_surface_with<Action>(
        self,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> DocumentActionSurface<Action> {
        let mut commands = DocumentCommandRegistry::new();
        configure(&mut commands);
        self.action_surface(commands)
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

    /// Applies a projection, resolves the document, and collects typed app actions.
    pub fn project_and_update_actions<Action>(
        &mut self,
        projection: &DocumentProjection,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let report = self.project(projection)?;
        let frame = self.update_actions(registry);
        Ok((report, frame))
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

    /// Builds a projection, applies it, resolves the document, and collects typed actions.
    pub fn project_with_and_update_actions<Action>(
        &mut self,
        project: impl FnOnce(&mut DocumentProjection),
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.project_and_update_actions(&projection, registry)
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

    /// Applies a projection, routes input, resolves the document, and collects typed actions.
    pub fn project_and_update_with_input_actions<Action>(
        &mut self,
        projection: &DocumentProjection,
        input: DocumentInput,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let report = self.project(projection)?;
        let frame = self.update_with_input_actions(input, registry);
        Ok((report, frame))
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

    /// Builds a projection, applies it, routes input, and collects typed actions.
    pub fn project_with_and_update_with_input_actions<Action>(
        &mut self,
        input: DocumentInput,
        project: impl FnOnce(&mut DocumentProjection),
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.project_and_update_with_input_actions(&projection, input, registry)
    }

    /// Applies a projection, routes input with a host text measurer, and collects actions.
    pub fn project_and_update_with_input_and_text_measurer_actions<Action>(
        &mut self,
        projection: &DocumentProjection,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let report = self.project(projection)?;
        let frame =
            self.update_with_input_and_text_measurer_actions(input, text_measurer, registry);
        Ok((report, frame))
    }

    /// Builds a projection, routes input with a host text measurer, and collects actions.
    pub fn project_with_and_update_with_input_and_text_measurer_actions<Action>(
        &mut self,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
        project: impl FnOnce(&mut DocumentProjection),
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.project_and_update_with_input_and_text_measurer_actions(
            &projection,
            input,
            text_measurer,
            registry,
        )
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

    /// Applies a widget projection, resolves the document, and collects typed actions.
    pub fn project_widget_and_update_actions<Action>(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let report = self.project_widget(widget)?;
        let frame = self.update_actions(registry);
        Ok((report, frame))
    }

    /// Applies a widget projection, resolves the document, and maps command names to actions.
    pub fn project_widget_and_update_bound_actions<Action, Command>(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
        Command: Into<String>,
    {
        let registry = DocumentCommandRegistry::new().bind_many(actions);
        self.project_widget_and_update_actions(widget, &registry)
    }

    /// Applies a widget projection, resolves the document, and maps intent-scoped commands.
    pub fn project_widget_and_update_bound_intent_actions<Action, Command>(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
        Command: Into<String>,
    {
        let registry = DocumentCommandRegistry::new().bind_on_many(actions);
        self.project_widget_and_update_actions(widget, &registry)
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

    /// Applies widget projections, resolves the document, and collects typed actions.
    pub fn project_widgets_and_update_actions<'a, W, Action>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        W: DocumentWidget + ?Sized + 'a,
        Action: Clone,
    {
        let report = self.project_widgets(widgets)?;
        let frame = self.update_actions(registry);
        Ok((report, frame))
    }

    /// Applies widget projections, resolves the document, and maps command names to actions.
    pub fn project_widgets_and_update_bound_actions<'a, W, Action, Command>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        W: DocumentWidget + ?Sized + 'a,
        Action: Clone,
        Command: Into<String>,
    {
        let registry = DocumentCommandRegistry::new().bind_many(actions);
        self.project_widgets_and_update_actions(widgets, &registry)
    }

    /// Applies widget projections, resolves the document, and maps intent-scoped commands.
    pub fn project_widgets_and_update_bound_intent_actions<'a, W, Action, Command>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        W: DocumentWidget + ?Sized + 'a,
        Action: Clone,
        Command: Into<String>,
    {
        let registry = DocumentCommandRegistry::new().bind_on_many(actions);
        self.project_widgets_and_update_actions(widgets, &registry)
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

    /// Applies a widget projection, routes input, and collects typed actions.
    pub fn project_widget_and_update_with_input_actions<Action>(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        input: DocumentInput,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let report = self.project_widget(widget)?;
        let frame = self.update_with_input_actions(input, registry);
        Ok((report, frame))
    }

    /// Applies a widget projection, routes input, and maps command names to actions.
    pub fn project_widget_and_update_with_input_bound_actions<Action, Command>(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        input: DocumentInput,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
        Command: Into<String>,
    {
        let registry = DocumentCommandRegistry::new().bind_many(actions);
        self.project_widget_and_update_with_input_actions(widget, input, &registry)
    }

    /// Applies a widget projection, routes input, and maps intent-scoped commands.
    pub fn project_widget_and_update_with_input_bound_intent_actions<Action, Command>(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        input: DocumentInput,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
        Command: Into<String>,
    {
        let registry = DocumentCommandRegistry::new().bind_on_many(actions);
        self.project_widget_and_update_with_input_actions(widget, input, &registry)
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

    /// Applies widget projections, routes input, and collects typed actions.
    pub fn project_widgets_and_update_with_input_actions<'a, W, Action>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        input: DocumentInput,
        registry: &DocumentCommandRegistry<Action>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        W: DocumentWidget + ?Sized + 'a,
        Action: Clone,
    {
        let report = self.project_widgets(widgets)?;
        let frame = self.update_with_input_actions(input, registry);
        Ok((report, frame))
    }

    /// Applies widget projections, routes input, and maps command names to actions.
    pub fn project_widgets_and_update_with_input_bound_actions<'a, W, Action, Command>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        input: DocumentInput,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        W: DocumentWidget + ?Sized + 'a,
        Action: Clone,
        Command: Into<String>,
    {
        let registry = DocumentCommandRegistry::new().bind_many(actions);
        self.project_widgets_and_update_with_input_actions(widgets, input, &registry)
    }

    /// Applies widget projections, routes input, and maps intent-scoped commands.
    pub fn project_widgets_and_update_with_input_bound_intent_actions<'a, W, Action, Command>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        input: DocumentInput,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> DocumentResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        W: DocumentWidget + ?Sized + 'a,
        Action: Clone,
        Command: Into<String>,
    {
        let registry = DocumentCommandRegistry::new().bind_on_many(actions);
        self.project_widgets_and_update_with_input_actions(widgets, input, &registry)
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

    /// Extends the stylesheet used to resolve this document.
    pub fn extend_stylesheet(&mut self, stylesheet: StyleSheet) -> &mut Self {
        self.stylesheet.extend(stylesheet);
        self
    }

    /// Extends the stylesheet and returns the view.
    pub fn with_stylesheet(mut self, stylesheet: StyleSheet) -> Self {
        self.extend_stylesheet(stylesheet);
        self
    }

    /// Conditionally extends the stylesheet used to resolve this document.
    pub fn extend_stylesheet_if(&mut self, stylesheet: StyleSheet, present: bool) -> &mut Self {
        self.stylesheet.extend_if(stylesheet, present);
        self
    }

    /// Conditionally extends the stylesheet and returns the view.
    pub fn with_stylesheet_if(mut self, stylesheet: StyleSheet, present: bool) -> Self {
        self.extend_stylesheet_if(stylesheet, present);
        self
    }

    /// Parses strict CSS into the view stylesheet.
    pub fn extend_css(&mut self, css: &str) -> Result<&mut Self, crate::CssParseError> {
        self.stylesheet.extend_css(css)?;
        Ok(self)
    }

    /// Parses strict CSS into the view stylesheet and returns the view.
    pub fn with_css(mut self, css: &str) -> Result<Self, crate::CssParseError> {
        self.extend_css(css)?;
        Ok(self)
    }

    /// Conditionally parses strict CSS into the view stylesheet.
    pub fn extend_css_if(
        &mut self,
        present: bool,
        css: &str,
    ) -> Result<&mut Self, crate::CssParseError> {
        if present {
            self.stylesheet.extend_css(css)?;
        }
        Ok(self)
    }

    /// Conditionally parses strict CSS into the view stylesheet and returns the view.
    pub fn with_css_if(mut self, present: bool, css: &str) -> Result<Self, crate::CssParseError> {
        self.extend_css_if(present, css)?;
        Ok(self)
    }

    /// Parses browser-forgiving CSS into the view stylesheet.
    pub fn extend_css_forgiving(&mut self, css: &str) -> Result<&mut Self, crate::CssParseError> {
        self.stylesheet.extend_css_forgiving(css)?;
        Ok(self)
    }

    /// Parses browser-forgiving CSS into the view stylesheet and returns the view.
    pub fn with_css_forgiving(mut self, css: &str) -> Result<Self, crate::CssParseError> {
        self.extend_css_forgiving(css)?;
        Ok(self)
    }

    /// Conditionally parses browser-forgiving CSS into the view stylesheet.
    pub fn extend_css_forgiving_if(
        &mut self,
        present: bool,
        css: &str,
    ) -> Result<&mut Self, crate::CssParseError> {
        if present {
            self.stylesheet.extend_css_forgiving(css)?;
        }
        Ok(self)
    }

    /// Conditionally parses browser-forgiving CSS into the view stylesheet and returns the view.
    pub fn with_css_forgiving_if(
        mut self,
        present: bool,
        css: &str,
    ) -> Result<Self, crate::CssParseError> {
        self.extend_css_forgiving_if(present, css)?;
        Ok(self)
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

    /// Starts a fluent update request for projection, input, actions, or dispatch.
    pub fn update_request(&mut self) -> DocumentUpdateRequest<'_> {
        DocumentUpdateRequest::new(self)
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

    pub fn with(self, configure: impl FnOnce(Self) -> Self) -> Self {
        configure(self)
    }

    pub fn try_with<E>(self, configure: impl FnOnce(Self) -> Result<Self, E>) -> Result<Self, E> {
        configure(self)
    }

    pub fn stylesheet(mut self, stylesheet: StyleSheet) -> Self {
        self.stylesheet = stylesheet;
        self
    }

    pub fn extend_stylesheet(mut self, stylesheet: StyleSheet) -> Self {
        self.stylesheet.extend(stylesheet);
        self
    }

    pub fn extend_stylesheet_if(mut self, stylesheet: StyleSheet, present: bool) -> Self {
        self.stylesheet.extend_if(stylesheet, present);
        self
    }

    pub fn when(self, present: bool, configure: impl FnOnce(Self) -> Self) -> Self {
        if present { configure(self) } else { self }
    }

    pub fn try_when<E>(
        self,
        present: bool,
        configure: impl FnOnce(Self) -> Result<Self, E>,
    ) -> Result<Self, E> {
        if present { configure(self) } else { Ok(self) }
    }

    pub fn css(mut self, css: &str) -> Result<Self, crate::CssParseError> {
        self.stylesheet.extend_css(css)?;
        Ok(self)
    }

    pub fn with_css(self, css: &str) -> Result<Self, crate::CssParseError> {
        self.css(css)
    }

    pub fn css_if(mut self, present: bool, css: &str) -> Result<Self, crate::CssParseError> {
        if present {
            self.stylesheet.extend_css(css)?;
        }
        Ok(self)
    }

    pub fn css_forgiving(mut self, css: &str) -> Result<Self, crate::CssParseError> {
        self.stylesheet.extend_css_forgiving(css)?;
        Ok(self)
    }

    pub fn with_css_forgiving(self, css: &str) -> Result<Self, crate::CssParseError> {
        self.css_forgiving(css)
    }

    pub fn css_forgiving_if(
        mut self,
        present: bool,
        css: &str,
    ) -> Result<Self, crate::CssParseError> {
        if present {
            self.stylesheet.extend_css_forgiving(css)?;
        }
        Ok(self)
    }

    pub fn widget_styles(mut self, widget: &(impl DocumentWidget + ?Sized)) -> Self {
        widget.push_styles(&mut self.stylesheet);
        self
    }

    pub fn widget_styles_if(
        mut self,
        widget: &(impl DocumentWidget + ?Sized),
        present: bool,
    ) -> Self {
        if present {
            widget.push_styles(&mut self.stylesheet);
        }
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

    pub fn widget_styles_many_if<'a, W>(
        mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) -> Self
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        if present {
            for widget in widgets {
                widget.push_styles(&mut self.stylesheet);
            }
        }
        self
    }

    pub fn widget(self, widget: &(impl DocumentWidget + ?Sized)) -> DocumentView {
        self.try_widget(widget)
            .expect("document widget projection targets rendered elements")
    }

    pub fn widget_if(self, widget: &(impl DocumentWidget + ?Sized), present: bool) -> DocumentView {
        self.try_widget_if(widget, present)
            .expect("document widget projection targets rendered elements")
    }

    pub fn action_widget<Action>(
        self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
    ) -> DocumentActionSurface<Action> {
        self.try_action_widget(widget)
            .expect("document widget projection targets rendered elements")
    }

    pub fn action_widget_if<Action>(
        self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        present: bool,
    ) -> DocumentActionSurface<Action> {
        self.try_action_widget_if(widget, present)
            .expect("document widget projection targets rendered elements")
    }

    pub fn try_action_widget<Action>(
        self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
    ) -> DocumentResult<DocumentActionSurface<Action>> {
        let view = self.try_widget(widget)?;
        let commands = DocumentCommandRegistry::new().bind_widget(widget);
        Ok(DocumentActionSurface::new(view, commands))
    }

    pub fn try_action_widget_if<Action>(
        self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        present: bool,
    ) -> DocumentResult<DocumentActionSurface<Action>> {
        if present {
            self.try_action_widget(widget)
        } else {
            Ok(DocumentActionSurface::new(
                self.build(|_| {}),
                DocumentCommandRegistry::new(),
            ))
        }
    }

    pub fn try_widget(
        mut self,
        widget: &(impl DocumentWidget + ?Sized),
    ) -> DocumentResult<DocumentView> {
        widget.push_styles(&mut self.stylesheet);
        let mut view = DocumentView::new(
            Document::build(self.viewport, |ui| {
                ui.widget(widget);
            }),
            self.stylesheet,
        );
        view.project_widget(widget)?;
        Ok(view)
    }

    pub fn try_widget_if(
        self,
        widget: &(impl DocumentWidget + ?Sized),
        present: bool,
    ) -> DocumentResult<DocumentView> {
        if present {
            self.try_widget(widget)
        } else {
            Ok(self.build(|_| {}))
        }
    }

    pub fn widgets<'a, W>(self, widgets: impl IntoIterator<Item = &'a W>) -> DocumentView
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.try_widgets(widgets)
            .expect("document widget projection targets rendered elements")
    }

    pub fn widgets_if<'a, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) -> DocumentView
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.try_widgets_if(widgets, present)
            .expect("document widget projection targets rendered elements")
    }

    pub fn action_widgets<'a, Action, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
    ) -> DocumentActionSurface<Action>
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.try_action_widgets(widgets)
            .expect("document widget projection targets rendered elements")
    }

    pub fn action_widgets_if<'a, Action, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) -> DocumentActionSurface<Action>
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.try_action_widgets_if(widgets, present)
            .expect("document widget projection targets rendered elements")
    }

    pub fn try_action_widgets<'a, Action, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
    ) -> DocumentResult<DocumentActionSurface<Action>>
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        let widgets = widgets.into_iter().collect::<Vec<_>>();
        let commands = DocumentCommandRegistry::new().bind_widgets(widgets.iter().copied());
        let view = self.try_widgets(widgets)?;
        Ok(DocumentActionSurface::new(view, commands))
    }

    pub fn try_action_widgets_if<'a, Action, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) -> DocumentResult<DocumentActionSurface<Action>>
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        if present {
            self.try_action_widgets(widgets)
        } else {
            Ok(DocumentActionSurface::new(
                self.build(|_| {}),
                DocumentCommandRegistry::new(),
            ))
        }
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

    pub fn try_widgets_if<'a, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) -> DocumentResult<DocumentView>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        if present {
            self.try_widgets(widgets)
        } else {
            Ok(self.build(|_| {}))
        }
    }

    pub fn build_with_widget(
        self,
        widget: &(impl DocumentWidget + ?Sized),
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentView {
        self.try_build_with_widget(widget, build)
            .expect("document widget projection targets rendered elements")
    }

    pub fn build_with_widget_if(
        self,
        widget: &(impl DocumentWidget + ?Sized),
        present: bool,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentView {
        self.try_build_with_widget_if(widget, present, build)
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

    pub fn try_build_with_widget_if(
        self,
        widget: &(impl DocumentWidget + ?Sized),
        present: bool,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentResult<DocumentView> {
        if present {
            self.try_build_with_widget(widget, build)
        } else {
            Ok(self.build(build))
        }
    }

    pub fn build_with_action_widget<Action>(
        self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentActionSurface<Action> {
        self.try_build_with_action_widget(widget, build)
            .expect("document widget projection targets rendered elements")
    }

    pub fn build_with_action_widget_if<Action>(
        self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        present: bool,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentActionSurface<Action> {
        self.try_build_with_action_widget_if(widget, present, build)
            .expect("document widget projection targets rendered elements")
    }

    pub fn try_build_with_action_widget<Action>(
        self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentResult<DocumentActionSurface<Action>> {
        let view = self.try_build_with_widget(widget, build)?;
        let commands = DocumentCommandRegistry::new().bind_widget(widget);
        Ok(DocumentActionSurface::new(view, commands))
    }

    pub fn try_build_with_action_widget_if<Action>(
        self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        present: bool,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentResult<DocumentActionSurface<Action>> {
        if present {
            self.try_build_with_action_widget(widget, build)
        } else {
            Ok(DocumentActionSurface::new(
                self.build(build),
                DocumentCommandRegistry::new(),
            ))
        }
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

    pub fn build_with_widgets_if<'a, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentView
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        self.try_build_with_widgets_if(widgets, present, build)
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

    pub fn try_build_with_widgets_if<'a, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentResult<DocumentView>
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        if present {
            self.try_build_with_widgets(widgets, build)
        } else {
            Ok(self.build(build))
        }
    }

    pub fn build_with_action_widgets<'a, Action, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentActionSurface<Action>
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.try_build_with_action_widgets(widgets, build)
            .expect("document widget projection targets rendered elements")
    }

    pub fn build_with_action_widgets_if<'a, Action, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentActionSurface<Action>
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.try_build_with_action_widgets_if(widgets, present, build)
            .expect("document widget projection targets rendered elements")
    }

    pub fn try_build_with_action_widgets<'a, Action, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentResult<DocumentActionSurface<Action>>
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        let widgets = widgets.into_iter().collect::<Vec<_>>();
        let commands = DocumentCommandRegistry::new().bind_widgets(widgets.iter().copied());
        let view = self.try_build_with_widgets(widgets, build)?;
        Ok(DocumentActionSurface::new(view, commands))
    }

    pub fn try_build_with_action_widgets_if<'a, Action, W>(
        self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentResult<DocumentActionSurface<Action>>
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        if present {
            self.try_build_with_action_widgets(widgets, build)
        } else {
            Ok(DocumentActionSurface::new(
                self.build(build),
                DocumentCommandRegistry::new(),
            ))
        }
    }

    pub fn build_action_surface<Action>(
        self,
        commands: DocumentCommandRegistry<Action>,
        build: impl FnOnce(&mut DocumentBuilder),
    ) -> DocumentActionSurface<Action> {
        self.build(build).action_surface(commands)
    }

    pub fn build_action_surface_with<Action>(
        self,
        build: impl FnOnce(&mut DocumentBuilder),
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> DocumentActionSurface<Action> {
        self.build(build).action_surface_with(configure)
    }

    pub fn build(self, build: impl FnOnce(&mut DocumentBuilder)) -> DocumentView {
        DocumentView::new(Document::build(self.viewport, build), self.stylesheet)
    }
}
