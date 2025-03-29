use tokio::runtime::Runtime;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub fn install(location: &str) {
    let rt = Runtime::new().unwrap();

    // Spawn the root task
    rt.block_on(async {
        #[cfg(target_os = "linux")]
        linux::install(location).await;

        #[cfg(target_os = "windows")]
        windows::install(location).await;
    });
}

pub fn uninstall(location: &str) {
    #[cfg(target_os = "linux")]
    linux::uninstall(location);

    #[cfg(target_os = "windows")]
    windows::uninstall(location);
}