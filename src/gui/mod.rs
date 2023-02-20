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

    //only last message from channel is actual
    loop {
      match self.receiver.try_recv() {
        Ok( state ) => { self.state = state; },
        Err( TryRecvError::Disconnected ) => {
          log::error!("Worker thread is dead. Closing...");
          frame.close();
          break;
        },
        _ => break,
      };
    };

    let Vec2 {x : frame_width, y : frame_height} = ctx.screen_rect().size();
     egui::CentralPanel::default().show(ctx, |ui| {

      let aeropex_state_string = if self.state.is_aeropex_connected {
          String::from("Connected")
        } else {
          String::from("Disconnected")
        };

     Grid::new("unique grid")
       .min_col_width(frame_width / 3.0)
       .min_row_height(frame_height / 3.0)
       .num_columns(3)
       .show(ui, |ui| {
         ui.end_row();
         ui.add_visible(false, Separator::default());
         ui.group(|ui| {
            ui.vertical(|ui| {
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
          });
         ui.end_row();
         ui.end_row();
      });

      if ui.ctx().input( |i| i.key_pressed(Key::Q) )   {
        frame.close();
      }
    });
  }

}

