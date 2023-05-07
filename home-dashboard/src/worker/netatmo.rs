use reqwest;
use netatmo_connect::*;
use crate::interface::WeatherData;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TrySendError;
use chrono::naive::NaiveDateTime;

pub async fn watch_netatmo_loop(
    netatmo_sender : Sender<WeatherData> ,
    cfg : ConnectConfig) -> Result<(), String>
{
  let client = reqwest::Client::new();

  let mut token =  get_access_token(&client, &cfg).await?;

  let token_duration = token.expires_at - Instant::now();

  let mut weather_data =  WeatherData::default();

  loop {
    match netatmo_sender.try_send(weather_data.clone()) {
      Ok(()) => (),
      Err( TrySendError::Full( _ ) ) => log::warn!("Failed to send weather data, update_state_loop is not consuming it!"),
      Err( TrySendError::Closed( _ ) ) => {
        log::warn!("Failed to send weather data - channel is closed. Probably update_state_loop is dead now. Exiting....");
        return Ok(());
      },
   }

    if token.expires_at < Instant::now() {
      log::info!("Access token is expired!");
      token = get_fresh_token(&client, &cfg, &token).await?;
    }

    let res = get_stations_data(&client, &token).await?;

     let time_server = NaiveDateTime::from_timestamp_opt(res.time_server, 0);
     match time_server {
       None => println!("Failed to convert server time to NaiveDateTime"),
       Some( v ) => println!("server naive date time: {}", v),
     };

     for d in res.body.devices {
       println!("Device id : {}", d._id);
       println!("data : {}", d.dashboard_data);
       for m in d.modules {
         println!("  Module : {}", m._id);
         println!("  Battery : {}%", m.battery_percent);
         println!("  data : {}", m.dashboard_data);
       }
     };

     tokio::time::sleep(Duration::from_secs(60)).await;
   };

  //log::warn!("Event stream from BT is ended... strange... exiting...");
  Ok(())
}
