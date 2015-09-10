use std;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use rustc_serialize::json;
use flate2::read::ZlibDecoder;

#[derive(RustcDecodable, RustcEncodable)]
pub struct TiledTileset {
    pub image: String,
    pub image_width: u32,
    pub image_height: u32,
    pub tile_height: u32,
    pub tile_width: u32,
    pub tile_count: u32,
    pub margin: u32,
    pub spacing: u32,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct TiledLayer {
    pub data: String,
    pub width: u32,
    pub height: u32,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct TiledMap {
    pub layers: Vec<TiledLayer>,
    pub width: u32,
    pub height: u32,
    pub tilesets: Vec<TiledTileset>,
    pub tile_width: u32,
    pub tile_height: u32,
}

pub fn read_tiled_json<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let mut f = try!(File::open(path));
    let mut reader = BufReader::new(f);

    Ok(())
}
