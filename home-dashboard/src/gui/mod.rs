use eframe::egui;
use crate::egui::*;
use crate::egui::widget_text::RichText;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::sync::mpsc::error::TryRecvError;
use std::thread;
use log;

use crate::interface::*;
use crate::worker::worker_thread;

#[derive(Default)]
pub struct GUIState {
  aeropex_switch_state : bool,
  edifier_switch_state : bool,
}

pub struct HomeDashboard {
  state : HomeState,
  gui_state : GUIState,
  receiver : Receiver<HomeState>,
  sender : Sender<HomeCommand>,
}

impl HomeDashboard {
  pub fn new(cc : &eframe::CreationContext<'_>, cfg : HomeDashboardConfig) -> Self {

    const MAX_NUM_MESSAGES : usize = 10;

    let (worker_sender, gui_receiver) = channel::<HomeState>(MAX_NUM_MESSAGES);
    let (gui_sender, worker_receiver) = channel::<HomeCommand>(MAX_NUM_MESSAGES);

    let ctx = cc.egui_ctx.clone();

    let mut style = (*ctx.style()).clone();
    style.visuals.selection.bg_fill = Color32::DARK_GREEN;
    ctx.set_style(style);

    // it detaches but we are control it via channels
    thread::spawn(move|| worker_thread(worker_sender, worker_receiver, ctx, cfg));

    HomeDashboard {
     state : HomeState::default(),
     gui_state : GUIState::default(),
     receiver : gui_receiver,
     sender : gui_sender,
   }
  }

  fn send_command(&self, cmd : HomeCommand) {
    if let Err( err ) = self.sender.try_send( cmd ) {
      log::error!("Failed to send {:?} command. Ignoring.", err);
    }
  }

  fn bt_group(&self,
    ui: &mut Ui,
    label : &str,
    connect_state : bool,
    switch_state : bool,
    connect_command : HomeCommand,
    disconnect_command : HomeCommand) -> bool {

      let aeropex_state_text = if connect_state {
            RichText::new("Connected").heading().color(Color32::GREEN)
          } else {
            RichText::new("Disconnected").heading()
      };

      let mut switch_state = switch_state;

      ui.group(|ui| {
        ui.vertical_centered(|ui| {
          ui.heading(label);
          ui.label( aeropex_state_text );
          if switch_button(ui, &mut switch_state).clicked() {
            if switch_state {
              self.send_command( connect_command );
            } else {
              self.send_command( disconnect_command );
            }
          }
        });
      });

     switch_state
  }

  fn outdoor_group(&self, ui: &mut Ui, wd : &Option<WeatherData> ) {
     let mut texts = Vec::<String>::new();
     let mut bat_text = String::new();

     if let Some( wd ) = wd {

         if let Some( od ) = &wd.outdoor_weather {
             texts.push(format!("{:.1} °C {}", od.temperature, show_trend(&od.temperature_trend)));
             texts.push(format!("{} %", od.humidity));
             bat_text = format!("battery: {}%", od.battery);
         }

         let pressure = wd.pressure / 1.333223684; //to mmHg
         texts.push(format!("{:.1} mmHg {}", pressure, show_trend(&wd.pressure_trend)));


     }

      let rich_texts : Vec::<RichText> = texts.iter().map(|t| RichText::new(t).heading().color(Color32::GREEN).size(40.0)).collect();

      ui.group(|ui| {
        ui.vertical_centered(|ui| {
          for rt in rich_texts {
            ui.add_visible(false, Separator::default());
            ui.add_visible(false, Separator::default());
            ui.add_visible(false, Separator::default());
            ui.label( rt );
          }
          ui.add_visible(false, Separator::default());
          ui.add_visible(false, Separator::default());
          ui.add_visible(false, Separator::default());
          ui.label( bat_text );
        })
      });
  }

