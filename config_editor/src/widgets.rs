//! Generic, schema-assisted editors for YAML values.
//!
//! Every row uses the same three columns so forms are easy to scan: a label
//! column, a value column whose widgets all share one width, and a narrow
//! action column. Nested values are indented, but their value column stays
//! aligned with the parent's (see [`Form`]).

use egui::{vec2, Align, Id, Layout, RichText, Ui};

use crate::schema::{Field, FieldType};
use crate::yaml::{Kind, Yaml};

const LABEL_WIDTH: f32 = 190.0;
const ACTION_WIDTH: f32 = 28.0;

/// Shared column geometry of a form and everything nested in it.
#[derive(Clone, Copy)]
pub struct Form {
    /// Absolute x position where the value column starts.
    value_left: f32,
}

impl Form {
    /// Starts a form at the current cursor position.
    pub fn begin(ui: &Ui) -> Self {
        Self {
            value_left: ui.cursor().left() + LABEL_WIDTH,
        }
    }
}

fn cell<R>(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let height = ui.spacing().interact_size.y;
    ui.allocate_ui_with_layout(
        vec2(width, height),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            add(ui)
        },
    )
    .inner
}

/// One form row: label, value and action cells with fixed geometry.
pub fn row<V, A>(
    ui: &mut Ui,
    form: &Form,
    label: impl FnOnce(&mut Ui),
    value: impl FnOnce(&mut Ui) -> V,
    action: impl FnOnce(&mut Ui) -> A,
) -> (V, A) {
    ui.horizontal(|ui| {
        let spacing = ui.spacing().item_spacing.x;
        let label_width = (form.value_left - ui.cursor().left() - spacing).max(40.0);
        cell(ui, label_width, label);
        let value_width = (ui.available_width() - ACTION_WIDTH - spacing).max(40.0);
        let value = cell(ui, value_width, value);
        let action = cell(ui, ACTION_WIDTH, action);
        (value, action)
    })
    .inner
}

pub fn key_label(ui: &mut Ui, key: &str, hint: Option<&Field>) {
    let text = if hint.is_some_and(|h| h.required) {
        format!("{key} *")
    } else {
        key.to_string()
    };
    let response = ui.add(egui::Label::new(text).truncate());
    if let Some(hint) = hint {
        response.on_hover_text(hint.help);
    }
}

enum RowAction {
    Remove(usize),
    MoveUp(usize),
    MoveDown(usize),
    Convert(usize, Kind),
}

fn entry_menu(ui: &mut Ui, index: usize, kind: Kind) -> Option<RowAction> {
    let mut action = None;
    ui.menu_button("...", |ui| {
        ui.menu_button("Change type", |ui| {
            for target in Kind::ALL {
                if ui
                    .add_enabled(kind != target, egui::Button::new(target.label()))
                    .clicked()
                {
                    action = Some(RowAction::Convert(index, target));
                    ui.close();
                }
            }
        });
        if ui.button("Remove").clicked() {
            action = Some(RowAction::Remove(index));
            ui.close();
        }
    });
    action
}

fn item_menu(ui: &mut Ui, index: usize, len: usize) -> Option<RowAction> {
    let mut action = None;
    ui.menu_button("...", |ui| {
        if ui
            .add_enabled(index > 0, egui::Button::new("Move up"))
            .clicked()
        {
            action = Some(RowAction::MoveUp(index));
            ui.close();
        }
        if ui
            .add_enabled(index + 1 < len, egui::Button::new("Move down"))
            .clicked()
        {
            action = Some(RowAction::MoveDown(index));
            ui.close();
        }
        if ui.button("Remove").clicked() {
            action = Some(RowAction::Remove(index));
            ui.close();
        }
    });
    action
}

fn summary(value: &Yaml) -> String {
    let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
    match value {
        Yaml::Map(entries) => plural(entries.len(), "field"),
        Yaml::Seq(items) => plural(items.len(), "item"),
        other => other.summary(),
    }
}

