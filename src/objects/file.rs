use std::path::PathBuf;
use gtk::gio;
use adw::prelude::FileExt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct File {
	pub files: gio::File,
	pub path: PathBuf,
	pub name: String,
	pub extension: String,
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
		Self {
			files: file,
			path: temp_path.into(),
			extension: file_extension,
			name: name_no_extension,
		}
	}
	pub fn get_file(&self) -> &gio::File{
	    &self.files
	}
}

