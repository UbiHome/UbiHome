use serde::Deserialize;
use std::collections::BTreeSet;
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::Module;
use ubihome_core::{ChangedMessage, PublishedMessage};

macro_rules! generate_component_methods {
    (
        $(($variant:ident, $platform_name:literal, $module_path:ident, $type_name:ident)),* $(,)?
    ) => {
        // Generate the Platform enum
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Ord)]
        #[serde(rename_all = "lowercase")]
        #[derive(PartialOrd)]
        pub enum Platform {
            $(
                $variant,
            )*
        }

        impl Platform {
            /// Parse a platform from its string representation
            pub fn from_str(s: &str) -> Result<Self, String> {
                if crate::config::is_base_entity_property(s) {
                    return Err(format!(
                        "Reserved platform name: \nThe component '{}' is trying to use a reserved name. Please contact the developer of the component to change the platform name.",
                        s
                    ));
                }

                match s {
                    $(
                        $platform_name => Ok(Platform::$variant),
                    )*
                    _ => Err(format!("Unknown platform: {}", s)),
                }
            }
        }

        // Generate the configure_platforms function
        pub(crate) fn configure_platforms(
            config_string: &str,
            config_path: &str,
            platforms: &BTreeSet::<Platform>,
        ) -> Result<Vec<(String, Box<dyn Module>)>, String> {

            let mut modules: Vec<(String, Box<dyn Module>)> = Vec::new();
            for module in platforms.iter() {
                log::debug!("Configuring platform: {:?}", module);
                match module {
                    $(
                        Platform::$variant => {
                            let result = <$module_path::$type_name>::new(config_string, config_path);
                            match result {
                                Ok(component) => {
                                    modules.push(($platform_name.to_string(), Box::new(component)));
                                }
                                Err(e) => {
                                    return Err(format!("{}", e));
                                }
                            }
                        }
                    )*
                }
            }

            Ok(modules)
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/", "components.rs"));

pub(crate) fn initialize_platforms(
    modules: &mut Vec<(String, Box<dyn Module>)>,
) -> Result<Vec<UbiComponent>, String> {
    let mut all_components: Vec<UbiComponent> = Vec::new();
    for (_name, module) in modules.iter_mut() {
        let mut components = module.components();
        all_components.append(&mut components);
    }
    Ok(all_components)
}

/// Sends a shutdown signal when dropped, unless explicitly disarmed.
///
/// Each module task arms this guard before running its module. The guard fires
/// on `Drop`, which covers both an unwinding panic and a returned `Err` (the
/// guard is left armed in the error branch). This is the single, centralized
/// place that logs that a module stopped and requests application shutdown — so
/// the message appears identically whether the module returned `Err` or
/// panicked. A module that returns `Ok(())` — many modules do so immediately
/// after spawning their own worker tasks — disarms the guard so normal
/// completion does not stop the application.
struct ShutdownOnDrop {
    name: String,
    tx: tokio::sync::mpsc::UnboundedSender<()>,
    armed: bool,
}

impl ShutdownOnDrop {
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        if self.armed {
            log::error!(
                "Module '{}' stopped unexpectedly; shutting down the application.",
                self.name
            );
            let _ = self.tx.send(());
        }
    }
}

pub(crate) fn run_platforms(
    modules: Vec<(String, Box<dyn Module>)>,
    sender: Sender<ChangedMessage>,
    receiver: Receiver<PublishedMessage>,
    failure_tx: tokio::sync::mpsc::UnboundedSender<()>,
) {
    for (name, module) in modules {
        let tx = sender.clone();
        let rx = receiver.resubscribe();
        let failure_tx = failure_tx.clone();
        tokio::spawn(async move {
            // Armed by default: a panic while running the module unwinds through
            // this guard, which logs and triggers an application shutdown.
            let mut guard = ShutdownOnDrop {
                name: name.clone(),
                tx: failure_tx,
                armed: true,
            };
            match module.run(tx, rx).await {
                Ok(()) => {
                    // Normal completion (e.g. the module handed off to its own
                    // spawned worker tasks). Do not stop the application.
                    guard.disarm();
                }
                Err(e) => {
                    // Error-specific detail; the centralized shutdown log is
                    // emitted by the guard on drop (also for panics). The
                    // "reporting to Sentry" log is emitted centrally from the
                    // Sentry `before_send` hook for every captured event.
                    log::error!("Module '{}' terminated with error: {}", name, e);
                    if sentry::Hub::current().client().is_some() {
                        sentry::with_scope(
                            |scope| scope.set_tag("module", &name),
                            || sentry::capture_error(e.as_ref()),
                        );
                    }
                    // Leave the guard armed so dropping it stops the application.
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{run_platforms, Platform};
    use std::future::Future;
    use std::pin::Pin;
    use std::time::Duration;
    use tokio::sync::broadcast;
    use ubihome_core::internal::sensors::UbiComponent;
    use ubihome_core::{ChangedMessage, Module, PublishedMessage};

    #[test]
    fn test_reserved_platform_names_are_rejected() {
        let result = Platform::from_str("button");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Reserved platform name"));
    }

    #[derive(Clone, Copy)]
    enum Behavior {
        Ok,
        Err,
        Panic,
    }

    /// A fake module that exercises each `run()` outcome on demand.
    struct FakeModule {
        behavior: Behavior,
    }

    impl Module for FakeModule {
        fn new(_config_string: &str, _config_path: &str) -> Result<Self, String> {
            Ok(FakeModule {
                behavior: Behavior::Ok,
            })
        }

        fn components(&mut self) -> Vec<UbiComponent> {
            Vec::new()
        }

        fn run(
            &self,
            _sender: broadcast::Sender<ChangedMessage>,
            _receiver: broadcast::Receiver<PublishedMessage>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
        {
            let behavior = self.behavior;
            Box::pin(async move {
                match behavior {
                    Behavior::Ok => Ok(()),
                    Behavior::Err => Err("induced error in module".into()),
                    Behavior::Panic => panic!("induced panic in module"),
                }
            })
        }
    }

    /// Runs a single fake module with the given behavior and waits up to one
    /// second for a shutdown signal. Returns `true` if a shutdown was requested.
    async fn shutdown_requested_for(behavior: Behavior) -> bool {
        let (changed_tx, _changed_rx) = broadcast::channel::<ChangedMessage>(16);
        let (_published_tx, published_rx) = broadcast::channel::<PublishedMessage>(16);
        let (failure_tx, mut failure_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        let module: Box<dyn Module> = Box::new(FakeModule { behavior });
        run_platforms(
            vec![("fake".to_string(), module)],
            changed_tx,
            published_rx,
            failure_tx,
        );

        // A shutdown is signalled by sending `()`. Normal completion instead
        // drops all senders, so `recv()` resolves to `None`.
        matches!(
            tokio::time::timeout(Duration::from_secs(1), failure_rx.recv()).await,
            Ok(Some(()))
        )
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn returned_error_triggers_shutdown() {
        assert!(
            shutdown_requested_for(Behavior::Err).await,
            "a module returning Err must request application shutdown"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn panic_triggers_shutdown() {
        assert!(
            shutdown_requested_for(Behavior::Panic).await,
            "a module that panics must request application shutdown"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn normal_completion_does_not_trigger_shutdown() {
        assert!(
            !shutdown_requested_for(Behavior::Ok).await,
            "a module returning Ok must not request application shutdown"
        );
    }
}
