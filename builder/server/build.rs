//! Ensure the embedded-assets folder exists so `rust-embed` can compile even on
//! a fresh checkout where the Angular `dist/` has not been built yet (it is
//! gitignored). The real `npm run build` output overwrites this placeholder.

use std::fs;
use std::path::Path;

fn main() {
    let dir = Path::new("../frontend/dist/ubihome-builder/browser");
    let index = dir.join("index.html");
    if !index.exists() {
        let _ = fs::create_dir_all(dir);
        let _ = fs::write(
            &index,
            "<!doctype html><html><head><meta charset=\"utf-8\">\
<title>UbiHome Builder</title></head><body>\
UbiHome Builder — frontend not built. The REST API is available under /api.\
</body></html>",
        );
    }
    println!("cargo::rerun-if-changed=../frontend/dist/ubihome-builder/browser/index.html");
}
