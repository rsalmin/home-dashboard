use crate::egui::Context; // b/c of re-export
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::sync::mpsc::error::TrySendError;
use tokio::time::{sleep, Duration};
use log;
use std::process::Command;
use std::str;
use tokio;

use crate::interface::*;

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

  let h1 = tokio::task::spawn( update_state_loop(sender, cfg.clone(), ctx) );
  let h2 = tokio::task::spawn( execute_command_loop(receiver, cfg) );

  if let Err( e ) = h1.await {
    log::warn!("update_state_loop task is failed.... {:?}", e);
  };
  if let Err( e ) = h2.await {
    log::warn!("execute_command_loop task is faield... {:?}", e);
  }
}

async fn update_state_loop(sender : Sender<HomeState>, cfg : Configuration, ctx : Context)
{
  let mut state = HomeState::default();

  loop {
    match sender.try_send(state.clone()) {
      Ok(()) => ctx.request_repaint(),
      Err( TrySendError::Full( _ ) ) => log::warn!("Failed to send data, GUI is not consuming it!"),
      Err( TrySendError::Closed( _ ) ) => {
        log::warn!("Failed to send data - channel is closed. Probably GUI is dead, exiting....");
        break;
      },
    }

    sleep(Duration::from_millis(333)).await;

    state.is_aeropex_connected = check_bluetooth_status(&cfg);


  }
}

async fn execute_command_loop(mut receiver : Receiver<HomeCommand>, cfg : Configuration)
{
  loop {
      match receiver.recv().await {
      Some( cmd ) => execute_command( &cfg, cmd ),
      None => {
        log::warn!("Failed to receiver data, probably GUI is dead. Exiting...");
        break;
      },
     };

  }
}

fn execute_command(cfg : &Configuration, cmd : HomeCommand)
{
  log::debug!("Got CMD: {:?}", cmd);
  let out = match cmd {
    HomeCommand::ConnectAeropex =>
      execute_shell_command("bluetoothctl", &["connect", &cfg.aeropex_id]),
    HomeCommand::DisconnectAeropex =>
      execute_shell_command("bluetoothctl", &["disconnect", &cfg.aeropex_id]),
  };
  log::debug!("CMD output: {}", out);
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
