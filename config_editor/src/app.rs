use egui::util::undoer::Undoer;
use egui::{Align, Color32, Id, Layout, RichText, Ui};

use crate::schema::{self, ENTITY_KINDS, PLATFORMS};
use crate::validate::{self, Issue, Severity};
use crate::widgets::{self, Form};
use crate::yaml::Yaml;
use crate::Validator;

const OK_COLOR: Color32 = Color32::from_rgb(80, 170, 80);
/// Seconds to wait after the last change before running the full validation.
const VALIDATION_DELAY: f64 = 0.3;
const NEW_CONFIG: &str = "ubihome:\n  name: my_device\n";
#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "ubihome_config";

#[derive(Clone, Debug, PartialEq)]
enum Selection {
    Section(String),
    Entity { kind: String, index: usize },
    NewEntity,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Visual,
    Yaml,
}

struct NewEntity {
    kind: String,
    platform: String,
    name: String,
}

#[cfg(target_arch = "wasm32")]
type Inbox = std::rc::Rc<std::cell::RefCell<Option<(String, String)>>>;

pub struct ConfigEditorApp {
    doc: Yaml,
    /// Raw text shown in YAML mode (authoritative while in that mode).
    raw: String,
    mode: Mode,
    selection: Selection,
    #[cfg(not(target_arch = "wasm32"))]
    path: Option<std::path::PathBuf>,
    file_name: String,
    dirty: bool,
    /// The loaded text contained comments, which visual edits drop.
    had_comments: bool,
    /// `raw` still matches `doc` (no visual edits since it was loaded or
    /// last edited as text), so it can be saved verbatim, keeping comments
    /// and formatting.
    raw_in_sync: bool,
    /// The document `raw` represents.
    synced_doc: Yaml,
    issues: Vec<Issue>,
    parse_error: Option<String>,
    full_report: Option<Result<(), String>>,
    status: Option<(String, bool)>,
    validator: Option<Validator>,
    undoer: Undoer<Yaml>,
    new_entity: NewEntity,
    /// Name being typed for a platform without schema hints.
    custom_platform: Option<String>,
    validation_pending: bool,
    pending_since: Option<f64>,
    #[cfg(target_arch = "wasm32")]
    inbox: Inbox,
    #[cfg(not(target_arch = "wasm32"))]
    window_title: String,
}