/// A collapsible row for a mapping or list, with its editor as the body.
/// Returns whether the value changed, and the action chosen in the row menu.
#[allow(clippy::too_many_arguments)]
fn compound_row(
    ui: &mut Ui,
    form: &Form,
    id: Id,
    key: &str,
    value: &mut Yaml,
    label: impl FnOnce(&mut Ui),
    title: Option<String>,
    menu: impl FnOnce(&mut Ui) -> Option<RowAction>,
) -> (bool, Option<RowAction>) {
    let text = title.unwrap_or_else(|| summary(value));
    let state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        id.with("open"),
        true,
    );
    let mut changed = false;
    let (_, header, _) = state
        .show_header(ui, |ui| {
            row(
                ui,
                form,
                label,
                |ui| {
                    ui.add(egui::Label::new(RichText::new(text).weak()).truncate());
                },
                menu,
            )
            .1
        })
        .body(|ui| {
            changed = match value {
                Yaml::Map(entries) => map_form(ui, form, id, entries, nested_hints(key), &[]),
                Yaml::Seq(items) => seq_editor(ui, form, id, key, items),
                _ => false,
            };
        });
    (changed, header.inner)
}

/// Edits the entries of a mapping as form rows.
///
/// `hidden` keys are skipped (they are rendered elsewhere, e.g. `platform`).
pub fn map_form(
    ui: &mut Ui,
    form: &Form,
    id: Id,
    entries: &mut Vec<(String, Yaml)>,
    hints: &[Field],
    hidden: &[&str],
) -> bool {
    let mut changed = false;
    let mut action = None;

    for (index, (key, value)) in entries.iter_mut().enumerate() {
        if hidden.contains(&key.as_str()) {
            continue;
        }
        let hint = hints.iter().find(|f| f.key == key);
        let row_id = id.with(index).with(key.as_str());
        let kind = value.kind();
        let row_action = if matches!(kind, Kind::Map | Kind::Seq) {
            let (row_changed, row_action) = compound_row(
                ui,
                form,
                row_id,
                key,
                value,
                |ui| key_label(ui, key, hint),
                None,
                |ui| entry_menu(ui, index, kind),
            );
            changed |= row_changed;
            row_action
        } else {
            let (row_changed, row_action) = row(
                ui,
                form,
                |ui| key_label(ui, key, hint),
                |ui| scalar_editor(ui, row_id, key, value, hint),
                |ui| entry_menu(ui, index, kind),
            );
            changed |= row_changed;
            row_action
        };
        action = action.or(row_action);
    }

    match action {
        Some(RowAction::Remove(index)) => {
            entries.remove(index);
            changed = true;
        }
        Some(RowAction::Convert(index, kind)) => {
            entries[index].1 = entries[index].1.convert(kind);
            changed = true;
        }
        _ => {}
    }
    changed |= add_field(ui, form, id, entries, hints);
    changed
}

/// The "+ Add field" control: known fields from a menu, or a custom key
/// typed into the label column.
fn add_field(
    ui: &mut Ui,
    form: &Form,
    id: Id,
    entries: &mut Vec<(String, Yaml)>,
    hints: &[Field],
) -> bool {
    let mut changed = false;
    let custom_id = id.with("custom_key");
    let mut custom: Option<String> = ui.data(|d| d.get_temp(custom_id));

    if let Some(key) = &mut custom {
        let mut done = false;
        let confirmed = std::cell::Cell::new(false);
        let focus_id = custom_id.with("focus");
        row(
            ui,
            form,
            |ui| {
                let response = ui
                    .add(
                        egui::TextEdit::singleline(key)
                            .hint_text("new key")
                            .desired_width(f32::INFINITY),
                    )
                    .accessible_name("New key");
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    confirmed.set(true);
                }
                // Focus the new row once, when it appears.
                if ui.data_mut(|d| d.remove_temp::<bool>(focus_id)).is_some() {
                    response.request_focus();
                }
            },
            |ui| {
                if ui.button("Add").clicked() {
                    confirmed.set(true);
                }
                done |= ui.button("Cancel").clicked();
            },
            |_| (),
        );
        let name = key.trim().to_string();
        if confirmed.get() && !name.is_empty() && !entries.iter().any(|(k, _)| *k == name) {
            entries.push((name, Yaml::String(String::new())));
            changed = true;
            done = true;
        }
        if done {
            custom = None;
        }
    } else {
        ui.menu_button(RichText::new("+ Add field").weak(), |ui| {
            let missing: Vec<&Field> = hints
                .iter()
                .filter(|f| !entries.iter().any(|(k, _)| k == f.key))
                .collect();
            for field in missing {
                let label = if field.required {
                    format!("{} *", field.key)
                } else {
                    field.key.to_string()
                };
                if ui.button(label).on_hover_text(field.help).clicked() {
                    entries.push((field.key.to_string(), field.ty.initial(field.default)));
                    changed = true;
                    ui.close();
                }
            }
            if ui.button("Custom key...").clicked() {
                custom = Some(String::new());
                ui.data_mut(|d| d.insert_temp(custom_id.with("focus"), true));
                ui.close();
            }
        });
    }

    ui.data_mut(|d| match custom {
        Some(key) => {
            d.insert_temp(custom_id, key);
        }
        None => {
            d.remove::<String>(custom_id);
        }
    });
    changed
}

