use log::{debug, error, info};
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, time::Duration};
use std::{future::Future, pin::Pin, str};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time::sleep,
};
use ubihome_core::{
    config_template,
    home_assistant::sensors::{UbiButton, UbiEvent},
    internal::sensors::{InternalButton, InternalComponent, InternalEvent},
    ChangedMessage, Module, NoConfig, PublishedMessage,
};

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPosition, PlatformConfig};

#[derive(Clone, Deserialize, Debug)]
pub struct MediaControlsConfig {
    pub display_entity: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct MediaControlsButtonConfig {
    // pub action: PowerAction
}

config_template!(
    media_controls,
    MediaControlsConfig,
    MediaControlsButtonConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

pub struct UbiHomeDefault {
    config: MediaControlsConfig,
    components: Vec<InternalComponent>,
    events: HashMap<String, InternalEvent>,
}

impl Module for UbiHomeDefault {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        info!("Media Controls config: {:?}", config);
        let mut components: Vec<InternalComponent> = Vec::new();
        let mut events: HashMap<String, InternalEvent> = HashMap::new();

        for (_, any_sensor) in config.event.clone().unwrap_or_default() {
            match any_sensor.extra {
                EventKind::media_controls(event) => {
                    let id = any_sensor.default.get_object_id();
                    let button_component;
                    button_component = InternalEvent {
                        ha: UbiEvent {
                            platform: "sensor".to_string(),
                            icon: Some(
                                any_sensor.default.icon.unwrap_or("mdi:restart".to_string()),
                            ),
                            name: any_sensor.default.name.clone(),
                            id: id.clone(),
                            device_class: None,
                            event_types: vec![
                                "play".to_string(),
                                "pause".to_string(),
                                "next".to_string(),
                                "previous".to_string(),
                                "stop".to_string(),
                            ],
                        },
                    };

                    components.push(InternalComponent::Event(button_component.clone()));
                    events.insert(id, button_component);
                }
                _ => {}
            }
        }
        Ok(UbiHomeDefault {
            config: config.media_controls,
            components,
            events,
        })
    }

    fn components(&mut self) -> Vec<InternalComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        let events = self.events.clone();
        // let sender = buttons.clone();
        Box::pin(async move {
            if let Some(display_entity) = config.display_entity {
                sleep(std::time::Duration::from_secs(10)).await;
                sender
                    .send(ChangedMessage::APISubscribeEntity {
                        entity: display_entity.clone(),
                        attribute: "media_title".to_string(),
                    })
                    .unwrap();
                sender
                    .send(ChangedMessage::APISubscribeEntity {
                        entity: display_entity.clone(),
                        attribute: "media_artist".to_string(),
                    })
                    .unwrap();
                sender
                    .send(ChangedMessage::APISubscribeEntity {
                        entity: display_entity.clone(),
                        attribute: "media_duration".to_string(),
                    })
                    .unwrap();
                sender
                    .send(ChangedMessage::APISubscribeEntity {
                        entity: display_entity.clone(),
                        attribute: "media_position".to_string(),
                    })
                    .unwrap();
                sender
                    .send(ChangedMessage::APISubscribeEntity {
                        entity: display_entity.clone(),
                        attribute: "entity_picture".to_string(),
                    })
                    .unwrap();
                sender
                    .send(ChangedMessage::APISubscribeEntity {
                        entity: display_entity.clone(),
                        attribute: "".to_string(),
                    })
                    .unwrap();
            }

            #[cfg(not(target_os = "windows"))]
            let hwnd = None;

            #[cfg(target_os = "windows")]
            let (hwnd, _dummy_window) = {
                let dummy_window = windows::DummyWindow::new().unwrap();
                let handle = Some(dummy_window.handle.0 as _);
                (handle, dummy_window)
            };

            let config = PlatformConfig {
                dbus_name: "my_player",
                display_name: "My Player",
                hwnd,
            };

            let mut controls = MediaControls::new(config).unwrap();

            // The closure must be Send and have a static lifetime.
            let sender_clone = sender.clone();
            let event = events
                .iter()
                .next()
                .map(|(id, event)| (id.clone(), event.clone()));
            if let Some((id, event)) = event {
                controls
                    .attach(move |event: MediaControlEvent| {
                        println!("Event received: {:?}", event);
                        let mut message: Option<ChangedMessage> = None;
                        match event {
                            MediaControlEvent::Play => {
                                message = Some(ChangedMessage::EventChange {
                                    key: id.to_string(),
                                    r#type: "play".to_string(),
                                });
                            }
                            MediaControlEvent::Pause => {
                                message = Some(ChangedMessage::EventChange {
                                    key: id.to_string(),
                                    r#type: "pause".to_string(),
                                });
                            }
                            MediaControlEvent::Next => {
                                message = Some(ChangedMessage::EventChange {
                                    key: id.to_string(),
                                    r#type: "next".to_string(),
                                });
                            }
                            MediaControlEvent::Previous => {
                                message = Some(ChangedMessage::EventChange {
                                    key: id.to_string(),
                                    r#type: "previous".to_string(),
                                });
                            }
                            MediaControlEvent::Stop => {
                                message = Some(ChangedMessage::EventChange {
                                    key: id.to_string(),
                                    r#type: "stop".to_string(),
                                });
                            }
                            _ => {}
                        }
                        if let Some(message) = message {
                            sender_clone.send(message).unwrap();
                        }
                    })
                    .unwrap();
            }

            // Handle Button Presses
            tokio::spawn(async move {
                let mut title: String = String::new();
                let mut artist: String = String::new();
                let mut entity_picture: String = String::new();
                let mut duration: Duration = Duration::new(0, 0);
                let mut progress: Duration = Duration::new(0, 0);

                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
                        PublishedMessage::APISubscribedEntityChange {
                            entity,
                            attribute,
                            state,
                        } => {
                            if attribute == "media_title" {
                                title = state.clone()
                            } else if attribute == "media_artist" {
                                artist = state.clone()
                            } else if attribute == "media_duration" {
                                duration = Duration::from_secs(state.parse::<u64>().unwrap_or(0));
                            } else if attribute == "media_position" {
                                progress = Duration::from_secs(state.parse::<u64>().unwrap_or(0));
                            } else if attribute == "entity_picture" {
                                entity_picture = state.clone();
                            }

                            if attribute.is_empty() {
                                match state.as_str() {
                                    "playing" => {
                                        controls
                                            .set_playback(souvlaki::MediaPlayback::Playing {
                                                progress: Some(MediaPosition(progress)),
                                            })
                                            .unwrap();
                                    }
                                    "idle" => {
                                        controls
                                            .set_playback(souvlaki::MediaPlayback::Paused {
                                                progress: Some(MediaPosition(progress)),
                                            })
                                            .unwrap();
                                    }
                                    _ => {}
                                }
                            }
                            if entity_picture.is_empty() {
                                controls
                                    .set_metadata(MediaMetadata {
                                        title: Some(&title),
                                        artist: Some(&artist),
                                        album: Some("Souvlaki"),
                                        duration: Some(duration),
                                        ..Default::default()
                                    })
                                    .unwrap();
                            } else {
                                controls
                                    .set_metadata(MediaMetadata {
                                        title: Some(&title),
                                        artist: Some(&artist),
                                        album: Some("Souvlaki"),
                                        duration: Some(duration),
                                        cover_url: Some(&entity_picture),
                                    })
                                    .unwrap();
                            }
                        }

                        _ => {
                            debug!("Ignored message type: {:?}", cmd);
                        }
                    }
                }
            });

            // Your actual logic goes here.
            loop {
                sleep(std::time::Duration::from_millis(100)).await;

                // this must be run repeatedly by your program to ensure
                // the Windows event queue is processed by your application
                #[cfg(target_os = "windows")]
                windows::pump_event_queue();
            }

            Ok(())
        })
    }
}