impl ConfigEditorApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        path: Option<std::path::PathBuf>,
        validator: Option<Validator>,
    ) -> Self {
        let mut app = Self::empty(validator);
        match path {
            Some(path) => {
                let text = if path.exists() {
                    std::fs::read_to_string(&path).unwrap_or_else(|e| {
                        app.status =
                            Some((format!("Failed to read {}: {e}", path.display()), true));
                        String::new()
                    })
                } else {
                    app.status = Some((
                        format!(
                            "{} does not exist yet; it will be created on save.",
                            path.display()
                        ),
                        false,
                    ));
                    NEW_CONFIG.to_string()
                };
                app.file_name = display_name(&path);
                app.path = Some(path);
                app.load_text(text);
            }
            None => app.load_text(NEW_CONFIG.to_string()),
        }
        app
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(cc: &eframe::CreationContext<'_>, validator: Option<Validator>) -> Self {
        let mut app = Self::empty(validator);
        let stored = cc.storage.and_then(|s| s.get_string(STORAGE_KEY));
        app.load_text(stored.unwrap_or_else(|| NEW_CONFIG.to_string()));
        app
    }

    fn empty(validator: Option<Validator>) -> Self {
        Self {
            doc: Yaml::default(),
            raw: String::new(),
            mode: Mode::Visual,
            selection: Selection::Section("ubihome".into()),
            #[cfg(not(target_arch = "wasm32"))]
            path: None,
            file_name: "config.yaml".into(),
            dirty: false,
            had_comments: false,
            raw_in_sync: true,
            synced_doc: Yaml::default(),
            issues: Vec::new(),
            parse_error: None,
            full_report: None,
            status: None,
            validator,
            undoer: Undoer::default(),
            new_entity: NewEntity {
                kind: "sensor".into(),
                platform: String::new(),
                name: String::new(),
            },
            custom_platform: None,
            validation_pending: false,
            pending_since: None,
            #[cfg(target_arch = "wasm32")]
            inbox: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            window_title: String::new(),
        }
    }

    /// Replaces the document with `text`. Falls back to YAML mode if the
    /// text doesn't parse, so nothing is lost.
    fn load_text(&mut self, text: String) {
        self.had_comments = text.lines().any(|l| l.trim_start().starts_with('#'));
        self.raw = text;
        self.raw_in_sync = true;
        self.dirty = false;
        self.full_report = None;
        self.schedule_validation();
        self.undoer = Undoer::default();
        match Yaml::parse(&self.raw) {
            Ok(doc) => {
                self.synced_doc = doc.clone();
                self.doc = doc;
                self.parse_error = None;
                self.mode = Mode::Visual;
            }
            Err(e) => {
                self.parse_error = Some(e);
                self.mode = Mode::Yaml;
            }
        }
        self.selection = Selection::Section("ubihome".into());
        self.revalidate();
    }

    /// The configuration as it would be saved.
    pub fn text(&self) -> Result<String, String> {
        self.current_text()
    }

    fn current_text(&self) -> Result<String, String> {
        if self.mode == Mode::Yaml || self.raw_in_sync {
            Ok(self.raw.clone())
        } else {
            self.doc.to_text()
        }
    }

    fn revalidate(&mut self) {
        match self.mode {
            Mode::Visual => {
                self.issues = validate::check(&self.doc);
            }
            Mode::Yaml => match Yaml::parse(&self.raw) {
                Ok(doc) => {
                    self.parse_error = None;
                    self.issues = validate::check(&doc);
                }
                Err(e) => {
                    self.parse_error = Some(e);
                    self.issues.clear();
                }
            },
        }
    }

    /// Requests a full validation once edits settle (see `auto_validate`).
    /// The previous report stays visible until then to avoid flicker.
    fn schedule_validation(&mut self) {
        self.validation_pending = true;
        self.pending_since = None;
    }

    fn mark_changed(&mut self) {
        if self.mode == Mode::Visual {
            // Undoing back to the loaded state keeps the original text.
            self.raw_in_sync = self.doc == self.synced_doc;
        }
        self.dirty = true;
        self.schedule_validation();
        self.revalidate();
    }

    fn set_mode(&mut self, mode: Mode) {
        if mode == self.mode {
            return;
        }
        match mode {
            Mode::Yaml => match self.current_text() {
                Ok(text) => {
                    self.raw = text;
                    self.raw_in_sync = true;
                    self.synced_doc = self.doc.clone();
                    self.mode = Mode::Yaml;
                }
                Err(e) => self.status = Some((format!("Cannot render YAML: {e}"), true)),
            },
            Mode::Visual => match Yaml::parse(&self.raw) {
                Ok(doc) => {
                    self.synced_doc = doc.clone();
                    self.doc = doc;
                    self.mode = Mode::Visual;
                    self.parse_error = None;
                }
                Err(e) => {
                    self.status = Some(("Fix the YAML syntax error first.".into(), true));
                    self.parse_error = Some(e);
                }
            },
        }
        self.revalidate();
    }

    fn run_full_validation(&mut self) {
        let Some(validator) = &self.validator else {
            return;
        };
        match self.current_text() {
            Ok(text) => {
                self.full_report =
                    Some(validator(&text, &self.file_name).map_err(|e| strip_ansi(&e)))
            }
            Err(e) => self.status = Some((e, true)),
        }
    }

    // ----- File handling -------------------------------------------------

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&mut self, save_as: bool) {
        let path = match (&self.path, save_as) {
            (Some(path), false) => path.clone(),
            _ => match rfd::FileDialog::new()
                .add_filter("YAML", &["yaml", "yml"])
                .set_file_name(&self.file_name)
                .save_file()
            {
                Some(path) => path,
                None => return,
            },
        };
        let text = match self.current_text() {
            Ok(text) => text,
            Err(e) => {
                self.status = Some((e, true));
                return;
            }
        };
        match std::fs::write(&path, text) {
            Ok(()) => {
                self.dirty = false;
                self.file_name = display_name(&path);
                let errors = self
                    .issues
                    .iter()
                    .filter(|i| i.severity == Severity::Error)
                    .count();
                self.status = Some(if errors > 0 {
                    (
                        format!("Saved {} ({errors} problem(s) remaining).", path.display()),
                        true,
                    )
                } else {
                    (format!("Saved {}.", path.display()), false)
                });
                self.path = Some(path);
            }
            Err(e) => self.status = Some((format!("Failed to save: {e}"), true)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    self.file_name = display_name(&path);
                    self.path = Some(path);
                    self.load_text(text);
                    self.status = None;
                }
                Err(e) => self.status = Some((format!("Failed to open: {e}"), true)),
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn open(&mut self, ctx: &egui::Context) {
        let inbox = self.inbox.clone();
        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("YAML", &["yaml", "yml"])
                .pick_file()
                .await
            {
                let bytes = file.read().await;
                let text = String::from_utf8_lossy(&bytes).into_owned();
                *inbox.borrow_mut() = Some((file.file_name(), text));
                ctx.request_repaint();
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn download(&mut self) {
        let result = self
            .current_text()
            .and_then(|text| crate::web::download(&self.file_name, &text));
        match result {
            Ok(()) => {
                self.dirty = false;
                self.status = Some((format!("Downloaded {}.", self.file_name), false));
            }
            Err(e) => self.status = Some((e, true)),
        }
    }

    // ----- UI ------------------------------------------------------------

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.strong(format!(
                "{}{}",
                self.file_name,
                if self.dirty { " *" } else { "" }
            ));
            ui.separator();

            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Open").clicked() {
                    self.open();
                }
                if ui.button("Save").on_hover_text("Ctrl+S").clicked() {
                    self.save(false);
                }
                if ui.button("Save as").clicked() {
                    self.save(true);
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                if ui.button("Open").clicked() {
                    self.open(ui.ctx());
                }
                if ui.button("Download").clicked() {
                    self.download();
                }
                if ui.button("New").clicked() {
                    self.load_text(NEW_CONFIG.to_string());
                }
            }

            ui.separator();
            let visual = self.mode == Mode::Visual;
            if ui.selectable_label(visual, "Visual").clicked() {
                self.set_mode(Mode::Visual);
            }
            if ui.selectable_label(!visual, "YAML").clicked() {
                self.set_mode(Mode::Yaml);
            }

            if visual {
                ui.separator();
                if ui
                    .add_enabled(self.undoer.has_undo(&self.doc), egui::Button::new("Undo"))
                    .on_hover_text("Ctrl+Z")
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(self.undoer.has_redo(&self.doc), egui::Button::new("Redo"))
                    .on_hover_text("Ctrl+Shift+Z")
                    .clicked()
                {
                    self.redo();
                }
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let (text, color) = self.validation_summary(ui);
                ui.colored_label(color, text);
            });
        });
    }

    /// One-line validation state shown in the toolbar.
    fn validation_summary(&self, ui: &Ui) -> (String, Color32) {
        let error_color = ui.visuals().error_fg_color;
        if self.parse_error.is_some() {
            return ("YAML syntax error".into(), error_color);
        }
        let errors = self.error_count();
        let warnings = self.issues.len() - errors;
        let full_failed = matches!(self.full_report, Some(Err(_)));
        if errors > 0 || full_failed {
            let count = errors.max(1);
            let plural = if count == 1 { "" } else { "s" };
            return (format!("{count} problem{plural}"), error_color);
        }
        if warnings > 0 {
            let plural = if warnings == 1 { "" } else { "s" };
            return (
                format!("{warnings} warning{plural}"),
                ui.visuals().warn_fg_color,
            );
        }
        if self.validation_pending && self.validator.is_some() {
            return ("Checking...".into(), ui.visuals().weak_text_color());
        }
        ("Valid".into(), OK_COLOR)
    }

    fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count()
    }

    /// The report of the full validation, unless the quick checks already
    /// explain the problem.
    fn full_report_error(&self) -> Option<&str> {
        match &self.full_report {
            Some(Err(report)) if self.error_count() == 0 && self.parse_error.is_none() => {
                Some(report)
            }
            _ => None,
        }
    }

    fn has_messages(&self) -> bool {
        self.status.is_some()
            || self.parse_error.is_some()
            || !self.issues.is_empty()
            || self.full_report_error().is_some()
    }

    fn undo(&mut self) {
        if let Some(state) = self.undoer.undo(&self.doc) {
            self.doc = state.clone();
            self.mark_changed();
        }
    }

    fn redo(&mut self) {
        if let Some(state) = self.undoer.redo(&self.doc) {
            self.doc = state.clone();
            self.mark_changed();
        }
    }

    fn messages_panel(&mut self, ui: &mut Ui) {
        let error_color = ui.visuals().error_fg_color;
        if let Some((message, error)) = self.status.clone() {
            let color = if error {
                error_color
            } else {
                ui.visuals().text_color()
            };
            ui.horizontal(|ui| {
                ui.colored_label(color, message);
                if ui.small_button("Dismiss").clicked() {
                    self.status = None;
                }
            });
        }

        let mut jump = None;
        egui::ScrollArea::vertical()
            .id_salt("messages")
            .max_height(160.0)
            .show(ui, |ui| {
                if let Some(error) = &self.parse_error {
                    ui.colored_label(error_color, format!("YAML syntax error: {error}"));
                }
                for issue in &self.issues {
                    let color = match issue.severity {
                        Severity::Error => error_color,
                        Severity::Warning => ui.visuals().warn_fg_color,
                    };
                    let text = format!("{}: {}", issue.path, issue.message);
                    if ui
                        .add(
                            egui::Label::new(RichText::new(text).color(color))
                                .sense(egui::Sense::click()),
                        )
                        .on_hover_text("Go to")
                        .clicked()
                    {
                        jump = selection_for_path(&issue.path);
                    }
                }
                if let Some(report) = self.full_report_error() {
                    ui.label(RichText::new(report).monospace().color(error_color));
                }
            });
        if let Some(selection) = jump {
            if self.mode == Mode::Visual {
                self.selection = selection;
            }
        }
    }

    fn sidebar(&mut self, ui: &mut Ui) {
        let mut add_section: Option<String> = None;
        let configured = validate::configured_platforms(&self.doc);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Justified, so every entry and button spans the full width.
            ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                ui.label(RichText::new("Sections").weak());
                let sections = ["ubihome", "logger", "globals"]
                    .into_iter()
                    .filter(|s| self.doc.get(s).is_some())
                    .map(str::to_string)
                    .chain(configured.iter().cloned());
                for section in sections.collect::<Vec<_>>() {
                    let selected = self.selection == Selection::Section(section.clone());
                    let response = ui.selectable_label(selected, &section);
                    let response = match schema::platform(&section) {
                        Some(info) => response.on_hover_text(info.description),
                        None => response,
                    };
                    if response.clicked() {
                        self.selection = Selection::Section(section);
                    }
                }

                if let Some(name) = &mut self.custom_platform {
                    let response = ui
                        .add(
                            egui::TextEdit::singleline(name)
                                .hint_text("platform name, then Enter")
                                .desired_width(f32::INFINITY),
                        )
                        .accessible_name("Platform name");
                    if response.lost_focus() {
                        let name = name.trim().to_string();
                        let valid = !name.is_empty()
                            && self.doc.get(&name).is_none()
                            && !schema::is_reserved_section(&name);
                        if valid && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            add_section = Some(name);
                        }
                        self.custom_platform = None;
                    } else if !response.has_focus() {
                        response.request_focus();
                    }
                } else {
                    ui.menu_button("+ Add section", |ui| {
                        for section in ["logger", "globals"] {
                            if self.doc.get(section).is_none() && ui.button(section).clicked() {
                                add_section = Some(section.into());
                                ui.close();
                            }
                        }
                        ui.separator();
                        for info in PLATFORMS.iter().filter(|p| p.has_section) {
                            if configured.iter().any(|c| c == info.name) {
                                continue;
                            }
                            if ui
                                .button(info.name)
                                .on_hover_text(info.description)
                                .clicked()
                            {
                                add_section = Some(info.name.into());
                                ui.close();
                            }
                        }
                        ui.separator();
                        if ui.button("Other platform...").clicked() {
                            self.custom_platform = Some(String::new());
                            ui.close();
                        }
                    });
                }

                ui.add_space(12.0);
                ui.label(RichText::new("Entities").weak());
                for kind in ENTITY_KINDS {
                    let Some(entities) = self.doc.get(kind.key).and_then(Yaml::as_seq) else {
                        continue;
                    };
                    egui::CollapsingHeader::new(format!("{} ({})", kind.label, entities.len()))
                        .id_salt(kind.key)
                        .default_open(true)
                        .show(ui, |ui| {
                            for (index, entity) in entities.iter().enumerate() {
                                let selection = Selection::Entity {
                                    kind: kind.key.into(),
                                    index,
                                };
                                let platform =
                                    entity.get("platform").and_then(Yaml::as_str).unwrap_or("?");
                                if ui
                                    .selectable_label(
                                        self.selection == selection,
                                        entity_label(entity, index),
                                    )
                                    .on_hover_text(format!("platform: {platform}"))
                                    .clicked()
                                {
                                    self.selection = selection;
                                }
                            }
                        });
                }
                if ui
                    .selectable_label(self.selection == Selection::NewEntity, "+ Add entity")
                    .clicked()
                {
                    self.selection = Selection::NewEntity;
                }
            });
        });

        if let Some(section) = add_section {
            self.insert_section(&section);
        }
    }

    /// Adds a top-level section, keeping platform sections above the
    /// entity lists.
    fn insert_section(&mut self, section: &str) {
        let value = match section {
            "globals" => Yaml::Seq(Vec::new()),
            other => schema::new_section(other),
        };
        let Some(map) = self.doc.as_map_mut() else {
            return;
        };
        let position = map
            .iter()
            .position(|(k, _)| schema::entity_kind(k).is_some())
            .unwrap_or(map.len());
        map.insert(position, (section.to_string(), value));
        self.selection = Selection::Section(section.into());
        self.mark_changed();
    }

    fn central(&mut self, ui: &mut Ui) {
        if self.mode == Mode::Yaml {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let response = ui
                        .add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(&mut self.raw).code_editor(),
                        )
                        .accessible_name("YAML");
                    if response.changed() {
                        self.mark_changed();
                    }
                });
            return;
        }

        if self.had_comments && !self.raw_in_sync {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    "Visual edits rewrite the file without its comments. Undo, or edit in the YAML view to keep them.",
                );
                if ui.small_button("Dismiss").clicked() {
                    self.had_comments = false;
                }
            });
            ui.separator();
        }

        let changed = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.selection.clone() {
                Selection::Section(section) => self.section_editor(ui, &section),
                Selection::Entity { kind, index } => self.entity_editor(ui, &kind, index),
                Selection::NewEntity => self.new_entity_form(ui),
            })
            .inner;
        if changed {
            self.mark_changed();
        }
    }

    /// Title row shared by all editors: heading, a weak subtitle, and
    /// right-aligned actions. Returns the label of the clicked action.
    fn header(
        ui: &mut Ui,
        title: &str,
        subtitle: &str,
        docs: Option<String>,
        actions: &[&'static str],
    ) -> Option<&'static str> {
        let mut clicked = None;
        ui.horizontal(|ui| {
            ui.heading(title);
            ui.label(RichText::new(subtitle).weak());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                for action in actions.iter().rev() {
                    if ui.button(*action).clicked() {
                        clicked = Some(*action);
                    }
                }
                if let Some(url) = docs {
                    ui.hyperlink_to("Documentation", url);
                }
            });
        });
        ui.separator();
        clicked
    }

    fn section_editor(&mut self, ui: &mut Ui, section: &str) -> bool {
        let Some(value) = self.doc.get_mut(section) else {
            ui.label("Select a section or entity on the left.");
            return false;
        };
        let subtitle = schema::platform(section).map_or("", |p| p.description);
        let actions: &[&'static str] = if section == "ubihome" {
            &[]
        } else {
            &["Remove"]
        };
        let remove =
            Self::header(ui, section, subtitle, schema::docs_url(section), actions).is_some();

        let mut changed = false;
        let id = Id::unique("section").with(section);
        let form = Form::begin(ui);
        if section == "globals" {
            if let Yaml::Seq(items) = value {
                changed |= widgets::seq_editor(ui, &form, id, "globals", items);
            }
        } else {
            // Keep `section:` (null) as-is until a field is actually added,
            // because `{}` and null aren't always equivalent (e.g. optional
            // sections with required fields).
            let mut working = match &*value {
                Yaml::Null => Yaml::Map(Vec::new()),
                other => other.clone(),
            };
            if let Yaml::Map(entries) = &mut working {
                if entries.is_empty() {
                    ui.label(RichText::new("No options set; defaults are used.").weak());
                }
                let hints = schema::section_fields(section);
                if widgets::map_form(ui, &form, id, entries, hints, &[]) {
                    *value = working;
                    changed = true;
                }
            } else {
                ui.label("This section is not a mapping; edit it in the YAML view.");
            }
        }

        if remove {
            self.doc.remove(section);
            self.selection = Selection::Section("ubihome".into());
            changed = true;
        }
        changed
    }

    fn entity_editor(&mut self, ui: &mut Ui, kind: &str, index: usize) -> bool {
        let configured = validate::configured_platforms(&self.doc);
        let Some(Yaml::Seq(entities)) = self.doc.get_mut(kind) else {
            self.selection = Selection::Section("ubihome".into());
            return false;
        };
        let Some(entity) = entities.get_mut(index) else {
            self.selection = Selection::Section("ubihome".into());
            return false;
        };

        let current = entity
            .get("platform")
            .and_then(Yaml::as_str)
            .unwrap_or("")
            .to_string();
        let action = Self::header(
            ui,
            &entity_label(entity, index),
            kind,
            schema::docs_url(&current),
            &["Duplicate", "Remove"],
        );

        let Some(entries) = entity.as_map_mut() else {
            ui.label("This entity is not a mapping; edit it in the YAML view.");
            return false;
        };

        let mut changed = false;
        let mut candidates: Vec<String> = configured
            .iter()
            .filter(|p| schema::platform(p).is_none_or(|i| i.entity_kinds.contains(&kind)))
            .cloned()
            .collect();
        for builtin in schema::BUILTIN_ENTITY_PLATFORMS {
            if schema::platform(builtin).is_some_and(|i| i.entity_kinds.contains(&kind)) {
                candidates.push(builtin.to_string());
            }
        }
        if !current.is_empty() && !candidates.contains(&current) {
            candidates.push(current.clone());
        }

        let form = Form::begin(ui);
        widgets::row(
            ui,
            &form,
            |ui| widgets::key_label(ui, "platform *", None),
            |ui| {
                egui::ComboBox::from_id_salt(("entity_platform", kind, index))
                    .width(ui.available_width())
                    .selected_text(&current)
                    .show_ui(ui, |ui| {
                        for candidate in &candidates {
                            if ui
                                .selectable_label(*candidate == current, candidate)
                                .clicked()
                            {
                                let value = Yaml::String(candidate.clone());
                                match entries.iter_mut().find(|(k, _)| k == "platform") {
                                    Some(entry) => entry.1 = value,
                                    None => entries.insert(0, ("platform".into(), value)),
                                }
                                changed = true;
                            }
                        }
                    })
                    .response
                    .accessible_name("Platform");
            },
            |_| (),
        );

        let platform = (!current.is_empty()).then_some(current.as_str());
        let hints = schema::entity_fields(kind, platform);
        changed |= widgets::map_form(
            ui,
            &form,
            Id::unique(("entity", kind, index)),
            entries,
            &hints,
            &["platform"],
        );

        match action {
            Some("Duplicate") => {
                let mut copy = entity.clone();
                for key in ["id", "name"] {
                    if let Some(Yaml::String(value)) = copy.get_mut(key) {
                        value.push_str(if key == "id" { "_copy" } else { " (copy)" });
                    }
                }
                entities.insert(index + 1, copy);
                self.selection = Selection::Entity {
                    kind: kind.into(),
                    index: index + 1,
                };
                changed = true;
            }
            Some("Remove") => {
                entities.remove(index);
                if entities.is_empty() {
                    self.doc.remove(kind);
                    self.selection = Selection::Section("ubihome".into());
                } else {
                    self.selection = Selection::Entity {
                        kind: kind.into(),
                        index: index.saturating_sub(1),
                    };
                }
                changed = true;
            }
            _ => {}
        }
        changed
    }

    fn new_entity_form(&mut self, ui: &mut Ui) -> bool {
        Self::header(ui, "Add entity", "", None, &[]);
        let configured = validate::configured_platforms(&self.doc);
        let form = Form::begin(ui);
        let new = &mut self.new_entity;

        widgets::row(
            ui,
            &form,
            |ui| widgets::key_label(ui, "kind", None),
            |ui| {
                egui::ComboBox::from_id_salt("new_entity_kind")
                    .width(ui.available_width())
                    .selected_text(&new.kind)
                    .show_ui(ui, |ui| {
                        for kind in ENTITY_KINDS {
                            ui.selectable_value(&mut new.kind, kind.key.to_string(), kind.key);
                        }
                    })
                    .response
                    .accessible_name("Kind");
            },
            |_| (),
        );
        widgets::row(
            ui,
            &form,
            |ui| widgets::key_label(ui, "platform", None),
            |ui| {
                egui::ComboBox::from_id_salt("new_entity_platform")
                    .width(ui.available_width())
                    .selected_text(&new.platform)
                    .show_ui(ui, |ui| {
                        for info in PLATFORMS
                            .iter()
                            .filter(|p| p.entity_kinds.contains(&new.kind.as_str()))
                        {
                            let missing =
                                info.has_section && !configured.iter().any(|c| c == info.name);
                            let label = if missing {
                                format!("{} (adds section)", info.name)
                            } else {
                                info.name.to_string()
                            };
                            ui.selectable_value(&mut new.platform, info.name.to_string(), label)
                                .on_hover_text(info.description);
                        }
                        for other in configured.iter().filter(|p| schema::platform(p).is_none()) {
                            ui.selectable_value(&mut new.platform, other.clone(), other);
                        }
                    })
                    .response
                    .accessible_name("Platform");
            },
            |_| (),
        );
        widgets::row(
            ui,
            &form,
            |ui| widgets::key_label(ui, "name", None),
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut new.name)
                        .hint_text("at least 3 characters")
                        .desired_width(f32::INFINITY),
                )
                .accessible_name("Name");
            },
            |_| (),
        );
        let valid = !new.platform.is_empty() && new.name.trim().chars().count() >= 3;
        let (create, _) = widgets::row(
            ui,
            &form,
            |_| (),
            |ui| ui.add_enabled(valid, egui::Button::new("Create")).clicked(),
            |_| (),
        );
        if !create {
            return false;
        }

        let kind = new.kind.clone();
        let platform = new.platform.clone();
        let entity = schema::new_entity(&kind, &platform, new.name.trim());
        new.name.clear();

        let needs_section = schema::platform(&platform).is_some_and(|p| p.has_section)
            && !configured.contains(&platform);
        if needs_section {
            self.insert_section(&platform);
        }
        if self.doc.get(&kind).and_then(Yaml::as_seq).is_none() {
            self.doc.set(&kind, Yaml::Seq(Vec::new()));
        }
        let Some(Yaml::Seq(entities)) = self.doc.get_mut(&kind) else {
            return false;
        };
        entities.push(entity);
        self.selection = Selection::Entity {
            index: entities.len() - 1,
            kind,
        };
        true
    }

    /// Runs the full validation shortly after the last change, so typing
    /// stays responsive and the result is always up to date.
    fn auto_validate(&mut self, ui: &Ui) {
        if !self.validation_pending || self.validator.is_none() {
            return;
        }
        let now = ui.input(|i| i.time);
        let since = *self.pending_since.get_or_insert(now);
        let remaining = VALIDATION_DELAY - (now - since);
        if remaining > 0.0 {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(remaining));
            return;
        }
        self.validation_pending = false;
        self.pending_since = None;
        if self.parse_error.is_none() {
            self.run_full_validation();
        }
    }
}

