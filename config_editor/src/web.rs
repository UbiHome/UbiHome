//! Browser specific glue.

use wasm_bindgen::{JsCast, JsValue};

const CANVAS_ID: &str = "ubihome_config_editor";

/// Starts the editor on the `<canvas id="ubihome_config_editor">` element.
pub fn start() {
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("no document");
        let canvas = document
            .get_element_by_id(CANVAS_ID)
            .expect("canvas element missing")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element is not a canvas");
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(crate::ConfigEditorApp::new(cc, None)))),
            )
            .await;
        if let Some(loading) = document.get_element_by_id("loading") {
            match result {
                Ok(()) => loading.remove(),
                Err(e) => loading.set_inner_html(&format!("The editor failed to start: {e:?}")),
            }
        }
    });
}

/// Offers `text` as a file download.
pub fn download(file_name: &str, text: &str) -> Result<(), String> {
    let err = |e: JsValue| format!("Download failed: {e:?}");
    let parts = js_sys::Array::of1(&JsValue::from_str(text));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/yaml");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options).map_err(err)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(err)?;
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("no document")?;
    let anchor = document
        .create_element("a")
        .map_err(err)?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "not an anchor".to_string())?;
    anchor.set_href(&url);
    anchor.set_download(file_name);
    anchor.click();
    web_sys::Url::revoke_object_url(&url).map_err(err)?;
    Ok(())
}
