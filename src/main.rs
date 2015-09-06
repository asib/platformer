#[doc="false"]

extern crate sdl2;
extern crate sdl2_image;
extern crate find_folder;
extern crate platformer;

use std::rc::Rc;
use platformer::*;
use sdl2_image::{LoadTexture, INIT_PNG};
use sdl2::rect::Rect;

fn main() {
    const TITLE: &'static str = "Platformer";
    const FPS: u8 = 30;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    sdl2_image::init(INIT_PNG);
    let window = video_subsystem.window(TITLE, 640, 480)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let asset_path = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    let mut r = window.renderer().software().build().unwrap();
    let mut sys = System::new(
        Game::new(Player::new(
            Point{x:50, y: 50},
            Rect::new(0, 0, 32, 62).unwrap().unwrap(),
            Rc::new(r.load_texture(&asset_path.join("claudius.png"))
                         .unwrap()),
            Rect::new(0, 0, 32, 62).unwrap(),
            Direction::Right,
            FPS,
            FPS,
            FPS,
            FPS,
            1,
            1,
            5,
            5
        )),
        r,
        FPS,
        sdl_context.event_pump().unwrap(),
        &asset_path
    );

    while sys.game.running {
        sys.update();
        sys.game.draw(&mut sys.r);
    }

    sdl2_image::quit();
}
