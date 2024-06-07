use std::path::PathBuf;
use gtk::gio;
use image::*;
use adw::prelude::FileExt;

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
        let name_no_extension = file_name.replace(&file_extension,"");
        let dynamic_image = image::open(temp_path.clone().into_os_string()).unwrap();
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

	pub fn from_dynamicimage(file: DynamicImage) -> Self{
        let thumbnail = file.clone().resize(255, 255, imageops::FilterType::Nearest);
		Self {
			files: None,
			path: PathBuf::new(),
			extension: String::from("svg"),
			name: String::from("file"),
			dynamic_image: file,
			thumbnail
		}
	}
}

