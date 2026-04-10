use serde::Deserialize;
use std::collections::HashSet;
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::Module;
use ubihome_core::{ChangedMessage, PublishedMessage};

macro_rules! generate_component_methods {
    (
        $(($variant:ident, $platform_name:literal, $module_path:ident, $type_name:ident)),* $(,)?
    ) => {
        // Generate the Platform enum
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
        #[serde(rename_all = "lowercase")]
        pub enum Platform {
            $(
                $variant,
            )*
        }

        impl Platform {
            /// Parse a platform from its string representation
            pub fn from_str(s: &str) -> Result<Self, String> {
                match s {
                    $(
                        $platform_name => Ok(Platform::$variant),
                    )*
                    _ => Err(format!("Unknown platform: {}", s)),
                }
            }

            // /// Get the string representation of the platform
            // pub fn as_str(&self) -> &'static str {
            //     match self {
            //         $(
            //             Platform::$variant => $platform_name,
            //         )*
            //     }
            // }
        }

        // Generate the configure_platforms function
        pub(crate) fn configure_platforms(
            config_string: &String,
            platforms: &HashSet::<Platform>,
        ) -> Result<Vec<Box<dyn Module>>, String> {

            let mut modules: Vec<Box<dyn Module>> = Vec::new();
            for module in platforms.iter() {
                match module {
                    $(
                        Platform::$variant => {
                            let result = <$module_path::$type_name>::new(config_string);
                            match result {
                                Ok(component) => {
                                    modules.push(Box::new(component));
                                }
                                Err(e) => {
                                    return Err(format!("Module {}: {}", stringify!($module_path), e));
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
        // println!("Module: {:?}", &components);
        all_components.append(&mut components);
    }
    Ok(all_components)
}

pub(crate) async fn run_platforms(
    modules: Vec<Box<dyn Module>>,
    sender: Sender<ChangedMessage>,
    receiver: Receiver<PublishedMessage>,
) {
    for module in modules {
        let tx = sender.clone();
        let rx = receiver.resubscribe();
        tokio::spawn({
            async move {
                module.run(tx, rx).await.unwrap();
            }
        });
    }
}
