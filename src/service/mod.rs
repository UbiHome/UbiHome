#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub async fn install(location: &str) {
    #[cfg(target_os = "linux")]
    linux::install(location).await;

    #[cfg(target_os = "windows")]
    windows::install(location).await;
}

pub async fn uninstall(location: &str) {
    #[cfg(target_os = "linux")]
    linux::uninstall(location).await;

    #[cfg(target_os = "windows")]
    windows::uninstall(location).await;
}