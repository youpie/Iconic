use adw::prelude::FileExt;
use gio::{Cancellable, FileQueryInfoFlags};
use gtk::gio;
use image::*;
use log::*;
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{Options, Transform, Tree};
use std::ffi::OsStr;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use crate::GenResult;
use crate::objects::errors::IntoResult;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub files: Option<gio::File>,
    pub path: PathBuf,
    pub filename: String,
    pub extension: String,
    pub dynamic_image: DynamicImage,
    pub dynamic_image_resized: bool,
    pub thumbnail: DynamicImage,
    pub hash: u64,
}

impl File {
    pub fn path_str(&self) -> String {
        self.path.clone().into_os_string().into_string().unwrap()
    }

    pub fn new(file: gio::File, size: u32, thumbnail_size: u32) -> GenResult<Self> {
        let temp_path = file.path().into_reason_result("Can't get file path")?;
        let file_info =
            file.query_info("standard::", FileQueryInfoFlags::NONE, Cancellable::NONE)?;
        let file_name_pathbuf = PathBuf::from(file_info.name().into_os_string());
        let file_name = file_name_pathbuf
            .file_stem()
            .unwrap_or(OsStr::new("unkwnown"))
            .to_str()
            .into_result()?
            .to_string();
        let file_extension = file_name_pathbuf
            .extension()
            .unwrap_or(OsStr::new("png"))
            .to_str()
            .into_result()?
            .to_string();
        let mime_type = file_info.content_type();
        debug!("Mime type: {:?}", mime_type);
        let mut dynamic_image = if mime_type == Some("image/svg+xml".into()) {
            let path = temp_path.as_os_str().to_str().into_result()?;
            Self::load_svg(path, size)?
        } else {
            match image::open(temp_path.clone().into_os_string()) {
                Err(_) => {
                    let image = ImageReader::open(temp_path.clone().into_os_string())?
                        .with_guessed_format()?;
                    image.decode()?
                }
                Ok(x) => x,
            }
        };
        let hash = Self::create_hash(&dynamic_image);
        debug!("hash of created file: {}", hash);
        let mut dynamic_image_resized = false;
        // Resize the image if the file is larger than the set size
        // Reduces cache file size, but will take even longer to load image
        if dynamic_image.width() > size as u32 || dynamic_image.height() > size as u32 {
            dynamic_image_resized = true;
            dynamic_image =
                dynamic_image.resize(size as u32, size as u32, imageops::FilterType::Nearest);
        }

        let mut thumbnail = DynamicImage::new_rgb8(0, 0);
        if thumbnail_size > 0 {
            thumbnail = if file_extension == "svg" {
                let path = &temp_path.as_os_str().to_str().unwrap();
                Self::load_svg(path, thumbnail_size)?
            } else {
                dynamic_image.clone().resize(
                    thumbnail_size as u32,
                    thumbnail_size as u32,
                    imageops::FilterType::Nearest,
                )
            };
        }
        Ok(Self {
            files: Some(file),
            path: temp_path.into(),
            extension: mime_type.into_result()?.into(),
            filename: file_name,
            dynamic_image,
            dynamic_image_resized,
            thumbnail,
            hash,
        })
    }

    pub fn from_path_string(path: &str, size: u32, thumbnail_size: u32) -> GenResult<Self> {
        let file = gio::File::for_path(PathBuf::from(path).as_path());
        Self::new(file, size, thumbnail_size)
    }

    pub fn from_path(path: PathBuf, size: u32, thumbnail_size: u32) -> GenResult<Self> {
        let file = gio::File::for_path(path);
        Self::new(file, size, thumbnail_size)
    }

    pub fn from_image(
        image: DynamicImage,
        thumbnail_size: u32,
        max_size: Option<u32>,
        filename: &str,
    ) -> Self {
        let mut image = image;
        let thumbnail = image.clone().resize(
            thumbnail_size,
            thumbnail_size,
            imageops::FilterType::Nearest,
        );
        let hash = Self::create_hash(&image);
        let mut dynamic_image_resized = false;
        // Resize the image if the file is larger than the set size
        // Reduces cache file size, but will take even longer to load image
        if let Some(size) = max_size {
            if image.width() > size || image.height() > size {
                dynamic_image_resized = true;
                image = image.resize(size, size, imageops::FilterType::Nearest);
            }
        }
        Self {
            files: None,
            path: "".into(),
            extension: ".dynamic".to_string(),
            filename: filename.to_string(),
            hash,
            dynamic_image: image,
            thumbnail,
            dynamic_image_resized,
        }
    }

    pub fn load_svg(path: &str, size: u32) -> GenResult<DynamicImage> {
        // Load the SVG file content
        let svg_data = match fs::read(path) {
            Ok(x) => x,
            Err(_) => fs::read("/usr/share/icons/Adwaita/scalable/places/folder.svg")?,
        };
        // Create an SVG tree
        let opt = Options::default();
        let rtree = Tree::from_data(&svg_data, &opt)?;

        // Specify the output dimensions (you can adjust these as needed)
        let width = rtree.size().width();
        let height = rtree.size().height();

        // Calculate the scale factor
        let scale_x = size as f32 / width;
        let scale_y = size as f32 / height;
        let scale = scale_x.min(scale_y); // Maintain aspect ratio

        // Create a Pixmap to render into
        let mut pixmap = Pixmap::new(size, size).unwrap();

        // Render the SVG tree to the Pixmap
        resvg::render(
            &rtree,
            Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );
        Ok(Self::pixmap_to_image(pixmap))
    }

    // Adds hash of file to file name
    fn create_hash(image: &DynamicImage) -> u64 {
        let mut hasher = DefaultHasher::new();
        let _ = image.as_bytes().hash(&mut hasher);
        hasher.finish()
    }

    fn pixmap_to_image(pixmap: Pixmap) -> DynamicImage {
        // Create an empty RgbaImage with the same dimensions as the Pixmap.
        let pixmap_clone = pixmap.clone();
        let mut img = RgbaImage::new(pixmap_clone.width(), pixmap.height());

        // Iterate over each pixel in the Pixmap.
        for y in 0..pixmap_clone.height() {
            for x in 0..pixmap_clone.width() {
                // Get the pixel at (x, y). `pixel` returns an Option, we unwrap it safely because we know (x, y) is valid.
                if let Some(pixel) = pixmap_clone.pixel(x, y) {
                    // Copy the pixel's RGBA data into the RgbaImage.
                    img.put_pixel(
                        x,
                        y,
                        Rgba([pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]),
                    );
                }
            }
        }

        // Convert the RgbaImage to a DynamicImage.
        DynamicImage::ImageRgba8(img)
    }
}
