mod interface;
mod worker;
mod gui;

use eframe::egui;
use env_logger;
use gui::HomeDashboard;


fn main() {
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

  let mut native_options = eframe::NativeOptions::default();
  native_options.fullscreen = true;

  if let Err( e ) = eframe::run_native(
      "Home Dashboard",
      native_options,
      Box::new(|cc| Box::new(HomeDashboard::new(cc)) )
    )  {
    log::error!("Failed to run Home Dashboard {:?}", e);
  };
}
