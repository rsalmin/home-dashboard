use crate::egui::Context; // b/c of re-export
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::sync::mpsc::error::TrySendError;
use tokio;
use log;
use crate::interface::*;
use std::thread;

mod bluetooth;
mod netatmo;
pub mod ddc_display;
use bluetooth::*;
use netatmo::*;
use ddc_display::*;

#[tokio::main]
pub async fn worker_thread(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context, cfg : HomeDashboardConfig) {
  let result = worker_thread_prime(sender, receiver, ctx, cfg).await;
  if let Err ( e ) = result {
    log::error!("Error in worker_thread : {}. exiting....", e);
  }
}

pub async fn worker_thread_prime(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context, cfg : HomeDashboardConfig) -> Result<(), String> {

  let bt_module = BluetoothModule::new(&cfg.bt_config).await?;

  const MAX_NUM_MESSAGES : usize = 5;
  let (bt_sender, bt_receiver) = channel::<BluetoothState>(MAX_NUM_MESSAGES);
  let (netatmo_sender, netatmo_receiver) = channel::<NetatmoData>(MAX_NUM_MESSAGES);
  let (display_sender, display_receiver) = channel::<DisplayState>(MAX_NUM_MESSAGES);

  let h1 = tokio::task::spawn( update_state_loop(sender, bt_receiver, netatmo_receiver, display_receiver, ctx) );
  let h3 = tokio::task::spawn( watch_bluetooth_loop(bt_module.clone(), bt_sender) );
  let h2 = tokio::task::spawn( execute_command_loop(receiver, bt_module) );
  let h4 = tokio::task::spawn ( watch_netatmo_loop(netatmo_sender, cfg.connect_config.clone()) );
  let h5 = thread::spawn( ||
      {
          if let Err( e ) = watch_ddc_display_loop(display_sender) {
              log::error!("watch_ddc_display_loop finised with error: {}", e);
          };
       } );

  match h1.await {
    Err( e ) => log::warn!("update_state_loop task is faield... {:?}", e),
    Ok( o ) => o?,
  }
  if let Err( e ) = h2.await {
    log::warn!("execute_command_loop task is faield... {:?}", e);
  }
  if let Err( e ) = h3.await {
    log::warn!("watch_bluetooth_loop task is faield... {:?}", e);
  }
  if let Err( e ) = h4.await {
    log::warn!("watch_netatmo_loop task is faield... {:?}", e);
  }
  if let Err( e ) = h5.join() {
    log::warn!("watch_ddc_display_loop task is faield... {:?}", e);
  }

  Ok(())
}

async fn update_state_loop(
  sender : Sender<HomeState>,
  mut bt_receiver : Receiver<BluetoothState>,
  mut netatmo_receiver : Receiver<NetatmoData>,
  mut display_receiver : Receiver<DisplayState>,
  egui_ctx : Context) -> Result<(), String>
{
  let mut state = HomeState::default();

  loop {
    match sender.try_send(state.clone()) {
      Ok(()) => egui_ctx.request_repaint(),
      Err( TrySendError::Full( _ ) ) => log::warn!("Failed to send data, GUI is not consuming it!"),
      Err( TrySendError::Closed( _ ) ) => {
        log::warn!("Failed to send data - channel is closed. Probably GUI is dead, exiting....");
        break;
      },
    }

    tokio::select! {
      Some( bt_state ) = bt_receiver.recv() => {
          state.bt_state = bt_state;
      }
      Some( netatmo_data ) = netatmo_receiver.recv() => {
          state.weather_data = netatmo_data.weather_station;
          state.child_room_data = netatmo_data.child_room_data;
          state.office_room_data = netatmo_data.office_room_data;
      }
      Some( display_state ) = display_receiver.recv() => {
          state.display_state = Some( display_state );
      }
     else => { break; }
    }

  }

  log::warn!("One of data streams is ended... strange... exiting...");
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

