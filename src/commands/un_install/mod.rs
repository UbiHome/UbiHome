use inquire::Text;
use tokio::runtime::Runtime;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(any(target_os = "linux", target_os = "macos"))]
const DEFAULT_INSTALLATION_PATH: &str = "/usr/bin/ubihome";
#[cfg(target_os = "windows")]
const DEFAULT_INSTALLATION_PATH: &str = "C:\\Program Files\\ubihome";

pub fn install(user_specified_location: Option<String>) {
    let rt = Runtime::new().unwrap();

    let location: String = user_specified_location.unwrap_or(
        Text::new("Where do you want to install UbiHome?")
            .with_default(DEFAULT_INSTALLATION_PATH)
            .prompt()
            .unwrap(),
    );

    // Spawn the root task
    rt.block_on(async {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        linux::install(&location).await;

        #[cfg(target_os = "windows")]
        windows::install(&location).await.unwrap();
    });
}

pub fn uninstall(user_specified_location: Option<String>) {
    let rt = Runtime::new().unwrap();

    // TODO: Check the default locations

    let location: String = user_specified_location.unwrap_or(
        Text::new("OS Home is not installed under the default location. Where should the uninstall script run?")
        .with_default(DEFAULT_INSTALLATION_PATH).prompt().expect("Nothing specified")
    );

    // Spawn the root task
    rt.block_on(async {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        linux::uninstall(&location).await;

        #[cfg(target_os = "windows")]
        windows::uninstall(&location).await.unwrap();
    });
}
