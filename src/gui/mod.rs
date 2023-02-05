use eframe::egui;
use crate::egui::*;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;

use crate::interface::*;
use crate::worker::worker_thread;

pub struct HomeDashboard {
  state : HomeState,
  receiver : Receiver<HomeState>,
  sender : Sender<HomeCommand>,
}

impl HomeDashboard {
  pub fn new(cc : &eframe::CreationContext<'_>) -> Self {

    let (worker_sender, gui_receiver) = channel::<HomeState>();
    let (gui_sender, worker_receiver) = channel::<HomeCommand>();

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
    if let Err( err ) = self.sender.send( cmd ) {
      println!("Failed to send {:?} command. Ignoring.", err.0);
    }
  }
}

impl eframe::App for HomeDashboard {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

    match self.receiver.try_recv() {
      Ok( state ) => { self.state = state; },
      Err( TryRecvError::Disconnected ) => {
        println!("Worker thread is dead. Closing...");
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

