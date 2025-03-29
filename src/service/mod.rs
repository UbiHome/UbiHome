#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub fn install(location: &str) {
    #[cfg(target_os = "linux")]
    linux::install(location);

    #[cfg(target_os = "windows")]
    windows::install(location);
}

pub fn uninstall(location: &str) {
    #[cfg(target_os = "linux")]
    linux::uninstall(location);

    #[cfg(target_os = "windows")]
    windows::uninstall(location);
}