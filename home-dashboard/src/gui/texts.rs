
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

 fn select<'a>(&self, t1 : &'a str, t2: &'a str) -> &'a str
 {
     if self.language == Language::Russian {
         t1
     } else {
         t2
     }
 }

}
