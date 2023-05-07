use log;
use std::str::FromStr;
use bluez_async::{MacAddress, DeviceId, DeviceInfo, BluetoothEvent, DeviceEvent, BluetoothSession};
use futures::Stream;
use futures::stream::StreamExt;
use crate::interface::*;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TrySendError;

#[derive(Clone)]
pub struct BluetoothModule {
  session : BluetoothSession,
  aeropex_id : DeviceId,
  edifier_id : DeviceId,
}

impl BluetoothModule {
  pub async fn new(bt_config : &BluetoothConfig) -> Result<Self, String> {

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

      let aeropex_id = find_device_id(&devices, &bt_config.aeropex_mac)?;
      let edifier_id = find_device_id(&devices, &bt_config.edifier_mac)?;

      Ok( BluetoothModule { session, aeropex_id, edifier_id } )
  }

  pub async fn get_state(&self) -> BluetoothState {
      let mut bt_state = BluetoothState::default();
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

pub async fn watch_bluetooth_loop(
    bt_module : BluetoothModule ,
    bt_sender : Sender<BluetoothState>) -> Result<(), String>
{
  let mut bt_state = bt_module.get_state().await;

  //FIXME: if devices switched it's state between initial state request and event_stream loop, we will have a problem
  let mut aeropex_event_stream = bt_module.aeropex_event_stream().await?;
  let mut edifier_event_stream = bt_module.edifier_event_stream().await?;

  loop {
    match bt_sender.try_send(bt_state.clone()) {
      Ok(()) => (),
      Err( TrySendError::Full( _ ) ) => log::warn!("Failed to send BT data, update_state_loop is not consuming it!"),
      Err( TrySendError::Closed( _ ) ) => {
        log::warn!("Failed to send data - channel is closed. Probably update_state_loop is dead now. Exiting....");
        return Ok(());
      },
    }

    tokio::select! {
      Some( event ) = aeropex_event_stream.next() => {
        log::debug!("Got BT event {:?}", event);
        if let BluetoothEvent::Device { event : DeviceEvent::Connected{ connected }, .. } = event {
          bt_state.is_aeropex_connected = connected;
        }
      }
      Some( event ) = edifier_event_stream.next() => {
        log::debug!("Got BT event {:?}", event);
        if let BluetoothEvent::Device { event : DeviceEvent::Connected{ connected }, .. } = event {
          bt_state.is_edifier_connected = connected;
        }
      }
      else => { break; }
    }

  }

  log::warn!("Event stream from BT is ended... strange... exiting...");
  Ok(())
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

