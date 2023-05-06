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
