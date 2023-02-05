use eframe::egui;
use egui::*;
use std::sync::mpsc::{channel, Sender, Receiver, RecvTimeoutError, TryRecvError};
use std::thread;
use std::time::Duration;

fn main() {

  let mut native_options = eframe::NativeOptions::default();
  native_options.fullscreen = true;

  eframe::run_native(
    "My egui App",
    native_options,
    Box::new(|cc| Box::new(MyEguiApp::new(cc)) )
  );
}

fn worker_thread(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : egui::Context) {
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


#[derive(Default, Clone)]
struct HomeState {
  is_aeropex_connected : bool,
}

#[derive(Debug)]
enum HomeCommand {
  ConnectAeropex,
  DisconnectAeropex,
}

struct MyEguiApp {
  state : HomeState,
  receiver : Receiver<HomeState>,
  sender : Sender<HomeCommand>,
}

impl MyEguiApp {
  fn new(cc : &eframe::CreationContext<'_>) -> Self {

    let (worker_sender, gui_receiver) = channel::<HomeState>();
    let (gui_sender, worker_receiver) = channel::<HomeCommand>();

    let ctx = cc.egui_ctx.clone();
    // it detaches but we are control it via channels
    thread::spawn(move|| worker_thread(worker_sender, worker_receiver, ctx));

    MyEguiApp {
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

impl eframe::App for MyEguiApp {
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

