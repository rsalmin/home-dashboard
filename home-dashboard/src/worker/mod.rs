use crate::egui::Context; // b/c of re-export
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::sync::mpsc::error::TrySendError;
use tokio;
use log;
use bluez_async::{BluetoothEvent, DeviceEvent};
use crate::interface::*;
use futures::stream::StreamExt;

mod bluetooth;
use bluetooth::*;

#[tokio::main]
pub async fn worker_thread(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context, cfg : HomeDashboardConfig) {
  let result = worker_thread_prime(sender, receiver, ctx, cfg).await;
  if let Err ( e ) = result {
    log::error!("Error in worker_thread : {}. exiting....", e);
  }
}

pub async fn worker_thread_prime(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context, cfg : HomeDashboardConfig) -> Result<(), String> {

  let bt_module = BluetoothModule::new(&cfg.bt_config).await?;

  let h1 = tokio::task::spawn( update_state_loop(sender, bt_module.clone(), ctx) );
  let h2 = tokio::task::spawn( execute_command_loop(receiver, bt_module) );

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
  bt_module : BluetoothModule,
  egui_ctx : Context) -> Result<(), String>
{
  let mut state = HomeState::default();

  state.bt_state = bt_module.get_state().await;

  sender.try_send(state.clone()).map_err(|x| x.to_string())?;
  egui_ctx.request_repaint();

  //FIXME: if devices switched it's state between initial state request and event_stream loop, we will have a problem
  let mut aeropex_event_stream = bt_module.aeropex_event_stream().await?;
  let mut edifier_event_stream = bt_module.edifier_event_stream().await?;

  loop {
    tokio::select! {
      Some( event ) = aeropex_event_stream.next() => {
        log::debug!("Got BT event {:?}", event);
        if let BluetoothEvent::Device { event : DeviceEvent::Connected{ connected }, .. } = event {
          state.bt_state.is_aeropex_connected = connected;
        }
      }
      Some( event ) = edifier_event_stream.next() => {
        log::debug!("Got BT event {:?}", event);
        if let BluetoothEvent::Device { event : DeviceEvent::Connected{ connected }, .. } = event {
          state.bt_state.is_edifier_connected = connected;
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
  bt_module : BluetoothModule,
  )
{
  loop {
      match receiver.recv().await {
      Some( cmd ) => execute_command( &bt_module, cmd ).await,
      None => {
        log::warn!("Failed to receiver data, probably GUI is dead. Exiting...");
        break;
      },
     };

  }
}

