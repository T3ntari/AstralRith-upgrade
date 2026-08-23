#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![recursion_limit = "256"]

use native_dialog::{MessageDialog, MessageType};
use tauri::{Listener, Manager};
use theseus::prelude::*;

mod api;
mod error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate cocoa;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

// Should be called in launcher initialization
#[tracing::instrument(skip_all)]
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    tracing::info!("Initializing app event state...");
    theseus::EventState::init(app.clone()).await?;

    // #[cfg(feature = "updater")]
    // {
    //     use tauri_plugin_updater::UpdaterExt;

    //     let updater = app.updater_builder().build()?;

    //     let update_fut = updater.check();

    //     let check_bar = theseus::init_loading(
    //         theseus::LoadingBarType::CheckingForUpdates,
    //         1.0,
    //         "Checking for updates...",
    //     )
    //     .await?;

        // tracing::info!("Checking for updates...");
        // let update = update_fut.await;

    //     drop(check_bar);

    //     if let Some(update) = update.ok().flatten() {
    //         tracing::info!("Update found: {:?}", update.download_url);
    //         let loader_bar_id = theseus::init_loading(
    //             theseus::LoadingBarType::LauncherUpdate {
    //                 version: update.version.clone(),
    //                 current_version: update.current_version.clone(),
    //             },
    //             1.0,
    //             "Updating Modrinth App...",
    //         )
    //         .await?;

    //         // 100 MiB
    //         const DEFAULT_CONTENT_LENGTH: u64 = 1024 * 1024 * 100;

    //         update
    //             .download_and_install(
    //                 |chunk_length, content_length| {
    //                     let _ = theseus::emit_loading(
    //                         &loader_bar_id,
    //                         (chunk_length as f64)
    //                             / (content_length
    //                                 .unwrap_or(DEFAULT_CONTENT_LENGTH)
    //                                 as f64),
    //                         None,
    //                     );
    //                 },
    //                 || {},
    //             )
    //             .await?;

    //         app.restart();
    //     }
    // }

    // #[cfg(not(feature = "updater"))]
    // {
        // }
        
    State::init().await?;
    tracing::info!("AstralRinth state successfully initialized.");
    let state = State::get().await?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir(), true)?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir().join("icons"), true)?;

    Ok(())
}

// Should be call once Vue has mounted the app
#[tracing::instrument(skip_all)]
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    let win = app.get_window("main").unwrap();
    if let Err(e) = win.show() {
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Initialization error")
            .set_text(&format!(
                "Cannot display application window due to an error:\n{}",
                e
            ))
            .show_alert()
            .unwrap();
        panic!("cannot display application window")
    } else {
        let _ = win.set_focus();
    }
}

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

// Toggles decorations
#[tauri::command]
async fn toggle_decorations(b: bool, window: tauri::Window) -> api::Result<()> {
    window.set_decorations(b).map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to toggle decorations: {}",
            e
        )))
    })?;
    Ok(())
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

