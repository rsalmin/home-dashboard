use log;
use ddc_hi::{Ddc, Display};
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TrySendError;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Preset {
    Standart,
    Comfort,
    Movie,
    Game,
    Unknown{ val_dc: u16, val_f0 : u16 },
}

#[derive(Debug, Clone)]
pub struct DisplayState {
  pub brightness : u16,
  pub preset: Preset,
}

pub fn watch_ddc_display_loop(
    display_sender : Sender<DisplayState>) -> Result<(), String>
{
    let prefered_model = "DELL U3421WE";
    let mut displays : Vec<Display> = Display::enumerate();
    let display : &mut Display  = find_display(&mut displays, &prefered_model)?;

    //display.update_capabilities().unwrap();
    log::info!("Found display {}", display_string(display));

    loop {
        let brightness = get_brightness(display)?;
        let preset = get_preset(display)?;
        log::info!("Brightness: {}, Preset {:?}", brightness, preset);

        let ds = DisplayState{ brightness, preset };
        match display_sender.try_send(ds) {
            Ok(()) => (),
            Err( TrySendError::Full( _ ) ) => log::warn!("Failed to send display state, update_state_loop is not consuming it!"),
            Err( TrySendError::Closed( _ ) ) => {
                log::warn!("Failed to send display state - channel is closed. Probably update_state_loop is dead now. Exiting....");
                break;
            },
        }
        sleep(Duration::from_millis(2000));
    }

  log::warn!("watch_ddc_display_loop finsied");
  Ok(())
}

fn get_brightness(display : &mut Display) -> Result<u16, String>
{
    match display.handle.get_vcp_feature(0x10) {
        Err( e ) => Err( e.to_string() ),
        Ok( v ) => Ok( v.value() ),
    }
}

fn set_brightness(display : &mut Display, val : u16) -> Result<(), String>
{
    display.handle.set_vcp_feature(0x10, val).map_err(|e| e.to_string())
}

fn get_preset(display : &mut Display) -> Result<Preset, String>
{
    let val_dc = display.handle.get_vcp_feature(0xDC).map_err(|e| e.to_string())?.value();
    //let val_e2 = display.handle.get_vcp_feature(0xE2).map_err(|e| e.to_string())?.value();
    let val_f0 = display.handle.get_vcp_feature(0xF0).map_err(|e| e.to_string())?.value();

    Ok( match (val_dc, val_f0) {
        (0, 0) => Preset::Standart,
        (0, 0xC) => Preset::Comfort,
        (3, 0) => Preset::Movie,
        (5, 0) => Preset::Game,
        (val_dc, val_f0) => Preset::Unknown{val_dc, val_f0},
    })
}

fn set_preset(display : &mut Display, preset : Preset) -> Result<(), String>
{
    match preset {
        Preset::Standart => display.handle.set_vcp_feature(0xDC, 0).map_err(|e| e.to_string()),
        Preset::Comfort  => display.handle.set_vcp_feature(0xF0, 0xC).map_err(|e| e.to_string()),
        Preset::Movie => display.handle.set_vcp_feature(0xDC, 3).map_err(|e| e.to_string()),
        Preset::Game => display.handle.set_vcp_feature(0xDC, 5).map_err(|e| e.to_string()),
        Preset::Unknown{..} => Err(String::from("seting of Unknown  presets are not supported!")),
    }
}

fn find_display<'a>(displays : &'a mut [Display], prefered_model: &str) -> Result<&'a mut Display, String>
{
    if displays.is_empty() {
        return Err( String::from("Can't find any DDC displays") );
    }

    let idx =
        if let Some(i) = displays.iter().position(|d| d.info.model_name.is_some() && d.info.model_name.as_ref().unwrap() == prefered_model) {
            i
        } else {
            0
        };

    Ok( &mut displays[idx] )
}

fn display_string(display :&Display) -> String
{
    let mut str = format!("Id:{}", display.info.id);
    if let Some( man ) = &display.info.manufacturer_id {
        str += &format!("  Manufacturer:{}", man);
    }
    if let Some( model ) = &display.info.model_name {
        str += &format!("  Model:{}", model);
    }
    str
}