  fn home_group(&self, ui: &mut Ui, wd : &Option<WeatherData> ) {
     let mut texts = Vec::<String>::new();

     if let Some( wd ) = wd {
             texts.push( format!("{:.1} °C", wd.room_temperature) );
             texts.push( format!("{} %", wd.room_humidity) );
             texts.push( format!("{} ppm", wd.room_co2) );
             texts.push( format!("{} dB", wd.room_noise) );
     }

      ui.group(|ui| {
        ui.vertical_centered(|ui| {
          for txt in texts {
            ui.add_visible(false, Separator::default());
            ui.add_visible(false, Separator::default());
            ui.add_visible(false, Separator::default());
            ui.label( RichText::new(txt).heading().color(Color32::GREEN).size(40.0) );
          }
        })
      });
  }
}

impl eframe::App for HomeDashboard {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

    log::debug!("screen_rect {:?}", ctx.screen_rect());
    log::debug!("available_rect {:?}", ctx.available_rect());
    log::debug!("pixels_per_point {}", ctx.pixels_per_point());

    //only last message from channel is actual
    let mut new_state : Option<HomeState> = None;
    loop {
      match self.receiver.try_recv() {
        Ok( state ) => {
            new_state = Some( state );
            log::debug!("recv: {:?}", new_state);
         },
        Err( TryRecvError::Disconnected ) => {
          log::error!("Worker thread is dead. Closing...");
          frame.close();
          break;
        },
        _ => break,
      }
    }

    if let Some( new_state ) = new_state {
      if  self.state.bt_state.is_aeropex_connected != new_state.bt_state.is_aeropex_connected {
        self.gui_state.aeropex_switch_state = new_state.bt_state.is_aeropex_connected;
      }
      if  self.state.bt_state.is_edifier_connected != new_state.bt_state.is_edifier_connected {
        self.gui_state.edifier_switch_state = new_state.bt_state.is_edifier_connected;
      }
      self.state = new_state;
    }

    let Vec2 {x : frame_width, y : frame_height} = ctx.screen_rect().size();
    egui::CentralPanel::default().show(ctx, |ui| {
      Grid::new("unique grid")
       .min_col_width(frame_width / 6.0)
       .min_row_height(frame_height / 3.0)
       .num_columns(6)
       .show(ui, |ui| {
         ui.end_row();
         ui.add_visible(false, Separator::default());
         self.home_group(ui, &self.state.weather_data);
         ui.add_visible(false, Separator::default());

         let new_switch_state = self.bt_group(ui, "Aeropex",
           self.state.bt_state.is_aeropex_connected,
           self.gui_state.aeropex_switch_state,
           HomeCommand::ConnectAeropex,
           HomeCommand::DisconnectAeropex,
         );
         self.gui_state.aeropex_switch_state = new_switch_state;

         let new_switch_state = self.bt_group(ui, "Edifier",
           self.state.bt_state.is_edifier_connected,
           self.gui_state.edifier_switch_state,
           HomeCommand::ConnectEdifier,
           HomeCommand::DisconnectEdifier,
         );
         self.gui_state.edifier_switch_state = new_switch_state;

         ui.end_row();
         for _ in 1..5 {
             ui.add_visible(false, Separator::default());
         }
         self.outdoor_group(ui, &self.state.weather_data);
         ui.end_row();
      });

      if ui.ctx().input( |i| i.key_pressed(Key::Q) )   {
        frame.close();
      }
    });
  }

}

fn switch_button(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(1.0, 2.5);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().visuals.widgets.style(&response);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.width();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_y = egui::lerp((rect.bottom() - radius)..=(rect.top() + radius), how_on);
        let center = egui::pos2(rect.center().x, circle_y);

        let bg_c = Color32::GRAY;
        let mut stroke = visuals.fg_stroke.clone();
        stroke.color = bg_c;
        ui.painter()
            .circle(center, 0.85 * radius, bg_c, stroke);
    }

    response
}


fn show_trend(t : &Option<Trend>) -> String
{
  match t {
       None => String::from("(?)"),
       Some( t ) =>
           match t {
               Trend::Stable => String::from("="),
               Trend::Up => String::from("/|\\"),
               Trend::Down => String::from("\\|/"),
           },
  }
}