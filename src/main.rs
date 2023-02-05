mod interface;
mod worker;
mod gui;

use eframe::egui;
use gui::HomeDashboard;

fn main() {

  let mut native_options = eframe::NativeOptions::default();
  native_options.fullscreen = true;

  eframe::run_native(
    "Home Dashboard",
    native_options,
    Box::new(|cc| Box::new(HomeDashboard::new(cc)) )
  );
}
