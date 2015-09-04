extern crate sdl2;

use std::rc::Rc;
use std::path::Path;
use std::sync::mpsc::Receiver;
use sdl2::EventPump;
use sdl2::render::{Renderer, Texture};
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};

fn timer_periodic(ms: u32) -> Receiver<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep_ms(ms);
            if tx.send(()).is_err() {
                break;
            }
        }
    });
    rx
}

pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    pub fn origin() -> Self {
        Point{x: 0, y: 0}
    }
}

pub struct Velocity {
    pub x: f64,
    pub y: f64,
}

impl Velocity {
    pub fn zero() -> Self {
        Velocity{x: 0.0, y: 0.0}
    }
}

pub struct Acceleration {
    pub x: f64,
    pub y: f64,
}

impl Acceleration {
    pub fn zero() -> Self {
        Acceleration{x: 0.0, y: 0.0}
    }
}

pub enum Direction {
    Down,
    Left,
    Up,
    Right,
}

impl Direction {
    fn to_int(&self) -> u8 {
        match *self {
            Direction::Down  => 0,
            Direction::Left  => 1,
            Direction::Up    => 2,
            Direction::Right => 3,
        }
    }
}

pub struct Entity {
    pub pos: Point,
    pub collision_rect: Rect,
    pub sprite_map: Rc<Texture>,
    pub draw_rect: Option<Rect>,
}

impl Entity {
    fn new(p: Point, cr: Rect, t: Rc<Texture>, dr: Option<Rect>) -> Self {
        Entity {
            pos: p,
            collision_rect: cr,
            sprite_map: t,
            draw_rect: dr,
        }
    }
}

/// CollisionType tells game what to do when this object
/// collides with another.
pub enum CollisionType {
    NoCollide,
    Collide,
    Damage(u8),
    Kill,
}

/// Tile contains all the information about an individual tile
/// in the scene.
pub struct Tile {
    pub en: Entity,
    pub collision: CollisionType,
}

impl Tile {
    pub fn new(p: Point, cr: Rect, t: Rc<Texture>, dr: Option<Rect>, ct: CollisionType) -> Self {
        Tile {
            en: Entity::new(p, cr, t, dr),
            collision: ct,
        }
    }
}

pub struct MoveableEntity {
    pub en: Entity,
    pub dir: Direction,
    pub v: Velocity,
    pub a: Acceleration,
}

impl MoveableEntity {
    pub fn new(p: Point,
               cr: Rect,
               t: Rc<Texture>,
               dr: Option<Rect>,
               d: Direction,
               v: Velocity,
               a: Acceleration) -> Self {
        MoveableEntity {
            en: Entity::new(p, cr, t, dr),
            dir: d,
            v: v,
            a: a,
        }
    }
}

pub struct Game {
    pub running: bool,
    pub player: MoveableEntity,
}

impl Game {
    pub fn new(p: MoveableEntity) -> Self {
        Game {
            running: true,
            player: p,
        }
    }
}

pub struct System<'a> {
    pub game: Game,
    pub r: Renderer<'a>,
    pub fc: u8,
    pub fps: u8,
    pub timer: Receiver<()>,
    pub ev_pump: EventPump,
    pub assets: &'a Path,
}

impl<'a> System<'a> {
    pub fn new(g: Game, r: Renderer<'a>, fps: u8, ep: EventPump, a: &'a Path) -> Self {
        System {
            game: g,
            r: r,
            fc: 0,
            fps: fps,
            timer: timer_periodic(1000/fps as u32),
            ev_pump: ep,
            assets: a,
        }
    }
}

pub trait Drawable {
    fn draw(&self, r: &mut Renderer);
}

impl Drawable for Game {
    fn draw(&self, r: &mut Renderer) {
        r.clear();
        self.player.draw(r);
        r.present();
    }
}

impl Drawable for Entity {
    fn draw(&self, r: &mut Renderer) {
        let (w, h) = if let Some(dr) = self.draw_rect {
            (dr.width(), dr.height())
        } else {
            let q = self.sprite_map.query();
            (q.width, q.height)
        };
        r.copy(&self.sprite_map, self.draw_rect,
            Rect::new(self.pos.x as i32, self.pos.y as i32, w, h).unwrap());
    }
}

impl Drawable for MoveableEntity {
    fn draw(&self, r: &mut Renderer) {
        self.en.draw(r);
    }
}

pub trait Updateable {
    fn update(&mut self);
}

impl Updateable for Game {
    fn update(&mut self) {
        self.player.update();
    }
}

impl<'a> Updateable for System<'a> {
    fn update(&mut self) {
        let _ = self.timer.recv();
        self.fc += 1;
        if self.fc > self.fps {
            self.fc = 0;
        }

        for event in self.ev_pump.poll_iter() {
            match event {
                Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
                    self.game.running = false
                },
                _ => ()
            }
        }

        if self.ev_pump.keyboard_state().is_scancode_pressed(Scancode::Left) {
            self.game.player.a.x -= 1.0;
        }
        if self.ev_pump.keyboard_state().is_scancode_pressed(Scancode::Right) {
            self.game.player.a.x += 1.0;
        }

        self.game.update();
    }
}

impl Updateable for MoveableEntity {
    fn update(&mut self) {
        const MOVEABLE_VELOCITY_DECAY_FACTOR: f64 = 0.9;
        const MOVEABLE_VELOCITY_CUTOFF: f64 = 0.1;
        const MOVEABLE_ACCELERATION_DECAY_FACTOR: f64 = 0.9;
        const MOVEABLE_ACCELERATION_CUTOFF: f64 = 1.0;
        self.en.pos.x += self.v.x as i64;
        self.en.pos.y += self.v.y as i64;
        self.v.x += self.a.x;
        self.v.y += self.a.y;

        self.v.x *= MOVEABLE_VELOCITY_DECAY_FACTOR;
        self.v.y *= MOVEABLE_VELOCITY_DECAY_FACTOR;
        if self.v.x < MOVEABLE_VELOCITY_CUTOFF &&
           self.v.x > -MOVEABLE_VELOCITY_CUTOFF { self.v.x = 0.0; }
        if self.v.y < MOVEABLE_VELOCITY_CUTOFF &&
           self.v.y > -MOVEABLE_VELOCITY_CUTOFF { self.v.y = 0.0; }

        self.a.x *= MOVEABLE_ACCELERATION_DECAY_FACTOR;
        self.a.y *= MOVEABLE_ACCELERATION_DECAY_FACTOR;
        if self.a.x < MOVEABLE_ACCELERATION_CUTOFF &&
           self.a.x > -MOVEABLE_ACCELERATION_CUTOFF { self.a.x = 0.0; }
        if self.a.y < MOVEABLE_ACCELERATION_CUTOFF &&
           self.a.y > -MOVEABLE_ACCELERATION_CUTOFF { self.a.y = 0.0; }
    }
}
