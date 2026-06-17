pub mod models;
pub mod coins;
pub mod portfolio;
pub mod db;
pub mod prices;
pub mod commands;
pub mod scheduler;

use tauri::Manager;
use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("app data dir");
            std::fs::create_dir_all(&dir).ok();
            let db = db::Db::open_at(&dir.join("tracker.db")).expect("open db");
            app.manage(AppState { db, prices: prices::PriceClient::new() });
            scheduler::spawn(app.handle().clone(), 120);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_coins,
            commands::list_portfolios,
            commands::create_portfolio,
            commands::add_transaction,
            commands::list_transactions,
            commands::get_dashboard,
            commands::refresh_prices,
            commands::get_coin_history,
            commands::get_portfolio_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