// if Tauri app is called with arguments, then those arguments will be treated as commands
// ie: deep links or filepaths for .mrpacks
fn main() {
    // Linux rendering (launcher window): keep WebKitGTK on the hardware
    // accelerated path. The launcher is 2D UI — it should render on the
    // integrated GPU (fast, low power), NOT be offloaded to the discrete GPU.
    // Forcing __NV_PRIME_RENDER_OFFLOAD=1 here made WebKitGTK try to allocate
    // GBM buffers through NVIDIA, which fails on PRIME laptops (white screen /
    // "Failed to create GBM buffer"), or forced WEBKIT_DISABLE_DMABUF_RENDERER
    // = software SHM rendering (15 FPS UI on high-end machines).
    //
    // Policy:
    //   * Prefer the X11 backend (GLX) because WebKitGTK DMABUF is reliable
    //     there and NVIDIA GLX breaks under XWayland.
    //   * Keep DMABUF ENABLED (hardware compositing). Only fall back to
    //     disabling it if the user explicitly needs to (env override).
    //   * Do NOT set __NV_PRIME_RENDER_OFFLOAD for the launcher. Minecraft
    //     instances get their own GPU selection from settings.gpu_preference.
    #[cfg(target_os = "linux")]
    {
        // X11 backend so GLX + DMABUF hardware path is used (XWayland SHM is
        // software-rendered and janky). Respect an explicit user override.
        if std::env::var("GDK_BACKEND").is_err() {
            std::env::set_var("GDK_BACKEND", "x11");
        }

        // VSYNC OFF — prevents scroll stutter on NVIDIA GLX.
        if std::env::var("__GL_SYNC_TO_VBLANK").is_err() {
            std::env::set_var("__GL_SYNC_TO_VBLANK", "0");
        }

        // On PRIME/hybrid laptops the default GLX vendor may be NVIDIA, whose
        // GBM/DMABUF path fails under XWayland ("Failed to create GBM buffer"
        // => white window). The launcher is 2D UI: render it with the Mesa
        // (integrated GPU) GLX vendor so DMA-BUF hardware compositing works.
        // Only force this when an Intel/AMD (Mesa) GPU is actually present.
        if std::env::var("__GLX_VENDOR_LIBRARY_NAME").is_err() {
            let has_mesa_gpu = std::fs::read_dir("/sys/class/drm")
                .map(|entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        let vendor_path = e.path().join("device/vendor");
                        std::fs::read_to_string(vendor_path)
                            .map(|v| {
                                let v = v.trim();
                                v == "0x8086" || v == "0x1002"
                            })
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
            if has_mesa_gpu {
                std::env::set_var("__GLX_VENDOR_LIBRARY_NAME", "mesa");
                // Keep the discrete GPU out of EGL/DMA-BUF too. WebKitGTK picks
                // the EGL platform it finds first; with PRIME offload unset,
                // NVIDIA's EGL can still grab DMA-BUF and fail. Pinning the
                // offload to "0" makes Mesa/Intel own the buffers.
                std::env::set_var("__NV_PRIME_RENDER_OFFLOAD", "0");
            }
        }
    }

    /*
        tracing is set basd on the environment variable RUST_LOG=xxx, depending on the amount of logs to show
            ERROR > WARN > INFO > DEBUG > TRACE
        eg. RUST_LOG=info will show info, warn, and error logs
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)

        Error messages returned to Tauri will display as traced error logs if they return an error.
        This will also include an attached span trace if the error is from a tracing error, and the level is set to info, debug, or trace

        on unix:
            RUST_LOG="theseus=trace" {run command}

    */
    let _log_guard = theseus::start_logger();

    tracing::info!("Initialized tracing subscriber. Loading Modrinth App!");

    let mut builder = tauri::Builder::default();

    #[cfg(feature = "updater")]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(payload) = args.get(1) {
                tracing::info!("Handling deep link from arg {payload}");
                let payload = payload.clone();
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
            }

            if let Some(win) = app.get_window("main") {
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_filename(
                    if std::env::current_dir()
                        .ok()
                        .map(|dir| dir.join("portable.txt").exists())
                        .unwrap_or(false)
                    {
                        std::env::current_dir()
                            .ok()
                            .map(|dir| dir.join("UserData/app-window-state.json").to_string_lossy().into_owned())
                            .unwrap()
                    } else {
                        "app-window-state.json".to_string()
                    },
                )
                .build(),
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let payload = macos::deep_link::get_or_init_payload(app);

                let mtx_copy = payload.payload.clone();
                app.listen("deep-link://new-url", move |url| {
                    let mtx_copy_copy = mtx_copy.clone();
                    let request = url.payload().to_owned();

                    let actual_request =
                        serde_json::from_str::<Vec<String>>(&request)
                            .ok()
                            .map(|mut x| x.remove(0))
                            .unwrap_or(request);

                    tauri::async_runtime::spawn(async move {
                        tracing::info!("Handling deep link {actual_request}");

                        let mut payload = mtx_copy_copy.lock().await;
                        if payload.is_none() {
                            *payload = Some(actual_request.clone());
                        }

                        let _ =
                            api::utils::handle_command(actual_request).await;
                    });
                });
            };

            #[cfg(not(target_os = "macos"))]
            app.listen("deep-link://new-url", |url| {
                let payload = url.payload().to_owned();
                tracing::info!("Handling deep link {payload}");
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
                dbg!(url);
            });

            #[cfg(not(target_os = "linux"))]
            {
                if let Some(window) = app.get_window("main") {
                    window.set_shadow(true).unwrap();
                }
            }

            Ok(())
        });

    builder = builder
        .plugin(api::auth::init())
        .plugin(api::cf::init())
        .plugin(api::mr_auth::init())
        .plugin(api::import::init())
        .plugin(api::logs::init())
        .plugin(api::jre::init())
        .plugin(api::metadata::init())
        .plugin(api::pack::init())
        .plugin(api::process::init())
        .plugin(api::profile::init())
        .plugin(api::profile_create::init())
        .plugin(api::settings::init())
        .plugin(api::tags::init())
        .plugin(api::utils::init())
        .plugin(api::cache::init())
        .plugin(api::friends::init())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            is_dev,
            toggle_decorations,
            show_window,
            restart_app,
        ]);

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(macos::window_ext::init());
    }

    tracing::info!("Initializing app...");
    let app = builder.build(tauri::generate_context!());

    match app {
        Ok(app) => {
            #[allow(unused_variables)]
            app.run(|app, event| {
                #[cfg(target_os = "macos")]
                if let tauri::RunEvent::Opened { urls } = event {
                    tracing::info!("Handling webview open {urls:?}");

                    let file = urls
                        .into_iter()
                        .filter_map(|url| url.to_file_path().ok())
                        .next();

                    if let Some(file) = file {
                        let payload =
                            macos::deep_link::get_or_init_payload(app);

                        let mtx_copy = payload.payload.clone();
                        let request = file.to_string_lossy().to_string();
                        tauri::async_runtime::spawn(async move {
                            let mut payload = mtx_copy.lock().await;
                            if payload.is_none() {
                                *payload = Some(request.clone());
                            }

                            let _ = api::utils::handle_command(request).await;
                        });
                    }
                }
            });
        }
        Err(e) => {
            #[cfg(target_os = "windows")]
            {
                // tauri doesn't expose runtime errors, so matching a string representation seems like the only solution
                if format!("{:?}", e).contains(
                    "Runtime(CreateWebview(WebView2Error(WindowsError",
                ) {
                    MessageDialog::new()
                        .set_type(MessageType::Error)
                        .set_title("Initialization error")
                        .set_text("Your Microsoft Edge WebView2 installation is corrupt.\n\nMicrosoft Edge WebView2 is required to run Modrinth App.\n\nLearn how to repair it at https://support.modrinth.com/en/articles/8797765-corrupted-microsoft-edge-webview2-installation")
                        .show_alert()
                        .unwrap();

                    panic!("webview2 initialization failed")
                }
            }

            MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("Initialization error")
                .set_text(&format!(
                    "Cannot initialize application due to an error:\n{:?}",
                    e
                ))
                .show_alert()
                .unwrap();

            tracing::error!("Error while running tauri application: {:?}", e);
            panic!("{1}: {:?}", e, "error while running tauri application")
        }
    }
}
