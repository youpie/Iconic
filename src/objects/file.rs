use std::path::PathBuf;
use gtk::gio;
use image::*;
use adw::prelude::FileExt;
use std::fs;
use resvg::tiny_skia::{Pixmap};
use resvg::usvg::{Tree, Options};

#[derive(Debug, Clone, PartialEq)]
pub struct File {
	pub files: Option<gio::File>,
	pub path: PathBuf,
	pub name: String,
	pub extension: String,
	pub dynamic_image: DynamicImage,
	pub thumbnail: DynamicImage
}

impl File{
	pub fn path_str (&self) -> String{
		self.path.clone().into_os_string().into_string().unwrap()
	}
	pub fn new(file: gio::File) -> Self{
		let temp_path = file.path().unwrap();
		let file_name = file.basename().unwrap().into_os_string().into_string().unwrap();
		let period_split:Vec<&str> = file_name.split(".").collect();
		let file_extension = format!(".{}",period_split.last().unwrap());
		let dynamic_image = if file_extension == ".svg" {
		    let path = &temp_path.as_os_str().to_str().unwrap();
            Self::load_svg(path)
		} else{
		    image::open(temp_path.clone().into_os_string()).unwrap()
		};
        let name_no_extension = file_name.replace(&file_extension,"");

        let thumbnail = dynamic_image.clone().resize(255, 255, imageops::FilterType::Nearest);
		Self {
			files: Some(file),
			path: temp_path.into(),
			extension: file_extension,
			name: name_no_extension,
			dynamic_image,
			thumbnail
		}
	}

	pub fn from_path(path: &str) -> Self{
        //let thumbnail = file.clone().resize(255, 255, imageops::FilterType::Nearest);
        let file = gio::File::for_path(PathBuf::from(path).as_path());
        Self::new(file)
	}

	pub fn load_svg(path: &str) -> DynamicImage{
        // Load the SVG file content
        let svg_data = fs::read(path).expect("Failed to read SVG file");

        // Create an SVG tree
        let opt = Options::default();
        let rtree = Tree::from_data(&svg_data, &opt).expect("Failed to parse SVG data");

        // Specify the output dimensions (you can adjust these as needed)
        let width = rtree.size().width();
        let height = rtree.size().height();

        // Desired dimensions
        let desired_width = 1024.0;
        let desired_height = 1024.0;

        // Calculate the scale factor
        let scale_x = desired_width / width;
        let scale_y = desired_height / height;
        let scale = scale_x.min(scale_y); // Maintain aspect ratio

        // Create a Pixmap to render into
        let mut pixmap = Pixmap::new(desired_width as u32, desired_height as u32).expect("Failed to create Pixmap");

        // Render the SVG tree to the Pixmap
        let _ = resvg::render(
            &rtree,
            usvg::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut()
        );
        Self::pixmap_to_image(&pixmap)
    }

    fn pixmap_to_image(pixmap: &Pixmap) -> DynamicImage {
        // Create an empty RgbaImage with the same dimensions as the Pixmap.
        let mut img = RgbaImage::new(pixmap.width(), pixmap.height());

        // Iterate over each pixel in the Pixmap.
        for y in 0..pixmap.height() {
            for x in 0..pixmap.width() {
                // Get the pixel at (x, y). `pixel` returns an Option, we unwrap it safely because we know (x, y) is valid.
                if let Some(pixel) = pixmap.pixel(x, y) {
                    // Copy the pixel's RGBA data into the RgbaImage.
                    img.put_pixel(x, y, Rgba([pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]));
                }
            }
        }

        // Convert the RgbaImage to a DynamicImage.
        DynamicImage::ImageRgba8(img)
    }
}

