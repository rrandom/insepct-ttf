use nfd::Response;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct RawFontInfo {
    pub path: PathBuf,
    pub data: Vec<u8>,
}

pub async fn open_dialog() -> Result<PathBuf, io::Error> {
    let result: nfd::Response =
        match async { return nfd::open_file_dialog(Some("ttf"), None) }.await {
            Ok(result) => result,
            Err(e) => {
                dbg!(&e);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unable to unwrap data from new file dialog",
                ));
            }
        };

    let file_string: String = match result {
        Response::Okay(file_path) => file_path,
        Response::OkayMultiple(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Multiple files returned when one was expected",
            ))
        }
        Response::Cancel => {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "User cancelled file open",
            ))
        }
    };

    let mut result: PathBuf = PathBuf::new();
    result.push(Path::new(&file_string));

    if result.exists() {
        Ok(result)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "File does not exist",
        ))
    }
}

pub async fn open() -> Result<RawFontInfo, super::LoadError> {
    use super::LoadError;

    let path = match open_dialog().await {
        Ok(path) => path,
        Err(error) => {
            println!("{:?}", error);
            return Err(LoadError::FileError);
        }
    };

    let font_data = async_std::fs::read(path.as_path()).await.unwrap();

    Ok(RawFontInfo {
        path,
        data: font_data,
    })
}
