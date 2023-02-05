use crate::egui::Context; // b/c of re-export
use std::sync::mpsc::{Sender, Receiver, RecvTimeoutError};
use std::time::Duration;

use crate::interface::*;

pub fn worker_thread(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context) {
  let mut state = HomeState::default();
  loop {

    match receiver.recv_timeout( Duration::from_secs(1) ) {
      Ok( cmd ) => { println!("Got CMD: {:?}", cmd) },
      Err( RecvTimeoutError::Disconnected ) => {
        println!("Failed to receiver data, probably GUI is dead. Exiting...");
        break;
      },
      _ => (),
    };

    if let Err( _ ) = sender.send(state.clone()) {
      println!("Failed to send data, probably GUI is dead, exiting....");
      break;
    }
    ctx.request_repaint();

    state.is_aeropex_connected = ! state.is_aeropex_connected;
  }
}
