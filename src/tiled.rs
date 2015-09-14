use std;
use std::path::Path;
use std::error::Error;
use std::result::Result;
use std::fs::File;
use std::io::Read;
use std::string::FromUtf8Error;
use rustc_serialize::json;
use rustc_serialize::base64::FromBase64;
use rustc_serialize::base64::FromBase64Error;
use flate2::read::ZlibDecoder;

#[derive(Debug)]
pub enum ReadError {
    IoError(std::io::Error),
    StringError(FromUtf8Error),
    JsonError(json::DecoderError),
}

impl<'a> From<std::io::Error> for ReadError {
    fn from(e: std::io::Error) -> ReadError {
        ReadError::IoError(e)
    }
}

impl<'a> From<FromUtf8Error> for ReadError {
    fn from(e: FromUtf8Error) -> ReadError {
        ReadError::StringError(e)
    }
}

impl<'a> From<json::DecoderError> for ReadError {
    fn from(e: json::DecoderError) -> ReadError {
        ReadError::JsonError(e)
    }
}

#[derive(Debug)]
pub enum DataError {
    NoDataError,
    NoLayerError,
    Base64Error(FromBase64Error),
    IoError(std::io::Error),
}

impl<'a> From<FromBase64Error> for DataError {
    fn from(e: FromBase64Error) -> DataError {
        DataError::Base64Error(e)
    }
}

impl<'a> From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> DataError {
        DataError::IoError(e)
    }
}

#[derive(RustcDecodable, RustcEncodable, Clone, Debug)]
pub struct Tileset {
    pub image: String,
    pub imagewidth: u32,
    pub imageheight: u32,
    pub tileheight: u32,
    pub tilewidth: u32,
    pub tilecount: u32,
    pub margin: u32,
    pub spacing: u32,
}

#[derive(RustcDecodable, RustcEncodable, Clone, Debug)]
pub struct Layer {
    pub data: Option<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(RustcDecodable, RustcEncodable, Clone, Debug)]
pub struct Map {
    pub layers: Vec<Layer>,
    pub width: u32,
    pub height: u32,
    pub tilesets: Vec<Tileset>,
    pub tilewidth: u32,
    pub tileheight: u32,
}

impl Map {
    pub fn read_json<P: AsRef<Path>>(path: P) -> Result<Self, ReadError> {
        let mut f = try!(File::open(path));
        let mut contents = vec!();
        try!(f.read_to_end(&mut contents));
        let contents = try!(String::from_utf8(contents));

        let map = try!(json::decode(&contents));
        Ok(map)
    }

    pub fn data(&self, layer: usize) -> Result<Vec<u8>, DataError> {
        let data = if let Some(l) = self.layers.get(layer) {
            if let Some(d) = l.data.clone() { d }
            else { return Err(DataError::NoDataError) }
        } else { return Err(DataError::NoLayerError) };
        let data = try!((*data.as_bytes()).from_base64());

        let mut decoder = ZlibDecoder::new(&data as &[u8]);
        let mut decoded_data = vec!();
        try!(decoder.read_to_end(&mut decoded_data));
        Ok(decoded_data)
    }
}
