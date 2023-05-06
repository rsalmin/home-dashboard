use crate::egui::Context; // b/c of re-export
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::sync::mpsc::error::TrySendError;
use tokio;
use log;
use std::str::FromStr;
use bluez_async::{MacAddress, DeviceId, DeviceInfo, BluetoothSession, BluetoothEvent, DeviceEvent};
use crate::interface::*;
use futures::stream::StreamExt;

#[derive(Clone)]
struct Configuration {
  aeropex_id : String,
  edifier_id : String,
}

impl Configuration {
 fn new() -> Self {
   Configuration {
     aeropex_id : String::from("20:74:CF:BD:61:41"),
     edifier_id : String::from("0C:AE:BD:6D:95:0D"),
   }
 }
}

#[tokio::main]
pub async fn worker_thread(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context) {
  let result = worker_thread_prime(sender, receiver, ctx).await;
  if let Err ( e ) = result {
    log::error!("Error in worker_thread : {}. exiting....", e);
  }
}

pub async fn worker_thread_prime(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context) -> Result<(), String> {

  let cfg = Configuration::new();

  let bt_session = BluetoothSession::new().await;
  if let Err( e ) = bt_session {
    return Err( format!("Failed to open BluetoothSession : {:?} ; can't fulfill my duty.", e) );
  }
  let bt_session = bt_session.unwrap().1;

  let devices = bt_session.get_devices().await;
  if let Err( e ) = devices {
       return Err( format!("Failed to get bluetooth device list : {:?}; nothing to do.", e) );
  }
  let devices = devices.unwrap();

  let aeropex_id = find_device_id(&devices, &cfg.aeropex_id)?;
  let edifier_id = find_device_id(&devices, &cfg.edifier_id)?;

  let h1 = tokio::task::spawn( update_state_loop(sender, aeropex_id.clone(), edifier_id.clone(), ctx, bt_session.clone()) );
  let h2 = tokio::task::spawn( execute_command_loop(receiver, aeropex_id, edifier_id, bt_session) );

  match h1.await {
    Err( e ) => log::warn!("update_state_loop task is faield... {:?}", e),
    Ok( o ) => o?,
  }
  if let Err( e ) = h2.await {
    log::warn!("execute_command_loop task is faield... {:?}", e);
  }

  Ok(())
}

async fn update_state_loop(
  sender : Sender<HomeState>,
  aeropex_id : DeviceId,
  edifier_id : DeviceId,
  egui_ctx : Context,
  bt_session : BluetoothSession ) -> Result<(), String>
{
  let mut state = HomeState::default();
  state.is_aeropex_connected = check_bluetooth_status(&aeropex_id, &bt_session).await;
  state.is_edifier_connected = check_bluetooth_status(&edifier_id, &bt_session).await;

  sender.try_send(state.clone()).map_err(|x| x.to_string())?;
  egui_ctx.request_repaint();

  //FIXME: if devices switched it's state between initial state request and event_stream loop, we will have a problem
  let mut aeropex_event_stream = bt_session.device_event_stream(&aeropex_id).await.map_err(|x| x.to_string())?;
  let mut edifier_event_stream = bt_session.device_event_stream(&edifier_id).await.map_err(|x| x.to_string())?;

  loop {
    tokio::select! {
      Some( event ) = aeropex_event_stream.next() => {
        log::debug!("Got BT event {:?}", event);
        if let BluetoothEvent::Device { event : DeviceEvent::Connected{ connected }, .. } = event {
          state.is_aeropex_connected = connected;
        }
      }
      Some( event ) = edifier_event_stream.next() => {
        log::debug!("Got BT event {:?}", event);
        if let BluetoothEvent::Device { event : DeviceEvent::Connected{ connected }, .. } = event {
          state.is_edifier_connected = connected;
        }
      }
      else => { break; }
    }

    match sender.try_send(state.clone()) {
      Ok(()) => egui_ctx.request_repaint(),
      Err( TrySendError::Full( _ ) ) => log::warn!("Failed to send data, GUI is not consuming it!"),
      Err( TrySendError::Closed( _ ) ) => {
        log::warn!("Failed to send data - channel is closed. Probably GUI is dead, exiting....");
        break;
      },
    }

  }


  log::warn!("Event stream from BT is ended... strange... exiting...");
  Ok(())
}

async fn execute_command_loop(
  mut receiver : Receiver<HomeCommand>,
  aeropex_id : DeviceId,
  edifier_id : DeviceId,
  bt_session : BluetoothSession
  )
{
  loop {
      match receiver.recv().await {
      Some( cmd ) => execute_command( &aeropex_id, &edifier_id, &bt_session, cmd ).await,
      None => {
        log::warn!("Failed to receiver data, probably GUI is dead. Exiting...");
        break;
      },
     };

  }
}

async fn execute_command(aeropex_id : &DeviceId, edifier_id : &DeviceId, bt_session : &BluetoothSession, cmd : HomeCommand)
{
  log::debug!("Got CMD: {:?}", cmd);
  match cmd {
    HomeCommand::ConnectAeropex => {
      if let Err( e ) =  bt_session.connect(aeropex_id).await {
        log::warn!("Error while connecting to {:?} : {:?}", aeropex_id, e);
      }
    },
    HomeCommand::DisconnectAeropex =>
      if let Err( e ) =  bt_session.disconnect(aeropex_id).await {
        log::warn!("Error while disconnecting to {:?} : {:?}", aeropex_id, e);
    },
    HomeCommand::ConnectEdifier => {
      if let Err( e ) =  bt_session.connect(edifier_id).await {
        log::warn!("Error while connecting to {:?} : {:?}", edifier_id, e);
      }
    },
    HomeCommand::DisconnectEdifier =>
      if let Err( e ) =  bt_session.disconnect(edifier_id).await {
        log::warn!("Error while disconnecting to {:?} : {:?}", edifier_id, e);
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

