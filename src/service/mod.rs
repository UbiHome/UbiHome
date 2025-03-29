use tokio::runtime::Runtime;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub fn install(location: &str) {
    let rt = Runtime::new().unwrap();

    // Spawn the root task
    rt.block_on(async {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        linux::install(location).await;

        #[cfg(target_os = "windows")]
        windows::install(location).await.unwrap();
    });
}

pub fn uninstall(location: &str) {
    let rt = Runtime::new().unwrap();

    // Spawn the root task
    rt.block_on(async {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        linux::uninstall(location).await;

        #[cfg(target_os = "windows")]
        windows::uninstall(location).await.unwrap();
    });
}