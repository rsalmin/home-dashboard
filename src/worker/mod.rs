use crate::egui::Context; // b/c of re-export
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::sync::mpsc::error::TrySendError;
use tokio;
use log;
use std::str::FromStr;
use bluez_async::{MacAddress, DeviceId, BluetoothSession, BluetoothEvent, DeviceEvent};
use crate::interface::*;
use futures::stream::StreamExt;

#[derive(Clone)]
struct Configuration {
  aeropex_id : String,
}

impl Configuration {
 fn new() -> Self {
   Configuration { aeropex_id : String::from("20:74:CF:BD:61:41") }
 }
}

#[tokio::main]
pub async fn worker_thread(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context) {

  let cfg = Configuration::new();
  let aeropex_mac = MacAddress::from_str(&cfg.aeropex_id)
                              .expect("MAC for aeropex in configuration is Incorrect! Programmer is morron!");

  let bt_session = BluetoothSession::new().await;
  if let Err( e ) = bt_session {
    log::error!("Failed to open BluetoothSession : {:?} ; can't fulfill my duty. exiting....", e);
    return;
  }
  let bt_session = bt_session.unwrap().1;

  let devices = bt_session.get_devices().await;
  if let Err( e ) = devices {
       log::error!("Failed to get bluetooth device list : {:?}; nothing to do. exiting....", e);
       return;
  }
  let devices = devices.unwrap();
  let device = devices.into_iter().find(|device| device.mac_address == aeropex_mac);
  if device.is_none() {
      log::error!("Failed to find device with mac {:?}; nothing to do. exiting...", aeropex_mac);
      return
  }
  let aeropex_id = device.unwrap().id;

  let h1 = tokio::task::spawn( update_state_loop(sender, aeropex_id.clone(), ctx, bt_session.clone()) );
  let h2 = tokio::task::spawn( execute_command_loop(receiver, aeropex_id, bt_session) );

  if let Err( e ) = h1.await {
    log::warn!("update_state_loop task is failed.... {:?}", e);
  };
  if let Err( e ) = h2.await {
    log::warn!("execute_command_loop task is faield... {:?}", e);
  }
}

async fn update_state_loop(
  sender : Sender<HomeState>,
  aeropex_id : DeviceId,
  egui_ctx : Context,
  bt_session : BluetoothSession )
{
  let mut state = HomeState::default();
  state.is_aeropex_connected = check_bluetooth_status(&aeropex_id, &bt_session).await;
  match sender.try_send(state.clone()) {
      Ok(()) => egui_ctx.request_repaint(),
      Err( e ) => {
         log::error!("Failed to send inital data {:?}. Probably GUI is dead, exiting....", e);
         return;
      },
  };

  let event_stream = bt_session.device_event_stream(&aeropex_id).await;
  if let Err( e ) = event_stream {
    log::error!("Failed to get device_event_stream for id {:?} : {:?}", aeropex_id, e);
    return;
  };
  let mut event_stream = event_stream.unwrap();

  while let Some( event ) = event_stream.next().await {

    log::debug!("Got BT event {:?}", event);

    if let BluetoothEvent::Device { event : DeviceEvent::Connected{ connected }, .. } = event {
      state.is_aeropex_connected = connected;
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

  log::warn!("Event stream from BT is ended... strange... exitin...");
}

async fn execute_command_loop(
  mut receiver : Receiver<HomeCommand>,
  aeropex_id : DeviceId,
  bt_session : BluetoothSession
  )
{
  loop {
      match receiver.recv().await {
      Some( cmd ) => execute_command( &aeropex_id, &bt_session, cmd ).await,
      None => {
        log::warn!("Failed to receiver data, probably GUI is dead. Exiting...");
        break;
      },
     };

  }
}

async fn execute_command(aeropex_id : &DeviceId, bt_session : &BluetoothSession, cmd : HomeCommand)
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
      }
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
