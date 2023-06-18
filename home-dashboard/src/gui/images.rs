use log;
use egui_extras::image::RetainedImage;
use std::fs;
use std::path::Path;

pub struct Images {
  pub arrow_up : Option<RetainedImage>,
  pub arrow_down : Option<RetainedImage>,
  pub stable : Option<RetainedImage>,
}

impl Images {
  pub fn new(path : &Path) -> Images {
    let up_image =  read_svg_image_with_log(&path.join("up_arrow.svg"));
    let down_image =  read_svg_image_with_log(&path.join("down_arrow.svg"));
    let stable_image =  read_svg_image_with_log(&path.join("stable.svg"));

    Images {
      arrow_up : up_image,
      arrow_down: down_image,
      stable: stable_image,
    }

  }
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
