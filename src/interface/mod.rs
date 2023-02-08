#[derive(Default, Debug, Clone)]
pub struct HomeState {
  pub is_aeropex_connected : bool,
}

#[derive(Debug)]
pub enum HomeCommand {
  ConnectAeropex,
  DisconnectAeropex,
}
