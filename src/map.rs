use std::rc::Rc;
use std::path::Path;
use sdl2;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};
use sdl2_image::LoadTexture;
use tiled;
use super::{CameraDrawable, Camera};

pub struct Tileset {
    pub firstgid: u32,
    pub texture: Rc<Texture>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_count: u32,
    pub margin: u32,
    pub spacing: u32,
}

impl Tileset {
    pub fn new_from_tiled_tileset(img_path: &Path, ts: &tiled::Tileset, r: &Renderer) -> Self {
        let tx = Rc::new(r.load_texture(img_path).ok().expect("couldn't load tileset image"));
        let sdl2::render::TextureQuery{width: w, height: h, ..} = tx.query();
        Tileset {
            firstgid: ts.firstgid,
            texture: tx,
            texture_width: w,
            texture_height: h,
            tile_width: ts.tilewidth,
            tile_height: ts.tileheight,
            tile_count: ts.tilecount,
            margin: ts.margin,
            spacing: ts.spacing,
        }
    }

    pub fn side_len(&self) -> u32 {
        return (self.tile_count as f64).sqrt() as u32;
    }

    fn row_for_id(&self, id: u32) -> u32 {
        return id / self.side_len();
    }

    fn col_for_id(&self, id: u32) -> u32 {
        return id % self.side_len();
    }

    pub fn tile_for_id(&self, mut id: u32) -> Option<Rect> {
        if id == 0 {
            return None;
        }
        id -= 1;

        let (mut x, mut y) = (self.margin, self.margin);
        let (row, col) = (self.row_for_id(id), self.col_for_id(id));

        x += col * (self.tile_width + self.spacing);
        y += row * (self.tile_height + self.spacing);

        Some(Rect::new_unwrap(x as i32,  y as i32, self.tile_width, self.tile_height))
    }
}

pub struct Tile {
    pub texture: Rc<Texture>,
    pub clip_rect: Option<Rect>,
}

impl Tile {
    pub fn new(tx: Rc<Texture>, cr: Option<Rect>) -> Self {
        Tile {
            texture: tx,
            clip_rect: cr,
        }
    }
}

impl Clone for Tile {
    fn clone(&self) -> Self {
        Tile {
            texture: self.texture.clone(),
            clip_rect: self.clip_rect.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new_from_tiled_map(tmap: &tiled::Map) -> Self {
        Map {
            width: tmap.width,
            height: tmap.height,
            tile_width: tmap.tilewidth,
            tile_height: tmap.tileheight,
            tiles: Vec::new(),
        }
    }

    pub fn pixel_width(&self) -> u32 {
        return self.width*self.tile_width;
    }

    pub fn pixel_height(&self) -> u32 {
        return self.height*self.tile_height;
    }

    pub fn insert_data_using_tilset(&mut self, data: &[u8], ts: &Tileset) {
        for i in 0..self.height {
            let _i = i as usize;
            self.tiles.push(Vec::with_capacity(self.width as usize));

            for j in 0..self.width {
                let _j = j as usize;
                self.tiles[_i].push(Tile::new(ts.texture.clone(),
                    ts.tile_for_id(data[_i*self.width as usize + _j] as u32)));
            }
        }
    }
}

impl CameraDrawable for Map {
    fn draw(&mut self, r: &mut Renderer, c: &Camera) {
        for (i, row) in self.tiles.iter().enumerate() {
            let i = i as i32;
            for (j, tile) in row.iter().enumerate() {
                if tile.clip_rect == None {
                    continue;
                }

                let j = j as i32;
                let (x, y) = (j*self.tile_width as i32, i*self.tile_height as i32);
                if (x+self.tile_width as i32) < c.pos.x as i32 || x > (c.pos.x+c.width) as i32 { continue }
                if (y+self.tile_height as i32) < c.pos.y as i32 || y > (c.pos.y+c.height) as i32 { continue }
                r.copy(&*tile.texture, tile.clip_rect,
                    Some(Rect::new_unwrap(x - c.pos.x as i32, y - c.pos.y as i32,
                        self.tile_width, self.tile_height)));
            }
        }
        r.present();
    }
}
