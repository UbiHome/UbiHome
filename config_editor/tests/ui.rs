//! Headless UI tests (rendered with egui_kittest).
//!
//! Set `UBIHOME_EDITOR_SCREENSHOTS=<dir>` to also write rendered PNGs.

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use ubihome_config_editor::ConfigEditorApp;

const CONFIG: &str = r#"ubihome:
  name: test_device
shell:
  timeout: 5s
api:
switch:
  - platform: shell
    name: Screen
    id: screen
    command_on: echo on
    command_off: echo off
sensor:
  - platform: shell
    name: CPU Temperature
    command: cat /sys/class/thermal/thermal_zone0/temp
    unit_of_measurement: °C
    filters:
      - round: 1
"#;

fn harness(dir: &std::path::Path) -> Harness<'static, ConfigEditorApp> {
    let path = dir.join("config.yaml");
    std::fs::write(&path, CONFIG).unwrap();
    let validator: ubihome_config_editor::Validator = Box::new(|_, _| Ok(()));
    Harness::builder()
        .with_size([1100.0, 720.0])
        .build_eframe(move |cc| ConfigEditorApp::new(cc, Some(path), Some(validator)))
}

fn screenshot(harness: &mut Harness<'_, ConfigEditorApp>, name: &str) {
    if let Ok(dir) = std::env::var("UBIHOME_EDITOR_SCREENSHOTS") {
        let image = harness.render().expect("render");
        image
            .save(std::path::Path::new(&dir).join(format!("{name}.png")))
            .unwrap();
    }
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ubihome_editor_{name}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn edits_entities_and_round_trips() {
    let dir = temp_dir("entities");
    let mut harness = harness(&dir);
    harness.run();
    screenshot(&mut harness, "01_general");

    harness.get_by_label("CPU Temperature").click();
    harness.run();
    screenshot(&mut harness, "02_entity");

    harness.get_by_label("YAML").click();
    harness.run();
    screenshot(&mut harness, "03_yaml");
    harness.get_by_label("Visual").click();
    harness.run();

    harness.get_by_label("Screen").click();
    harness.run();
    harness.get_by_label("Remove").click();
    harness.run();

    let text = harness.state().text().unwrap();
    assert!(!text.contains("Screen"), "{text}");
    assert!(text.contains("CPU Temperature"), "{text}");
    assert!(
        !text.contains("switch"),
        "empty entity lists are removed: {text}"
    );

    harness.get_by_label("Save").click();
    harness.run();
    let saved = std::fs::read_to_string(dir.join("config.yaml")).unwrap();
    assert_eq!(saved, text);
}

#[test]
fn adds_entity_with_required_fields() {
    let dir = temp_dir("add");
    let mut harness = harness(&dir);
    harness.run();

    harness.get_by_label("+ Add entity").click();
    harness.run();
    screenshot(&mut harness, "04_add_entity");
}

#[test]
fn keeps_comments_until_visual_edit() {
    let dir = temp_dir("comments");
    let path = dir.join("config.yaml");
    let original = "# My device\nubihome:\n  name: test_device # inline\napi:\n";
    std::fs::write(&path, original).unwrap();
    let mut harness = Harness::builder()
        .with_size([1100.0, 720.0])
        .build_eframe(move |cc| ConfigEditorApp::new(cc, Some(path), None));
    harness.run();
    assert_eq!(harness.state().text().unwrap(), original);

    // Switching views without editing keeps the text verbatim.
    harness.get_by_label("YAML").click();
    harness.run();
    harness.get_by_label("Visual").click();
    harness.run();
    assert_eq!(harness.state().text().unwrap(), original);
}

#[test]
fn validates_automatically() {
    let dir = temp_dir("autovalidate");
    let path = dir.join("config.yaml");
    std::fs::write(&path, CONFIG).unwrap();
    let validator: ubihome_config_editor::Validator =
        Box::new(|_, _| Err("command is too dangerous".to_string()));
    let mut harness = Harness::builder()
        .with_size([1100.0, 720.0])
        .build_eframe(move |cc| ConfigEditorApp::new(cc, Some(path), Some(validator)));
    // No click needed: the report shows up once the debounce has elapsed.
    harness.run_steps(4);
    harness.get_by_label("command is too dangerous");
    harness.get_by_label("1 problem");
    screenshot(&mut harness, "05_invalid");
}