// demonstrates how to make a minimal window to allow use of media keys on the command line
#[cfg(target_os = "windows")]
mod windows {
    use std::io::Error;
    use std::mem;

    use windows::core::PCWSTR;
    use windows::w;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetAncestor,
        IsDialogMessageW, PeekMessageW, RegisterClassExW, TranslateMessage, GA_ROOT, MSG,
        PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT, WNDCLASSEXW,
    };

    pub struct DummyWindow {
        pub handle: HWND,
    }

    impl DummyWindow {
        pub fn new() -> Result<DummyWindow, String> {
            let class_name = w!("SimpleTray");

            let handle_result = unsafe {
                let instance = GetModuleHandleW(None)
                    .map_err(|e| (format!("Getting module handle failed: {e}")))?;

                let wnd_class = WNDCLASSEXW {
                    cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                    hInstance: instance,
                    lpszClassName: PCWSTR::from(class_name),
                    lpfnWndProc: Some(Self::wnd_proc),
                    ..Default::default()
                };

                if RegisterClassExW(&wnd_class) == 0 {
                    return Err(format!(
                        "Registering class failed: {}",
                        Error::last_os_error()
                    ));
                }

                let handle = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class_name,
                    w!("UbiHome"),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    instance,
                    None,
                );

                if handle.0 == 0 {
                    Err(format!(
                        "Message only window creation failed: {}",
                        Error::last_os_error()
                    ))
                } else {
                    Ok(handle)
                }
            };

            handle_result.map(|handle| DummyWindow { handle })
        }
        extern "system" fn wnd_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }

    impl Drop for DummyWindow {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.handle);
            }
        }
    }

    pub fn pump_event_queue() -> bool {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let mut has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
            while msg.message != WM_QUIT && has_message {
                if !IsDialogMessageW(GetAncestor(msg.hwnd, GA_ROOT), &msg).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
            }

            msg.message == WM_QUIT
        }
    }
}
