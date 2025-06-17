use std::fs::File;

use thiserror::Error;

use crate::GenResult;

#[derive(Debug, Clone, Default)]
pub struct FileProperties {
    pub bottom_image_type: BottomImageType,
    pub top_image_hash: u64,
    pub x_val: f64,
    pub y_val: f64,
    pub zoom_val: f64,
    pub monochrome_toggle: bool,
    pub monochrome_invert: bool,
    pub monochrome_default: bool,
    pub monochrome_color: Option<(u8, u8, u8)>,
    pub monochrome_threshold_val: f64,
}

impl FileProperties {
    pub fn from_filename(filename: String) -> GenResult<Self> {
        // Load all properties from the filename
        let file_properties = filename.split("-");
        let properties_list: Vec<&str> = file_properties.into_iter().collect();

        // If filename is not folder_new it is not compatible
        match properties_list[FilenameProperty::FileName as usize] {
            "folder_new" => (),
            _ => return Err(Box::new(PropertiesError::Incompatible)),
        }
        let x_val: f64 = properties_list[FilenameProperty::XScale as usize].parse()?;
        let y_val: f64 = properties_list[FilenameProperty::YScale as usize].parse()?;
        let zoom_val: f64 = properties_list[FilenameProperty::ZoomVal as usize].parse()?;
        let monochrome_toggle: bool =
            properties_list[FilenameProperty::MonochromeSelected as usize].parse()?;
        let monochrome_invert: bool =
            properties_list[FilenameProperty::MonochromeInverted as usize].parse()?;
        let monochrome_default: bool =
            properties_list[FilenameProperty::DefaultMonochromeColor as usize].parse()?;
        let monochrome_threshold_val: f64 =
            properties_list[FilenameProperty::MonochromeThreshold as usize].parse()?;
        let monochrome_color = if !monochrome_default {
            let mut rgb: (u8, u8, u8) = (0, 0, 0);
            rgb.0 = properties_list[FilenameProperty::MonochromeRed as usize].parse()?;
            rgb.1 = properties_list[FilenameProperty::MonochromeGreen as usize].parse()?;
            rgb.2 = properties_list[FilenameProperty::MonochromeBlue as usize].parse()?;
            Some(rgb)
        } else {
            None
        };
        let top_image_hash: u64 = properties_list[FilenameProperty::Hash as usize].parse()?;
        let bottom_image_type =
            match properties_list[FilenameProperty::DefaultBottomImage as usize].parse()? {
                true => BottomImageType::FolderSystem,
                false => BottomImageType::Unknown,
            };

        Ok(Self {
            x_val,
            y_val,
            zoom_val,
            monochrome_color,
            monochrome_default,
            monochrome_invert,
            monochrome_threshold_val,
            monochrome_toggle,
            top_image_hash,
            bottom_image_type,
        })
    }
}

#[derive(Debug, Error)]
pub enum PropertiesError {
    #[error("The provided filename is not compatible")]
    Incompatible,
}

#[derive(Debug, Clone, Default)]
pub enum BottomImageType {
    #[default]
    Unknown,
    FolderSystem,
    Folder(String),
    FolderCustom(String, String),
    Custom,
    Temporary,
}

impl BottomImageType {
    // Whether this image is able to be regenerate with strict mode enabled
    // By returning none, the image is not at all compatible for regeneration
    fn is_strict_type(&self) -> Option<bool> {
        match self {
            Self::FolderSystem => Some(true),
            Self::FolderCustom(_, _) => Some(false),
            Self::Folder(value) if value != "Unknown" => Some(false),
            _ => None,
        }
    }
}

// The properties of the to be regenerated file, are stored in the filename
// To get the properties, split the filename at the '-'.
// This enum provides a more visual method of knowing at what index each property is..
pub enum FilenameProperty {
    FileName = 0,
    DefaultBottomImage,
    XScale,
    YScale,
    ZoomVal,
    MonochromeSelected,
    MonochromeThreshold,
    MonochromeRed,
    MonochromeGreen,
    MonochromeBlue,
    MonochromeInverted,
    DefaultMonochromeColor,
    Hash,
}
