#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

use battle_net_manager_lib::commands;
use battle_net_manager_lib::BattleNetCore;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Manage the core business logic
            app.manage(BattleNetCore::new());

            // Disable default context menu in webview
            let window = app.get_webview_window("main").unwrap();
            window.eval("document.addEventListener('contextmenu', e => e.preventDefault())").unwrap();

            // Build system tray context menu
            let show_item = MenuItemBuilder::with_id("show", "主界面").build(app)?;
            let exit_item = MenuItemBuilder::with_id("exit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&exit_item)
                .build()?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("战网账号切换")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "exit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Close-to-tray: hide instead of closing
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_accounts,
            commands::get_groups,
            commands::create_group,
            commands::rename_group,
            commands::delete_group,
            commands::move_account_to_group,
            commands::update_account_info,
            commands::save_current_account_to_group,
            commands::switch_account,
            commands::delete_account,
            commands::add_new_account,
            commands::get_active_account_id,
            commands::refresh_account_session_state,
            commands::open_external_url,
            commands::drag_window,
            commands::minimize_app,
            commands::close_app,
            commands::show_window,
            commands::check_update,
            commands::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
