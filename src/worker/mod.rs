use crate::egui::Context; // b/c of re-export
use std::sync::mpsc::{Sender, Receiver, RecvTimeoutError};
use std::time::Duration;
use log;

use crate::interface::*;

pub fn worker_thread(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context) {
  let mut state = HomeState::default();
  loop {

    match receiver.recv_timeout( Duration::from_secs(1) ) {
      Ok( cmd ) => { log::info!("Got CMD: {:?}", cmd) },
      Err( RecvTimeoutError::Disconnected ) => {
        log::warn!("Failed to receiver data, probably GUI is dead. Exiting...");
        break;
      },
      _ => (),
    };

    if let Err( _ ) = sender.send(state.clone()) {
      log::warn!("Failed to send data, probably GUI is dead, exiting....");
      break;
    }
    ctx.request_repaint();

    state.is_aeropex_connected = ! state.is_aeropex_connected;
  }
}
