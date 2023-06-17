use serde::{Serialize, Deserialize};
use netatmo_connect::ConnectConfig;
use std::option::Option;

#[derive(Default, Debug, Clone)]
pub struct HomeState {
  pub bt_state : BluetoothState,
  pub weather_data : Option<WeatherData>,
}

#[derive(Default, Debug, Clone)]
pub struct BluetoothState {
  pub is_aeropex_connected : bool,
  pub is_edifier_connected : bool,
}

#[derive(Debug, Clone)]
pub enum Trend {
  Stable,
  Up,
  Down,
}

#[derive(Default, Debug, Clone)]
pub struct WeatherData {
  pub room_temperature : f32,
  pub room_humidity : i32,
  pub room_co2 : i32,
  pub room_noise : i32,
  pub pressure : f32,
  pub pressure_trend : Option<Trend>,
  pub outdoor_weather : Option<OutdoorWeatherData>,
}

#[derive(Default, Debug, Clone)]
pub struct OutdoorWeatherData {
  pub temperature : f32,
  pub temperature_trend : Option<Trend>,
  pub humidity : i32,
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
