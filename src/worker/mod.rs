use crate::egui::Context; // b/c of re-export
use std::sync::mpsc::{Sender, Receiver, RecvTimeoutError};
use std::time::Duration;
use log;
use std::process::Command;
use std::str;

use crate::interface::*;

struct Configuration {
  aeropex_id : String,
}

impl Configuration {
 fn new() -> Self {
   Configuration { aeropex_id : String::from("20:74:CF:BD:61:41") }
 }
}

pub fn worker_thread(sender : Sender<HomeState>, receiver : Receiver<HomeCommand>, ctx : Context) {

  let cfg = Configuration::new();
  let mut state = HomeState::default();

  loop {

    match receiver.recv_timeout( Duration::from_secs(1) ) {
      Ok( cmd ) => execute_command( &cfg, cmd ),
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

    state.is_aeropex_connected = check_bluetooth_status(&cfg);
  }
}

fn execute_command(cfg : &Configuration, cmd : HomeCommand)
{
  log::debug!("Got CMD: {:?}", cmd);
  match cmd {
    HomeCommand::ConnectAeropex =>
      execute_shell_command("bluetoothctl", &["connect", &cfg.aeropex_id]),
    HomeCommand::DisconnectAeropex =>
      execute_shell_command("bluetoothctl", &["disconnect", &cfg.aeropex_id]),
  };
}

fn execute_shell_command(cmd : &str, args : &[&str]) -> String {
  let result = Command::new(cmd).args(args).output();
  match result {
    Err( e ) => { log::error!("Error executing command {} {:?} : {}", cmd, args, e); String::new() },
    Ok( output ) => {
      match str::from_utf8(&output.stdout) {
        Ok( r ) => String::from(r) ,
        Err( r ) => { log::error!("Invalid utf8 from shell command {} {:?}: {}", cmd, args, r); String::new() },
      }
    }
  }
}

fn check_bluetooth_status(cfg : &Configuration) -> bool {
  let output = execute_shell_command("bluetoothctl", &["devices", "Connected"]);

  let mut is_connected = false;
  for  s in output.lines()  {
    if let Some( id ) = s.split_whitespace().nth(1) {
      if id == cfg.aeropex_id {
        is_connected = true;
      }
    }
  };

  is_connected
}
