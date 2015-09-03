extern crate sdl2;
extern crate sdl2_image;
extern crate find_folder;

use sdl2_image::{LoadTexture, INIT_PNG};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {
    const TITLE: &'static str = "Platformer";

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    sdl2_image::init(INIT_PNG);
    let window = video_subsystem.window(TITLE, 640, 480)
                                .position_centered()
                                .opengl()
                                .build()
                                .unwrap();
    let mut renderer = window.renderer().software().build().unwrap();
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let sprite = assets.join("sprite.png");
    let sprite = renderer.load_texture(&sprite).unwrap();

    renderer.clear();
    renderer.copy(&sprite, None, None);
    renderer.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut running = true;

    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
                    running = false
                },
                _ => ()
            }
        }
    }

    sdl2_image::quit();
}
