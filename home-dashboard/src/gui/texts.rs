use crate::worker::ddc_display::Preset;

#[derive(PartialEq)]
pub enum Language {
 Russian,
 English
}

pub struct Texts {
  language : Language,
}

impl Texts {
 pub fn new(language : Language) -> Texts {
   Texts {language}
 }

 pub fn temperature<'a>(&self) -> &'a str {
     self.select("Температура", "Temperature")
 }

 pub fn humidity<'a>(&self) -> &'a str {
     self.select("Влажность", "Humidity")
 }

 pub fn co2<'a>(&self) -> &'a str {
     self.select("Уровень СО2", "CO2 level")
 }

 pub fn noise<'a>(&self) -> &'a str {
     self.select("Уровень шума", "Noise level")
 }

 pub fn pressure<'a>(&self) -> &'a str {
     self.select("Давление", "Pressure")
 }

 pub fn brightness<'a>(&self) -> &'a str {
     self.select("Яркость", "Brightness")
 }

 pub fn preset<'a>(&self) -> &'a str {
     self.select("Режим", "Preset")
 }

 pub fn show_preset(&self, p : &Preset) -> String {
     match p {
         Preset::Standard => String::from(self.select("Стандартный", "Standard")),
         Preset::Comfort => String::from(self.select("Комфортный", "Comfort")),
         Preset::Game => String::from(self.select("Игровой", "Game")),
         Preset::Movie => String::from(self.select("Просмотра фильма", "Movie")),
         Preset::Unknown{val_dc, val_f0} => format!("{} {:#x} {:#x}", self.select("Неизвесный", "Unknown"),  val_dc, val_f0),
     }
 }

 fn select<'a>(&self, t1 : &'a str, t2: &'a str) -> &'a str
 {
     if self.language == Language::Russian {
         t1
     } else {
         t2
     }
 }

}
