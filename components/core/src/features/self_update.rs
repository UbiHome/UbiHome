use std::env;

/// Writes `bytes` to a sibling temp file next to the running executable and
/// atomically swaps it in via `self_replace`. Callers are responsible for
/// verifying the bytes (checksum/signature) before calling this - once called,
/// the running binary is replaced on disk.
pub fn apply_new_binary(bytes: &[u8]) -> Result<(), String> {
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;

    let mut new_exe_path = exe_path.clone();
    new_exe_path.set_file_name(format!(
        "new_{}",
        exe_path.file_name().unwrap_or_default().to_string_lossy()
    ));

    std::fs::write(&new_exe_path, bytes).map_err(|e| e.to_string())?;
    self_replace::self_replace(&new_exe_path).map_err(|e| e.to_string())?;
    std::fs::remove_file(&new_exe_path).ok();

    Ok(())
}