/// Edits a scalar (or empty) value; the widget always fills the value cell.
/// `name` is the accessible name (the row's label).
pub fn scalar_editor(
    ui: &mut Ui,
    id: Id,
    name: &str,
    value: &mut Yaml,
    hint: Option<&Field>,
) -> bool {
    let ty = hint.map(|h| h.ty);
    let default = hint.and_then(|h| h.default);
    let width = ui.available_width();
    let height = ui.spacing().interact_size.y;

    if let Some(FieldType::Choice(options)) = ty {
        let current = match &*value {
            Yaml::Null => String::new(),
            other => other.summary(),
        };
        let mut changed = false;
        egui::ComboBox::from_id_salt(id.with("choice"))
            .width(width)
            .selected_text(&current)
            .show_ui(ui, |ui| {
                for option in options {
                    if ui.selectable_label(current == *option, *option).clicked() {
                        *value = match (&*value, option.parse::<i64>()) {
                            (Yaml::Int(_), Ok(i)) => Yaml::Int(i),
                            _ => Yaml::String(option.to_string()),
                        };
                        changed = true;
                    }
                }
            })
            .response
            .accessible_name(name);
        return changed;
    }

    match value {
        Yaml::Bool(b) => ui.checkbox(b, "").accessible_name(name).changed(),
        Yaml::Int(i) => ui
            .add_sized([width, height], egui::DragValue::new(i))
            .accessible_name(name)
            .changed(),
        Yaml::Float(f) => ui
            .add_sized([width, height], egui::DragValue::new(f).speed(0.1))
            .accessible_name(name)
            .changed(),
        Yaml::String(s) => {
            let hint_text = match ty {
                Some(FieldType::Duration) => default.unwrap_or("e.g. 30s"),
                _ => default.unwrap_or(""),
            };
            let edit = if s.contains('\n') {
                egui::TextEdit::multiline(s).code_editor().desired_rows(3)
            } else {
                egui::TextEdit::singleline(s)
            };
            ui.add(edit.hint_text(hint_text).desired_width(f32::INFINITY))
                .accessible_name(name)
                .changed()
        }
        Yaml::Null => {
            // Typing into an empty value turns it into text.
            let mut text = String::new();
            let changed = ui
                .add(
                    egui::TextEdit::singleline(&mut text)
                        .hint_text("empty")
                        .desired_width(f32::INFINITY),
                )
                .accessible_name(name)
                .changed();
            if changed {
                *value = match ty {
                    Some(FieldType::Int) => text.parse().map(Yaml::Int).unwrap_or(Yaml::Null),
                    Some(FieldType::Float) => text.parse().map(Yaml::Float).unwrap_or(Yaml::Null),
                    _ => Yaml::String(text),
                };
            }
            changed
        }
        Yaml::Map(_) | Yaml::Seq(_) => false,
    }
}

