extern crate curl;
extern crate crypto;

use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;

use self::curl::http;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

// UtilsError
#[derive(Debug)]
pub enum UtilsError {
    Io(io::Error),
    String(String),
}

impl From<io::Error> for UtilsError {
    fn from(err: io::Error) -> UtilsError {
        UtilsError::Io(err)
    }
}

impl From<String> for UtilsError {
    fn from(err: String) -> UtilsError {
        UtilsError::String(err)
    }
}

pub fn get_profile_image(profile_image_url: &String) -> Result<String, UtilsError> {
    let home_dir = match env::home_dir() {
        Some(home_dir) => home_dir,
        None => {
            error!("home directory is not set");
            panic!("home directory is not set")
        },
    };

    let cache_dir = home_dir.clone().join(::vars::CACHE_DIR).join("rustytwit").join("images");

    let mut sha256 = Sha256::new();
    sha256.input_str(&profile_image_url);

    match File::open(cache_dir.join(sha256.result_str())) {
        Ok(_) => (),
        Err(_) => {
            match http::handle().get(profile_image_url.as_str()).exec() {
                Ok(resp) => {
                    match resp.get_code() {
                        200 => {
                            try!(File::create(cache_dir.join(sha256.result_str()))?.write_all(resp.get_body()));
                        },
                        code => {
                            return Err(UtilsError::String(format!("response status is {}", code)));
                        },
                    }
                },
                Err(err) => {
                    return Err(UtilsError::String(format!("{:?}", err)));
                },
            };
        },
    }

    let cache_path = cache_dir.join(sha256.result_str());
    return Ok(cache_path.to_str().unwrap().to_string());
}
