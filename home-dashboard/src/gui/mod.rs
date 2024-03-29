use egui::*;
use egui::widget_text::RichText;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::sync::mpsc::error::TryRecvError;
use std::thread;
use log;
use egui_extras::{TableBuilder, Column};
use std::path::Path;

use crate::interface::*;
use crate::worker::worker_thread;
use crate::worker::ddc_display::DisplayState;

mod images;
use images::Images;

mod texts;
use texts::{Texts, Language};

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
  images : Images,
  texts : Texts,
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

    HomeDashboard {
     state : HomeState::default(),
     gui_state : GUIState::default(),
     receiver : gui_receiver,
     sender : gui_sender,
     images : Images::new(Path::new("home-dashboard/resources")),
     texts : Texts::new(Language::Russian),
   }
  }

  fn send_command(&self, cmd : HomeCommand) {
    if let Err( err ) = self.sender.try_send( cmd ) {
      log::error!("Failed to send {:?} command. Ignoring.", err);
    }
  }

  fn bt_switch(&self,
    ui: &mut Ui,
    width : f32,
    label : &str,
    connect_state : bool,
    switch_state : bool,
    connect_command : HomeCommand,
    disconnect_command : HomeCommand) -> bool {

        let mut switch_state = switch_state;

        ui.allocate_ui(Vec2::new(width, 400.0), |ui| {
            ui.vertical_centered(|ui| {
                let text_color = Color32::from_rgb(242, 174, 73);
                ui.label( RichText::new(label).color(text_color).heading() );
                ui.add_visible(false, Separator::default());
                indicator(ui, connect_state);
                ui.add_visible(false, Separator::default());
                if switch_button(ui, &mut switch_state, label).clicked() {
                    if switch_state {
                        self.send_command( connect_command );
                    } else {
                        self.send_command( disconnect_command );
                    }
                }
            })
        });

     switch_state
  }

  fn bt_group(&mut self, ui: &mut Ui)
  {
    let title_color = Color32::from_rgb(105, 209, 203);
    ui.vertical_centered(|ui| {
        ui.group(|ui| {
            ui.label( RichText::new("Аудио").heading().color(title_color).size(20.0) );
        });
        ui.horizontal_centered(|ui| {
            let w = ui.available_width();
            ui.add_visible(false, Separator::default().spacing(w/4.0) );
            let new_switch_state = self.bt_switch(ui, w/4.0, "AEROPEX",
              self.state.bt_state.is_aeropex_connected,
              self.gui_state.aeropex_switch_state,
              HomeCommand::ConnectAeropex,
              HomeCommand::DisconnectAeropex,
            );
            self.gui_state.aeropex_switch_state = new_switch_state;

            let new_switch_state = self.bt_switch(ui, w/4.0, "EDIFIER",
              self.state.bt_state.is_edifier_connected,
              self.gui_state.edifier_switch_state,
              HomeCommand::ConnectEdifier,
              HomeCommand::DisconnectEdifier,
            );
            self.gui_state.edifier_switch_state = new_switch_state;
       });
    });
  }

  fn outdoor_group_table(&self, ui: &mut Ui, wd : &Option<WeatherData> ) {
    let name_texts = vec![self.texts.temperature(), self.texts.humidity(), self.texts.pressure()];
    let unit_texts = vec!["°C", "%", "mmHg"];
    let text_sizes = vec![40.0, 40.0, 40.0];
    let text_color = Color32::from_rgb(242, 174, 73);
    let data_color = Color32::GREEN;
    let title_color = Color32::from_rgb(105, 209, 203);

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
        ui.vertical_centered(|ui| {
            ui.group(|ui| {
                    ui.label( RichText::new("Во дворе").heading().color(title_color).size(20.0) );
            });
            let w = ui.available_width();
            TableBuilder::new(ui)
                .column( Column::exact(w/2.) )
                .column( Column::exact(w/6.) )
                .column( Column::exact(w/12.) )
                .column( Column::exact(w/4.) )
                .body(|body| {
                    body.rows(60.0,  name_texts.len(), |row_index, mut row| {
                        let text_size = text_sizes[row_index];
                        row.col(|ui| {
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.label( RichText::new(name_texts[row_index]).heading().color(text_color).size(text_size) );
                            });
                        });
                    if let Some( txt ) = data_texts.get(row_index) {
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label( RichText::new(txt).heading().color(data_color).size(text_size) );
                             });
                        });
                        row.col(|ui| {
                            if let Some( trend ) = &data_trends[row_index] {
                                self.show_trend(ui, trend);
                            };
                        });
                        row.col(|ui| {
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.label( RichText::new(unit_texts[row_index]).heading().color(text_color).size(text_size) );
                            });
                        });
                    };
                });
            });
        });
    });
  }

  fn home_group_table(&self, ui: &mut Ui, title : &str, wd : &Option<AirQualityData> ) {
    let name_texts = vec![self.texts.temperature(), self.texts.humidity(), self.texts.co2(), self.texts.noise()];
    let unit_texts = vec!["°C", "%", "ppm", "dB"];

    let mut data_texts = Vec::<String>::new();
    if let Some( wd ) = wd {
             data_texts.push( format!("{:.1}", wd.room_temperature) );
             data_texts.push( format!("{}", wd.room_humidity) );
             data_texts.push( format!("{}", wd.room_co2) );
             data_texts.push( format!("{}", wd.room_noise) );
    }

    let text_color = Color32::from_rgb(242, 174, 73);
    let data_color = Color32::GREEN;
    let title_color = Color32::from_rgb(105, 209, 203);

    ui.push_id(title, |ui| {
        ui.vertical_centered(|ui| {
            ui.group(|ui| {
                    ui.label( RichText::new(title).heading().color(title_color).size(20.0) );
            });
            let w = ui.available_width();
            TableBuilder::new(ui)
                .column( Column::exact(w/2.) )
                .column( Column::exact(w/4.) )
                .column( Column::exact(w/4.) )
                .body(|body| {
                    body.rows(60.0,  name_texts.len(), |row_index, mut row| {
                        row.col(|ui| {
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.label( RichText::new(name_texts[row_index]).heading().color(text_color).size(40.0) );
                            });
                        });
                        if let Some( txt ) = data_texts.get(row_index) {
                            row.col(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label( RichText::new(txt).heading().color(data_color).size(40.0) );
                                 });
                            });
                            row.col(|ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    ui.label( RichText::new(unit_texts[row_index]).heading().color(text_color).size(40.0) );
                                });
                            });
                       };
                   });
             });
        });
    });
  }

  fn display_group_table(&self, ui: &mut Ui, dd : &Option<DisplayState> ) {
    let name_texts = vec![self.texts.brightness(), self.texts.preset()];

    let mut data_texts = Vec::<String>::new();
    if let Some( dd ) = dd {
             data_texts.push( if let Some( br ) = dd.brightness { format!("{}", br) } else { String::new() } );
             data_texts.push( if let Some( pr ) = &dd.preset { format!("{}", self.texts.show_preset(pr)) } else { String::new() } );
    }

    let title = "Дисплей";
    let text_color = Color32::from_rgb(242, 174, 73);
    let data_color = Color32::GREEN;
    let title_color = Color32::from_rgb(105, 209, 203);

    ui.push_id(title, |ui| {
        ui.vertical_centered(|ui| {
            ui.group(|ui| {
                    ui.label( RichText::new(title).heading().color(title_color).size(20.0) );
            });
            let w = ui.available_width();
            TableBuilder::new(ui)
                .column( Column::exact(w/2.) )
                .column( Column::exact(w/2.) )
                .body(|body| {
                    body.rows(60.0,  name_texts.len(), |row_index, mut row| {
                        row.col(|ui| {
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.label( RichText::new(name_texts[row_index]).heading().color(text_color).size(40.0) );
                            });
                        });
                        if let Some( txt ) = data_texts.get(row_index) {
                            row.col(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label( RichText::new(txt).heading().color(data_color).size(40.0) );
                                 });
                            });
                       };
                   });
                });
        });
    });
  }

  fn show_trend(&self, ui : &mut Ui, trend : &Trend)
  {
    let scale = 0.5;

    let image = match trend {
       Trend::Stable => &self.images.stable,
       Trend::Down => &self.images.arrow_down,
       Trend::Up => &self.images.arrow_up,
    };

    if let Some( image ) = image {
        image.show_scaled(ui, scale);
    };
  }

}

