use serde::{Serialize, Deserialize};
use netatmo_connect::ConnectConfig;

#[derive(Default, Debug, Clone)]
pub struct HomeState {
  pub bt_state : BluetoothConnectionsState,
}

#[derive(Default, Debug, Clone)]
pub struct BluetoothConnectionsState {
  pub is_aeropex_connected : bool,
  pub is_edifier_connected : bool,
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
