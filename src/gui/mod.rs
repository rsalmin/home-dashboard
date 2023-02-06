use eframe::egui;
use crate::egui::*;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::sync::mpsc::error::TryRecvError;
use std::thread;
use log;

use crate::interface::*;
use crate::worker::worker_thread;

pub struct HomeDashboard {
  state : HomeState,
  receiver : Receiver<HomeState>,
  sender : Sender<HomeCommand>,
}

impl HomeDashboard {
  pub fn new(cc : &eframe::CreationContext<'_>) -> Self {

    const MAX_NUM_MESSAGES : usize = 10;

    let (worker_sender, gui_receiver) = channel::<HomeState>(MAX_NUM_MESSAGES);
    let (gui_sender, worker_receiver) = channel::<HomeCommand>(MAX_NUM_MESSAGES);

    let ctx = cc.egui_ctx.clone();
    // it detaches but we are control it via channels
    thread::spawn(move|| worker_thread(worker_sender, worker_receiver, ctx));

    HomeDashboard {
     state : HomeState::default(),
     receiver : gui_receiver,
     sender : gui_sender,
   }
  }

  fn send_command(&self, cmd : HomeCommand) {
    if let Err( err ) = self.sender.try_send( cmd ) {
      log::error!("Failed to send {:?} command. Ignoring.", err);
    }
  }
}

impl eframe::App for HomeDashboard {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

    match self.receiver.try_recv() {
      Ok( state ) => { self.state = state; },
      Err( TryRecvError::Disconnected ) => {
        log::error!("Worker thread is dead. Closing...");
        frame.close();
      },
      _ => (),
    };

    egui::CentralPanel::default().show(ctx, |ui| {

      let aeropex_state_string = if self.state.is_aeropex_connected {
          String::from("Connected")
        } else {
          String::from("Disconnected")
        };

      ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
          ui.heading("Aeropex :");
          ui.heading(aeropex_state_string);
        });
        ui.horizontal(|ui| {
          if ui.button("Connect").clicked() {
             self.send_command( HomeCommand::ConnectAeropex );
          }
          if ui.button("Disconnect").clicked() {
             self.send_command( HomeCommand::DisconnectAeropex );
          }
        });
      });

      if ui.ctx().input().key_pressed(Key::Q)   {
        frame.close();
      }
    });
  }

}

