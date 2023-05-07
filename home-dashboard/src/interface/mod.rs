use serde::{Serialize, Deserialize};
use netatmo_connect::ConnectConfig;

#[derive(Default, Debug, Clone)]
pub struct HomeState {
  pub bt_state : BluetoothState,
  pub weather_data : WeatherData,
}

#[derive(Default, Debug, Clone)]
pub struct BluetoothState {
  pub is_aeropex_connected : bool,
  pub is_edifier_connected : bool,
}

#[derive(Default, Debug, Clone)]
pub struct WeatherData {
  pub room_temperature : f32,
  pub room_humidity : i32,
  pub room_co2 : i32,
  pub room_noise : i32,
  pub pressure : f32,
  pub outdoor_temperature : f32,
  pub outdoor_humidity : i32,
  pub battery : i32,
}

#[derive(Debug)]
pub enum HomeCommand {
  ConnectAeropex,
  DisconnectAeropex,
  ConnectEdifier,
  DisconnectEdifier,
}

#[derive(Serialize, Deserialize, Default)]
pub struct HomeDashboardConfig {
  pub connect_config : ConnectConfig,
  pub bt_config : BluetoothConfig,
}

#[derive(Serialize, Deserialize, Default)]
pub struct BluetoothConfig {
  pub aeropex_mac : String,
  pub edifier_mac : String,
}