impl eframe::App for ConfigEditorApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        #[cfg(target_arch = "wasm32")]
        {
            let opened = self.inbox.borrow_mut().take();
            if let Some((name, text)) = opened {
                self.file_name = name;
                self.load_text(text);
            }
        }

        // Keyboard shortcuts.
        let (save, undo, redo) = ui.input_mut(|i| {
            let save = i.consume_key(egui::Modifiers::COMMAND, egui::Key::S);
            let redo = i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::Z,
            );
            let undo = i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z);
            (save, undo, redo)
        });
        if save {
            #[cfg(not(target_arch = "wasm32"))]
            self.save(false);
            #[cfg(target_arch = "wasm32")]
            self.download();
        }
        if self.mode == Mode::Visual && !ui.ctx().egui_wants_keyboard_input() {
            if undo {
                self.undo();
            }
            if redo {
                self.redo();
            }
        }

        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.add_space(4.0);
            self.toolbar(ui);
            ui.add_space(2.0);
        });
        if self.has_messages() {
            egui::Panel::bottom("messages")
                .resizable(false)
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    self.messages_panel(ui);
                    ui.add_space(2.0);
                });
        }
        if self.mode == Mode::Visual {
            egui::Panel::left("tree")
                .resizable(true)
                .default_size(220.0)
                .show(ui, |ui| self.sidebar(ui));
        }
        egui::CentralPanel::default().show(ui, |ui| self.central(ui));

        if self.mode == Mode::Visual {
            let time = ui.input(|i| i.time);
            self.undoer.feed_state(time, &self.doc);
        }
        self.auto_validate(ui);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let title = format!(
                "{}{} - UbiHome Configuration",
                self.file_name,
                if self.dirty { " *" } else { "" },
            );
            // Only send on change: viewport commands trigger a repaint.
            if title != self.window_title {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
                self.window_title = title;
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(text) = self.current_text() {
            storage.set_string(STORAGE_KEY, text);
        }
    }
}

fn entity_label(entity: &Yaml, index: usize) -> String {
    entity
        .get("name")
        .or_else(|| entity.get("id"))
        .and_then(Yaml::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("#{}", index + 1))
}

/// Maps an issue path like `sensor[2].platform` to the item to select.
fn selection_for_path(path: &str) -> Option<Selection> {
    let head = path.split('.').next()?;
    if let Some((kind, rest)) = head.split_once('[') {
        let index = rest.trim_end_matches(']').parse().ok()?;
        return Some(Selection::Entity {
            kind: kind.to_string(),
            index,
        });
    }
    (!head.is_empty()).then(|| Selection::Section(head.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn display_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Removes ANSI escape sequences (miette reports may be colored).
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_paths_map_to_selections() {
        assert_eq!(
            selection_for_path("sensor[2].platform"),
            Some(Selection::Entity {
                kind: "sensor".into(),
                index: 2
            })
        );
        assert_eq!(
            selection_for_path("ubihome.name"),
            Some(Selection::Section("ubihome".into()))
        );
    }

    #[test]
    fn strips_ansi_codes() {
        assert_eq!(strip_ansi("\u{1b}[31merror\u{1b}[0m!"), "error!");
    }
}
