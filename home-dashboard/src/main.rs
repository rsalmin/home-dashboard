mod interface;
mod worker;
mod gui;

use confy;
use eframe::egui;
use env_logger;
use gui::HomeDashboard;
use interface::HomeDashboardConfig;

fn main() {
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

  let configuration_name = "home-dashboard";
  let configuration_path = confy::get_configuration_file_path(configuration_name, None);
  if let Err ( e ) = configuration_path {
      log::error!("Failed to obtain configuration path for {}. {:?}. Exiting.", configuration_name, e);
      return;
  }
  let configuration_path = configuration_path.unwrap();

  log::info!("Configuration path: {}", configuration_path.display());

  let cfg  = confy::load(configuration_name, None);
  if let Err ( e ) = cfg {
      log::error!("Failed to load configuration from {}. {:?}. Exiting.", configuration_path.display(), e);
      return;
  }
  let cfg : HomeDashboardConfig = cfg.unwrap();

  let mut native_options = eframe::NativeOptions::default();
  native_options.fullscreen = true;

  if let Err( e ) = eframe::run_native(
      "Home Dashboard",
      native_options,
      Box::new(|cc| Box::new(HomeDashboard::new(cc, cfg)) )
    ) {
        log::error!("Failed to start HomeDashboard. {:?}", e);
   }

}
