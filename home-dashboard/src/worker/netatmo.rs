use reqwest;
use netatmo_connect::*;
use crate::interface::{WeatherData, OutdoorWeatherData, AirQualityData, Trend};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TrySendError;
use chrono::naive::NaiveDateTime;
use std::option::Option;

#[derive(Default)]
pub struct NetatmoData {
    pub weather_station : Option<WeatherData>,
    pub child_room_data : Option<AirQualityData>,
    pub office_room_data : Option<AirQualityData>,
}

pub async fn watch_netatmo_loop(
    netatmo_sender : Sender<NetatmoData> ,
    cfg : ConnectConfig) -> Result<(), String>
{
  let client = reqwest::Client::new();
  let timeout = Some( Duration::from_secs(1) );

  let mut token =  get_access_token(&client, &cfg, &timeout).await?;

  loop {
    if token.expires_at < Instant::now() {
      log::info!("Access token is expired!");
      token = get_fresh_token(&client, &cfg, &token, &timeout).await?;
    }

    let res = get_stations_data(&client, &token, &timeout).await?;

     let time_server = NaiveDateTime::from_timestamp_opt(res.time_server, 0);
     match time_server {
       None => println!("Failed to convert server time to NaiveDateTime"),
       Some( v ) => println!("server naive date time: {}", v),
     };

    let mut netatmo_data = NetatmoData::default();

    netatmo_data.weather_station = if res.body.devices.is_empty() {
        log::warn!("Can't find any device in netatmo data!");
        None
    } else {
        let device = &res.body.devices[0];

        let mut weather_data =  WeatherData::default();

        weather_data.room_temperature = device.dashboard_data.Temperature;
        weather_data.room_humidity = device.dashboard_data.Humidity;
        weather_data.room_co2 = device.dashboard_data.CO2;
        weather_data.room_noise = device.dashboard_data.Noise;
        weather_data.pressure = device.dashboard_data.Pressure;
        weather_data.pressure_trend = parse_trend(&device.dashboard_data.pressure_trend);
        //device.dashboard_data.temp_trend;

        weather_data.outdoor_weather = if device.modules.is_empty() {
            log::warn!("Can't find any outdoor modules within device");
            None
        } else {
            let module = &device.modules[0];

            Some( OutdoorWeatherData {
                temperature : module.dashboard_data.Temperature,
                temperature_trend : parse_trend(&module.dashboard_data.temp_trend),
                humidity : module.dashboard_data.Humidity,
             })
        };

        Some( weather_data )
    };

    let res = get_homecoachs_data(&client, &token, &timeout).await?;
    for d in res.body.devices {
        if d.station_name == "Переговорка" {
            netatmo_data.office_room_data = Some( from_dashboard_data( &d.dashboard_data ) );
        }
        if d.station_name == "Детская" {
            netatmo_data.child_room_data = Some( from_dashboard_data( &d.dashboard_data ) );
        }
    }

    match netatmo_sender.try_send(netatmo_data) {
      Ok(()) => (),
      Err( TrySendError::Full( _ ) ) => log::warn!("Failed to send weather data, update_state_loop is not consuming it!"),
      Err( TrySendError::Closed( _ ) ) => {
        log::warn!("Failed to send weather data - channel is closed. Probably update_state_loop is dead now. Exiting....");
        return Ok(());
      },
   }

     tokio::time::sleep(Duration::from_secs(60)).await;
   };

  //log::warn!("Event stream from BT is ended... strange... exiting...");
  //Ok(())
}

fn parse_trend(str : &str) -> Option<Trend>
{
  if str == "up" { return Some(Trend::Up); }
  if str == "down" { return Some(Trend::Down); }
  if str == "stable" { return Some(Trend::Stable); }

  log::error!("Unknown string for describing Trend: {}", str);
  None
}


fn from_dashboard_data( device_data : &HomeCoachsDeviceData ) -> AirQualityData
{
    let mut data = AirQualityData::default();
    data.room_temperature = device_data.Temperature;
    data.room_humidity = device_data.Humidity;
    data.room_co2 = device_data.CO2;
    data.room_noise = device_data.Noise;

    data
}