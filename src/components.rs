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
        ) -> Result<Vec<Box<dyn Module>>, String> {

            let mut modules: Vec<Box<dyn Module>> = Vec::new();
            for module in platforms.iter() {
                log::debug!("Configuring platform: {:?}", module);
                match module {
                    $(
                        Platform::$variant => {
                            let result = <$module_path::$type_name>::new(config_string, config_path);
                            match result {
                                Ok(component) => {
                                    modules.push(Box::new(component));
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
    modules: &mut Vec<Box<dyn Module>>,
) -> Result<Vec<UbiComponent>, String> {
    let mut all_components: Vec<UbiComponent> = Vec::new();
    for module in modules.iter_mut() {
        let mut components = module.components();
        all_components.append(&mut components);
    }
    Ok(all_components)
}

pub(crate) fn run_platforms(
    tasks: &mut tokio::task::JoinSet<()>,
    modules: Vec<Box<dyn Module>>,
    sender: Sender<ChangedMessage>,
    receiver: Receiver<PublishedMessage>,
) {
    for module in modules {
        let tx = sender.clone();
        let rx = receiver.resubscribe();
        tasks.spawn(async move {
            // A module returning Ok(()) is a normal, intentional exit and stays
            // silent. An Err is a real failure: unwrap panics, which is reported to
            // Sentry by the panic integration. Tasks are collected in a JoinSet so
            // the caller observes the panic (instead of it being swallowed by this
            // task) and shuts the application down.
            module.run(tx, rx).await.unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ubihome_core::ModuleRunFuture;

    #[test]
    fn test_reserved_platform_names_are_rejected() {
        let result = Platform::from_str("button");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Reserved platform name"));
    }

    /// Minimal module whose `run` either exits normally or returns an error,
    /// used to exercise how `run_platforms` supervises module tasks.
    struct MockModule {
        should_err: bool,
    }

    impl Module for MockModule {
        fn new(_config_string: &str, _config_path: &str) -> Result<Self, String> {
            Ok(MockModule { should_err: false })
        }

        fn components(&mut self) -> Vec<UbiComponent> {
            Vec::new()
        }

        fn run(
            &self,
            _sender: Sender<ChangedMessage>,
            _receiver: Receiver<PublishedMessage>,
        ) -> ModuleRunFuture {
            let should_err = self.should_err;
            Box::pin(async move {
                if should_err {
                    Err("mock module failure".into())
                } else {
                    Ok(())
                }
            })
        }
    }

    fn spawn_mock(should_err: bool) -> tokio::task::JoinSet<()> {
        let (changed_tx, _changed_rx) = tokio::sync::broadcast::channel::<ChangedMessage>(16);
        let (_published_tx, published_rx) = tokio::sync::broadcast::channel::<PublishedMessage>(16);
        let mut tasks = tokio::task::JoinSet::new();
        let modules: Vec<Box<dyn Module>> = vec![Box::new(MockModule { should_err })];
        run_platforms(&mut tasks, modules, changed_tx, published_rx);
        tasks
    }

    /// A module returning Ok(()) is an intentional exit: the task completes
    /// normally so the supervisor leaves the application running.
    #[tokio::test]
    async fn run_platforms_normal_exit_completes_cleanly() {
        let mut tasks = spawn_mock(false);
        let joined = tasks.join_next().await.expect("one task was spawned");
        assert!(
            joined.is_ok(),
            "a module returning Ok(()) must not panic its task"
        );
    }

    /// A module returning Err is a real failure: unwrap panics, which the
    /// supervisor observes (via a JoinError) to trigger application shutdown.
    #[tokio::test]
    async fn run_platforms_erroring_module_panics_task() {
        let mut tasks = spawn_mock(true);
        let joined = tasks.join_next().await.expect("one task was spawned");
        let join_error = joined.expect_err("a module returning Err must panic its task");
        assert!(
            join_error.is_panic(),
            "the failure must surface as a panic, not a cancellation"
        );
    }
}
