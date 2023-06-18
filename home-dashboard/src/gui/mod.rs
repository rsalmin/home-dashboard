use eframe::egui;
use crate::egui::*;
use crate::egui::widget_text::RichText;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::sync::mpsc::error::TryRecvError;
use std::thread;
use log;
use egui_extras::{TableBuilder, Column, image::RetainedImage};
use std::fs;
use std::path::Path;

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
  arrow_up_image : Option<RetainedImage>,
  arrow_down_image : Option<RetainedImage>,
  stable_image : Option<RetainedImage>,
}

impl HomeDashboard {
  pub fn new(cc : &eframe::CreationContext<'_>, cfg : HomeDashboardConfig) -> Self {

    //if let Some( monitor_size ) = cc.integration_info.window_info.monitor_size {

        //const EXPECTED_POINT_HEIGHT : f32 = 720.0;
        //let ppp = monitor_size.y / EXPECTED_POINT_HEIGHT;
        //cc.egui_ctx.set_pixels_per_point( ppp );
        //log::debug!("Setting pixels_per_point  {}", ppp);
    //}
    log::debug!("HomeDashobard created with IntegragtionInfo {:?}", cc.integration_info);
    const MAX_NUM_MESSAGES : usize = 10;

    let (worker_sender, gui_receiver) = channel::<HomeState>(MAX_NUM_MESSAGES);
    let (gui_sender, worker_receiver) = channel::<HomeCommand>(MAX_NUM_MESSAGES);

    let ctx = cc.egui_ctx.clone();

    let mut style = (*ctx.style()).clone();
    style.visuals.selection.bg_fill = Color32::DARK_GREEN;
    ctx.set_style(style);

    // it detaches but we are control it via channels
    thread::spawn(move|| worker_thread(worker_sender, worker_receiver, ctx, cfg));

    let up_image =  read_svg_image_with_log(Path::new("up_arrow.svg"));
    let down_image =  read_svg_image_with_log(Path::new("down_arrow.svg"));
    let stable_image =  read_svg_image_with_log(Path::new("stable.svg"));

    HomeDashboard {
     state : HomeState::default(),
     gui_state : GUIState::default(),
     receiver : gui_receiver,
     sender : gui_sender,
     arrow_up_image : up_image,
     arrow_down_image: down_image,
     stable_image: stable_image,
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

  fn outdoor_group_table(&self, ui: &mut Ui, wd : &Option<WeatherData> ) {
    let name_texts = vec!["Temperature    ", "Humidity", "Pressure "];
    let unit_texts = vec!["°C", "%", "mmHg"];
    let text_colors = vec![Color32::GREEN, Color32::GREEN, Color32::GREEN];
    let text_sizes = vec![40.0, 40.0, 40.0];

    let mut data_texts = vec![String::new(); 3];
    let mut data_trends : Vec<Option<Trend>> = vec![None; 3];

    if let Some( wd ) = wd {

        if let Some( od ) = &wd.outdoor_weather {
            data_texts[0] = format!("{:.1}", od.temperature);
            data_trends[0] = od.temperature_trend.clone();
            data_texts[1] = format!("{}", od.humidity);
        }

        let pressure = wd.pressure / 1.333223684; //to mmHg
        data_texts[2] = format!("{:.1}", pressure);
        data_trends[2] =  wd.pressure_trend.clone();
    }

    ui.push_id("Outdoor Group Table", |ui| {
        TableBuilder::new(ui)
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .body(|body| {
                body.rows(60.0,  name_texts.len(), |row_index, mut row| {
                    let text_color = text_colors[row_index];
                    let text_size = text_sizes[row_index];
                    row.col(|ui| {
                        ui.label( RichText::new(name_texts[row_index]).heading().color(text_color).size(text_size) );
                    });
                    if let Some( txt ) = data_texts.get(row_index) {
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label( RichText::new(txt).heading().color(text_color).size(text_size) );
                             });
                        });
                        row.col(|ui| {
                            if let Some( trend ) = &data_trends[row_index] {
                                self.show_trend(ui, trend);
                            };
                        });
                        row.col(|ui| {
                            ui.label( RichText::new(unit_texts[row_index]).heading().color(text_color).size(text_size) );
                        });
                    };
                });
            });
      });
  }

  fn home_group_table(&self, ui: &mut Ui, wd : &Option<WeatherData> ) {
    let name_texts = vec!["Temperature    ", "Humidity", "CO2", "Noise"];
    let unit_texts = vec!["°C", "%", "ppm", "dB"];

    let mut data_texts = Vec::<String>::new();
    if let Some( wd ) = wd {
             data_texts.push( format!("{:.1}", wd.room_temperature) );
             data_texts.push( format!("{}", wd.room_humidity) );
             data_texts.push( format!("{}", wd.room_co2) );
             data_texts.push( format!("{}", wd.room_noise) );
    }

    ui.push_id("Home Group Table", |ui| {
        TableBuilder::new(ui)
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .body(|body| {
                body.rows(60.0,  name_texts.len(), |row_index, mut row| {
                    row.col(|ui| {
                        ui.label( RichText::new(name_texts[row_index]).heading().color(Color32::GREEN).size(40.0) );
                    });
                    if let Some( txt ) = data_texts.get(row_index) {
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label( RichText::new(txt).heading().color(Color32::GREEN).size(40.0) );
                             });
                        });
                        row.col(|ui| {
                            ui.label( RichText::new(unit_texts[row_index]).heading().color(Color32::GREEN).size(40.0) );
                        });
                    };
                });
            });
      });
  }

  fn show_trend(&self, ui : &mut Ui, trend : &Trend)
  {
    let scale = 0.5;

    let image = match trend {
       Trend::Stable => &self.stable_image,
       Trend::Down => &self.arrow_down_image,
       Trend::Up => &self.arrow_up_image,
    };

    if let Some( image ) = image {
        image.show_scaled(ui, scale);
    };
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
         self.home_group_table(ui, &self.state.weather_data);
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
         ui.add_visible(false, Separator::default());
         self.outdoor_group_table(ui, &self.state.weather_data);
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


fn read_svg_image_with_log(file_path : &Path) -> Option<RetainedImage>
{
    match fs::read(file_path) {
        Err( err ) => {log::error!("Failed to read {} : {}", file_path.display(), err); None},
        Ok( image_bytes ) => {
            match RetainedImage::from_svg_bytes("up arrow image", &image_bytes) {
                Err( err ) => { log::error!("Failed to convert {} content to svg image : {}", file_path.display(), err); None },
                Ok( svg_image ) => Some( svg_image),
            }
        },
    }
}