impl eframe::App for HomeDashboard {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

    log::debug!("screen_rect {:?} available_rect {:?} ", ctx.screen_rect(), ctx.available_rect());

    //ctx.set_debug_on_hover(true);

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
       .max_col_width(frame_width / 6.0)
       .num_columns(6)
       .show(ui, |ui| {
         ui.end_row();

         ui.add_visible(false, Separator::default());
         let home_data =
             if let Some( wd ) = &self.state.weather_data {
                 Some ( AirQualityData {
                     room_temperature : wd.room_temperature,
                     room_humidity : wd.room_humidity,
                     room_co2 : wd.room_co2,
                     room_noise : wd.room_noise,
                 })
             } else {
                None
             };


         self.outdoor_group_table(ui, &self.state.weather_data);
         self.bt_group(ui);
         self.display_group_table(ui, &self.state.display_state);
         ui.end_row();

         ui.add_visible(false, Separator::default());
         self.home_group_table(ui,  "Дом", &home_data);
         self.home_group_table(ui,  "Переговорка", &self.state.office_room_data);
         self.home_group_table(ui,  "Детская", &self.state.child_room_data);
         ui.end_row();
      });

      if ui.ctx().input( |i| i.key_pressed(Key::Q) )   {
        frame.close();
      }
    });
  }

}

fn indicator(ui: &mut egui::Ui, is_on: bool) {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 2.0);
    let (rect, mut _response) = ui.allocate_exact_size(desired_size, egui::Sense::click());


    if ui.is_rect_visible(rect) {
        let ext_c = Color32::GRAY;
        let color = if is_on { Color32::GREEN } else { Color32::GRAY };

        let radius = 0.5 * rect.width();
        let center = rect.center();
        let stroke = egui::Stroke::new(1.0, ext_c);

        ui.painter().circle_stroke(center, radius, stroke);
        ui.painter().circle_filled(center, 0.9*radius, color);
    }

}

fn switch_button(ui: &mut egui::Ui, on: &mut bool, label : &str) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 5.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, label));

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