/// Edits a list. `key` is the mapping key the list belongs to (used to offer
/// ready-made items for well-known lists such as `filters` or `then`).
pub fn seq_editor(ui: &mut Ui, form: &Form, id: Id, key: &str, items: &mut Vec<Yaml>) -> bool {
    let mut changed = false;
    let mut action = None;
    let len = items.len();

    for (index, item) in items.iter_mut().enumerate() {
        let item_id = id.with(index);
        let number = |ui: &mut Ui| {
            ui.label(RichText::new(format!("{}.", index + 1)).weak());
        };
        let row_action = if matches!(item, Yaml::Map(_) | Yaml::Seq(_)) {
            // Single-key items (actions, filters) are titled by their key.
            let title = item.as_map().and_then(|m| m.first()).map(|(k, v)| match v {
                Yaml::Map(_) | Yaml::Seq(_) | Yaml::Null => k.clone(),
                v => format!("{k}: {}", v.summary()),
            });
            let (row_changed, row_action) =
                compound_row(ui, form, item_id, key, item, number, title, |ui| {
                    item_menu(ui, index, len)
                });
            changed |= row_changed;
            row_action
        } else {
            let (row_changed, row_action) = row(
                ui,
                form,
                number,
                |ui| scalar_editor(ui, item_id, &format!("{key} {}", index + 1), item, None),
                |ui| item_menu(ui, index, len),
            );
            changed |= row_changed;
            row_action
        };
        action = action.or(row_action);
    }

    match action {
        Some(RowAction::Remove(index)) => {
            items.remove(index);
            changed = true;
        }
        Some(RowAction::MoveUp(index)) => {
            items.swap(index, index - 1);
            changed = true;
        }
        Some(RowAction::MoveDown(index)) => {
            items.swap(index, index + 1);
            changed = true;
        }
        _ => {}
    }

    ui.menu_button(RichText::new("+ Add item").weak(), |ui| {
        let templates = list_templates(key);
        let has_templates = !templates.is_empty();
        for (label, template) in templates {
            if ui.button(label).clicked() {
                items.push(template);
                changed = true;
                ui.close();
            }
        }
        if has_templates {
            ui.separator();
        }
        for kind in [Kind::String, Kind::Int, Kind::Float, Kind::Bool, Kind::Map] {
            if ui.button(kind.label()).clicked() {
                items.push(Yaml::Null.convert(kind));
                changed = true;
                ui.close();
            }
        }
    });
    changed
}

fn single(key: &str, value: Yaml) -> Yaml {
    Yaml::Map(vec![(key.to_string(), value)])
}

/// Ready-made items for well-known lists.
fn list_templates(key: &str) -> Vec<(&'static str, Yaml)> {
    let s = |v: &str| Yaml::String(v.into());
    match key {
        "then" => vec![
            ("switch.turn_on", single("switch.turn_on", s(""))),
            ("switch.turn_off", single("switch.turn_off", s(""))),
            ("button.press", single("button.press", s(""))),
            ("delay", single("delay", s("1s"))),
            ("logger.log", single("logger.log", s(""))),
            (
                "globals.set",
                single(
                    "globals.set",
                    Yaml::Map(vec![("id".into(), s("")), ("value".into(), s(""))]),
                ),
            ),
        ],
        "filters" => vec![
            ("round (sensor)", single("round", Yaml::Int(1))),
            ("invert (binary sensor)", single("invert", Yaml::Null)),
            ("delayed_on (binary sensor)", single("delayed_on", s("1s"))),
            (
                "delayed_off (binary sensor)",
                single("delayed_off", s("1s")),
            ),
        ],
        "globals" => vec![(
            "global variable",
            Yaml::Map(vec![
                ("id".into(), s("my_global")),
                ("type".into(), s("bool")),
                ("initial_value".into(), Yaml::Bool(false)),
            ]),
        )],
        "targets" => vec![(
            "target",
            Yaml::Map(vec![
                ("host".into(), s("1.1.1.1")),
                ("protocol".into(), s("dns")),
            ]),
        )],
        _ => vec![],
    }
}

/// Field hints for well-known nested mappings.
fn nested_hints(key: &str) -> &'static [Field] {
    use crate::schema::{BASE_ENTITY_FIELDS, GLOBAL_FIELDS};
    const TRIGGER: &[Field] = &[Field {
        key: "then",
        ty: FieldType::List,
        required: true,
        default: None,
        help: "Actions to run.",
    }];
    const ENCRYPTION: &[Field] = &[Field {
        key: "key",
        ty: FieldType::Text,
        required: false,
        default: None,
        help: "32 byte base64 encoded key.",
    }];
    match key {
        k if k.starts_with("on_") || k.ends_with("_action") => TRIGGER,
        "encryption" => ENCRYPTION,
        "globals" => GLOBAL_FIELDS,
        "temperature" | "pressure" | "humidity" => &BASE_ENTITY_FIELDS[1..],
        _ => &[],
    }
}
