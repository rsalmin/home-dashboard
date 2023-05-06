use log;
use std::str::FromStr;
use bluez_async::{MacAddress, DeviceId, DeviceInfo, BluetoothEvent, BluetoothSession};
use futures::Stream;
use crate::interface::*;

#[derive(Clone)]
pub struct BluetoothModule {
  session : BluetoothSession,
  aeropex_id : DeviceId,
  edifier_id : DeviceId,
}

impl BluetoothModule {
  pub async fn new(aeropex_mac : &str, edifier_mac : &str) -> Result<Self, String> {

      let session = BluetoothSession::new().await;
      if let Err( e ) = session {
            return Err( format!("Failed to open BluetoothSession : {:?} ; can't fulfill my duty.", e) );
      }
      let session = session.unwrap().1;

      let devices = session.get_devices().await;
      if let Err( e ) = devices {
           return Err( format!("Failed to get bluetooth device list : {:?}; nothing to do.", e) );
      }
      let devices = devices.unwrap();

      let aeropex_id = find_device_id(&devices, aeropex_mac)?;
      let edifier_id = find_device_id(&devices, edifier_mac)?;

      Ok( BluetoothModule { session, aeropex_id, edifier_id } )
  }

  pub async fn get_state(&self) -> BluetoothConnectionsState {
      let mut bt_state = BluetoothConnectionsState::default();
      bt_state.is_aeropex_connected = check_bluetooth_status(&self.aeropex_id, &self.session).await;
      bt_state.is_edifier_connected = check_bluetooth_status(&self.edifier_id, &self.session).await;
      bt_state
  }

  pub async fn aeropex_event_stream(&self) -> Result<impl Stream<Item = BluetoothEvent>, String> {
      self.session.device_event_stream(&self.aeropex_id).await.map_err(|x| x.to_string())
  }

  pub async fn edifier_event_stream(&self) -> Result<impl Stream<Item = BluetoothEvent>, String> {
      self.session.device_event_stream(&self.edifier_id).await.map_err(|x| x.to_string())
  }
}

pub async fn execute_command(bt_module : &BluetoothModule, cmd : HomeCommand)
{
  log::debug!("Got CMD: {:?}", cmd);
  match cmd {
    HomeCommand::ConnectAeropex => {
      if let Err( e ) =  bt_module.session.connect(&bt_module.aeropex_id).await {
        log::warn!("Error while connecting to {:?} : {:?}", bt_module.aeropex_id, e);
      }
    },
    HomeCommand::DisconnectAeropex =>
      if let Err( e ) =  bt_module.session.disconnect(&bt_module.aeropex_id).await {
        log::warn!("Error while disconnecting to {:?} : {:?}", bt_module.aeropex_id, e);
    },
    HomeCommand::ConnectEdifier => {
      if let Err( e ) =  bt_module.session.connect(&bt_module.edifier_id).await {
        log::warn!("Error while connecting to {:?} : {:?}", bt_module.edifier_id, e);
      }
    },
    HomeCommand::DisconnectEdifier =>
      if let Err( e ) =  bt_module.session.disconnect(&bt_module.edifier_id).await {
        log::warn!("Error while disconnecting to {:?} : {:?}", bt_module.edifier_id, e);
    },
  };
}

async fn check_bluetooth_status(aeropex_id : &DeviceId, bt_session : &BluetoothSession) -> bool {

  match bt_session.get_device_info(aeropex_id).await {
    Err( e ) => {
       log::warn!("Failed to get device info: {:?}", e);
       false
    },
    Ok( info ) => info.connected,
  }

}

pub fn find_device_id(devices : &Vec<DeviceInfo>, mac_string : &str) -> Result<DeviceId, String> {
  let mac = MacAddress::from_str(&mac_string);

  if let Err( _ ) = mac {
    return Err( format!("bad MAC {mac_string} in configuration. Can't parse it.") );
  }
  let mac = mac.unwrap();

  let device = devices.into_iter().find(|device| device.mac_address == mac);
  if device.is_none() {
      return Err( format!("Failed to find device with mac {:?}; nothing to do. exiting...", mac) );
  };

  Ok( device.unwrap().id.clone() )
}

