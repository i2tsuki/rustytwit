extern crate curl;
extern crate crypto;

use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;

use self::curl::http;
use self::crypto::sha2::Sha256;
use self::crypto::digest::Digest;

// error
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
        None => { error!("home directory is not set");
                  panic!("home directory is not set") },
    };
    
    let cache_dir = home_dir.clone().join(::vars::CACHE_DIR).join("rustytwit").join("images");
    
    let mut sha256 = Sha256::new();
    sha256.input_str(&profile_image_url);
    
    match File::open(format!("/home/kinoo/.cache/rustytwit/image/{}", sha256.result_str())) {
        Ok(_) => (),
        Err(_) => {
            let resp = match http::handle().get(profile_image_url.as_str()).exec() {
                Ok(resp) => resp,
                Err(err) => {
                    let e = UtilsError::String(format!("{:?}", err));
                    return Err(e);
                },
            };
            if resp.get_code() != 200 {
                let e = UtilsError::String(format!("response status is {}", resp.get_code()));
                return Err(e);
            }
            let result = File::create(format!("/home/kinoo/.cache/rustytwit/image/{}", sha256.result_str()));
            if result.is_err() {
                ::std::process::exit(1);
            }            
            let result = result.unwrap().write_all(resp.get_body());
            if result.is_err() {
                ::std::process::exit(1);
            }
        }
    }
    
    let cache_path = format!("/tmp/rustytwit/{}", sha256.result_str());
    return Ok(cache_path)
}
