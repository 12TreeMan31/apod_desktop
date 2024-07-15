use curl::easy::{Easy, Transfer};
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::fs::{self, DirEntry, File};
use std::io::{self, prelude::*};
use std::path::Path;
use std::str;

#[derive(Deserialize)]
struct Config {
    api_url: String,
    api_key: String,
    storage_dir: String,
}

impl Config {
    fn load(config_path: &Path) -> io::Result<Self> {
        let mut raw_buf: Vec<u8> = Vec::new();
        let mut f: File = File::open(config_path)?;

        f.read_to_end(&mut raw_buf)?;
        let str_buf: &str = str::from_utf8(&raw_buf).unwrap_or_else(|error| unsafe {
            str::from_utf8_unchecked(&raw_buf[..error.valid_up_to()])
        });

        let json: Config = serde_json::from_str(str_buf).expect("Could not parse json from config");
        if !Path::new(&json.storage_dir).is_dir() {
            panic!("storage directory is not a directory!");
        }

        return Ok(json);
    }
}

#[derive(Serialize, Deserialize)]
struct RetFields {
    copyright: String,
    date: String,
    explanation: String,
    hdurl: String,
    media_type: String,
    service_version: String,
    title: String,
    url: String,
}

fn get_data(buf: &mut Vec<u8>, ez: &mut Easy) -> Result<(), curl::Error> {
    let mut tr: Transfer = ez.transfer();
    tr.write_function(|new_data: &[u8]| {
        buf.extend_from_slice(new_data);
        Ok(new_data.len())
    })?;
    tr.perform()
}

fn is_dup(path: &Path, name: &OsStr) -> std::io::Result<bool> {
    for file in fs::read_dir(path)? {
        let file: DirEntry = file?;
        if file.file_name() == name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn main() -> Result<(), curl::Error> {
    let config_path: &Path = Path::new("/home/treeman/.config/apod_desktop/apod.conf");
    let config: Config = Config::load(config_path).unwrap();
    let api: String = format!("{}?api_key={}", config.api_url, config.api_key);
    // Init curl
    let mut ez: Easy = Easy::new();
    ez.get(true)?;

    let mut buf: Vec<u8> = Vec::new();
    // Gets json data
    ez.url(&api)?;
    get_data(&mut buf, &mut ez)?;

    let json: &str = str::from_utf8(&buf).unwrap();
    let json: RetFields = serde_json::from_str(json).unwrap();

    let image_name: OsString = OsString::from(format!("{}.png", json.date));
    if is_dup(Path::new(&config.storage_dir), &image_name).unwrap() {
        return Ok(());
    }

    // Gets image
    buf.clear();
    ez.url(&json.url)?;
    get_data(&mut buf, &mut ez)?;

    // Write image
    let image_path: String = format!(
        "{}/{}",
        config.storage_dir,
        image_name.clone().into_string().unwrap()
    );
    let mut image: File = File::create(image_path).unwrap();
    image.write_all(&buf).unwrap();

    let background_path: String = format!("{}/background.png", config.storage_dir);
    let image_path: String = format!(
        "{}/{}",
        config.storage_dir,
        image_name.clone().into_string().unwrap()
    );
    fs::copy(image_path.clone(), background_path).unwrap();

    Ok(())
}
