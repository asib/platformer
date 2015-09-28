#[doc="false"]

extern crate sdl2;
extern crate sdl2_image;
extern crate find_folder;
#[macro_use(hashmap)]
extern crate platformer;

use std::rc::Rc;
use platformer::*;
use sdl2_image::{LoadTexture, INIT_PNG};
use sdl2::rect::Rect;

fn main() {
    const TITLE: &'static str = "Platformer";
    const FPS: u8 = 30;
    const WIDTH: u32 = 980;
    const HEIGHT: u32 = 700;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    sdl2_image::init(INIT_PNG);
    let window = video_subsystem.window(TITLE, WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let asset_path = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    let r = window.renderer().software().build().unwrap();

    let map = match tiled::Map::read_json(asset_path.join("map2.json")) {
        Ok(m) => m,
        Err(e) => match e {
            tiled::ReadError::IoError(e) => panic!("IOError: {:?}", e),
            tiled::ReadError::StringError(e) => panic!("StringError: {:?}", e),
            tiled::ReadError::JsonError(e) => panic!("JSONError: {:?}", e),
        },
    };

    let ts = map::Tileset::new_from_tiled_tileset(&asset_path.join("Platformer Pack/tiles_spritesheet.png"),
        &map.tilesets[0], &r);
    let mut new_map = map::Map::new_from_tiled_map(&map);
    if let &Some(ref data) = &map.layers[0].data {
        new_map.insert_data_using_tilset(data, &ts);
    }

    let mut sys = System::new(
        Game::new(
            true,
            // false,
            None,
            Camera::new(
                Point{x: 0, y: 0},
                WIDTH as i64,
                HEIGHT as i64,
                Rect::new_unwrap(100, 100, 780, 500)
            ),
            Player::new(
                Point{x: 250, y: 150},
                Rect::new(10, 00, 32, 60).unwrap().unwrap(),
                Rc::new(r.load_texture(&asset_path.join("sprite_map.png"))
                             .unwrap()),
                Rect::new(0, 0, 55, 65).unwrap(),
                Direction::Right,
                hashmap!(Direction::Up    => 1,
                         Direction::DoubleUp => 1,
                         Direction::Down  => 7,
                         Direction::Left  => 4,
                         Direction::StillLeft => 4,
                         Direction::Right => 3,
                         Direction::StillRight => 3),
                hashmap!(Direction::Up    => FPS,
                         Direction::DoubleUp => FPS,
                         Direction::Down  => FPS,
                         Direction::Left  => FPS,
                         Direction::StillLeft  => FPS,
                         Direction::Right => FPS,
                         Direction::StillRight => FPS),
                hashmap!(Direction::Up    => 1,
                         Direction::DoubleUp => 1,
                         Direction::Down  => 1,
                         Direction::Left  => 8,
                         Direction::StillLeft  => 1,
                         Direction::Right => 8,
                         Direction::StillRight => 1),
                hashmap!(Direction::Up    => Point::origin(),
                         Direction::DoubleUp => Point::origin(),
                         Direction::Down  => Point::origin(),
                         Direction::Left  => Point::origin(),
                         Direction::StillLeft  => Point{x:55*3, y:0},
                         Direction::Right => Point::origin(),
                         Direction::StillRight => Point{x:55*3, y:0}),
                true
            )),
        r,
        FPS,
        sdl_context.event_pump().unwrap(),
        &asset_path
    );

    sys.game.set_map(&mut new_map);

    // println!("{:?}", new_map.tiles.iter().map(|ref l| l.iter().map(|ref t| t.clip_rect).collect::<Vec<Option<Rect>>>()).collect::<Vec<Vec<Option<Rect>>>>());

    while sys.game.running {
        sys.update();
        sys.game.clear(&mut sys.r);
        //new_map.draw(&mut sys.r);
        sys.game.draw(&mut sys.r);
        if sys.game.debug {
            sys.game.draw_debug(&mut sys.r);
        }
        sys.game.flip_buffer(&mut sys.r);
    }

    sdl2_image::quit();
}
