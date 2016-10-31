extern crate gdk_pixbuf;
extern crate gdk_pixbuf_sys;
extern crate glib;
extern crate rustc_serialize;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path;

use rustc_serialize::json;

// error
#[derive(Debug)]
pub enum CacheError {
    Io(io::Error),
    JsonEncoder(json::EncoderError),
    JsonDecoder(json::DecoderError),
}

impl From<io::Error> for CacheError {
    fn from(err: io::Error) -> CacheError {
        CacheError::Io(err)
    }
}

impl From<json::EncoderError> for CacheError {
    fn from(err: json::EncoderError) -> CacheError {
        CacheError::JsonEncoder(err)
    }
}

impl From<json::DecoderError> for CacheError {
    fn from(err: json::DecoderError) -> CacheError {
        CacheError::JsonDecoder(err)
    }    
}

// pub fn write(filename: path::PathBuf, timeline: &Vec<::timeline::home::TimelineRow>) -> Result<(), CacheError> {
//     let mut file = try!(File::create(filename.clone()));
//     let json_str = try!(json::encode(timeline));
//     try!(file.write_all(json_str.as_bytes()));
//     info!("wrote cache to {:?}", filename);
//     Ok(())
// }

// pub fn load(filename: path::PathBuf) -> Result<Vec<::timeline::home::TimelineRow>, CacheError> {
//     let mut file = match File::open(filename) {
//         Ok(file) => file,
//         Err(_) => {
//             let home: Vec<::timeline::home::TimelineRow> = Vec::new();
//             return Ok(home);
//         },
//     };

//     let mut body = String::new();
//     try!(file.read_to_string(&mut body));
//     let home = try!(json::decode(body.as_str()).map_err(CacheError::JsonDecoder));
//     Ok(home)
// }
