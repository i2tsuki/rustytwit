extern crate toml;

use std::cell::Cell;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path;
use std::str::FromStr;


const DEFAULT: &'static str = r#"
[general]
update_timer = false

[access_key]
key = ""
secret = ""

[home_timeline]
last_update_id = 1
last_read_id = 1
limits = 500
"#;

// ConfigError
#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    TomlParserError(Vec<toml::ParserError>),
    String(String),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::Io(err)
    }
}

impl From<Vec<toml::ParserError>> for ConfigError {
    fn from(err: Vec<toml::ParserError>) -> ConfigError {
        ConfigError::TomlParserError(err)
    }
}

impl From<String> for ConfigError {
    fn from(err: String) -> ConfigError {
        ConfigError::String(err)
    }
}

// Struct definition
#[derive(Clone, Debug)]
pub struct Config {
    pub filename: String,
    pub toml: Toml,
}

// Toml struct implementation
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Toml {
    pub general: General,
    pub access_key: AccessKey,
    pub home_timeline: HomeTimeline,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct General {
    pub update_timer: bool,
    pub update_timer_duration: i32,
    pub url_filter: bool,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct AccessKey {
    pub key: String,
    pub secret: String,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct HomeTimeline {
    pub last_update_id: Cell<u64>,
    pub last_read_id: Cell<u64>,
    pub limits: Cell<usize>,
}
unsafe impl Sync for HomeTimeline {}


impl Config {
    pub fn new<P: AsRef<path::Path>>(filename: P) -> Result<Config, ConfigError> {
        let filename = filename.as_ref();
        let mut file = match File::open(filename) {
            Ok(file) => file,
            Err(_) => {
                let mut file = try!(File::create(filename));
                try!(file.write_all(DEFAULT.as_ref()));
                match File::open(filename) {
                    Ok(file) => file,
                    Err(err) => return Err(ConfigError::Io(err)),
                }
            },
        };

        let mut body = String::new();
        try!(file.read_to_string(&mut body));

        let value = try!(toml::Value::from_str(body.as_ref()));
        let toml = try!(toml::decode(value).ok_or(
            "failed to decode toml value".to_owned(),
        ));
        let filename = try!(filename.to_str().ok_or(
            "failed to generalize filename path to string".to_owned(),
        )).to_string();
        let config: Config = Config {
            filename: filename,
            toml: toml,
        };
        return Ok(config);
    }

    pub fn sync(&self) -> Result<(), ConfigError> {
        let mut file = try!(File::create(&self.filename));
        try!(file.write_all(toml::encode_str(&self.toml).as_ref()));
        try!(file.sync_all());
        info!("config is synchronized");
        Ok(())
    }
}
