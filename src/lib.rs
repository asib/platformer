extern crate sdl2;
extern crate sdl2_image;
extern crate rustc_serialize;

use std::rc::Rc;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use sdl2::EventPump;
use sdl2::render::{Renderer, Texture};
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;

pub mod tiled;
pub mod map;

#[macro_export]
macro_rules! hashmap {
    ($($k:expr => $v:expr),*) => ({
        let mut _tmp = std::collections::HashMap::new();
        $(_tmp.insert($k, $v);)*
        _tmp
    });
}

/// Returns a timer that sends the unit every
/// `ms` milliseconds.
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

/// Contains x, y position components.
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    /// Helper method to save typing out
    /// the origin Point struct.
    pub fn origin() -> Self {
        Point{x: 0, y: 0}
    }
}

/// Contains x, y velocity components.
pub struct Velocity {
    pub x: f64,
    pub y: f64,
}

impl Velocity {
    /// Helper method to save typing out
    /// the zero Velocity struct.
    pub fn zero() -> Self {
        Velocity{x: 0.0, y: 0.0}
    }
}

/// Contains x, y acceleration components.
pub struct Acceleration {
    pub x: f64,
    pub y: f64,
}

impl Acceleration {
    /// Helper method to save typing out
    /// the zero Acceleration struct.
    pub fn zero() -> Self {
        Acceleration{x: 0.0, y: 0.0}
    }
}

/// Enumeration of directions in a platformer.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Direction {
    Up,
    DoubleUp,
    Down,
    Left,
    StillLeft,
    Right,
    StillRight,
    Landed,
}

/// Holds information pertaining to the game's camera.
pub struct Camera {
    pub pos: Point,
    pub width: i64,
    pub height: i64,
    pub collision_rect: Rect,
}

impl Camera {
    /// Create a new `Camera`
    pub fn new(p: Point, w: i64, h: i64, cr: Rect) -> Self {
        Camera {
            pos: p,
            width: w,
            height: h,
            collision_rect: cr,
        }
    }
}

/// Building block struct that holds the basic
/// data that all game entities need.
pub struct Entity {
    pub pos: Point,
    pub collision_rect: Rect,
    pub sprite_map: Rc<Texture>,
    pub draw_rect: Option<Rect>,
}

impl Entity {
    /// Create a new `Entity`.
    fn new(p: Point, cr: Rect, t: Rc<Texture>, dr: Option<Rect>) -> Self {
        Entity {
            pos: p,
            collision_rect: cr,
            sprite_map: t,
            draw_rect: dr,
        }
    }
}

/// Contains all the data for animating a sprite.
pub struct Animation {
    /// Sprite counter.
    pub sc: u8,
    /// A `HashMap` from `Direction` to animation length in frames,
    /// used to calculate when to change the sprite frame.
    pub dir_to_anim_len: HashMap<Direction, u8>,
    /// Animation counter - holds how many frames into the
    /// current animation loop the entity is.
    pub ac: u8,
    /// A `HashMap` that is used when animating the sprite
    /// so that we know how many frames each direction has.
    /// This allows non-uniform sprite maps (e.g. 5 frames for
    /// left/right, but 1 frame for jump/fall).
    pub dir_to_frames: HashMap<Direction, u8>,
    /// A `HashMap` that contains offsets for sprites in the
    /// sprite map.
    pub dir_to_offset: HashMap<Direction, Point>,
    /// A `HashMap` that holds the `y`-offset for each `Direction`
    /// in the sprite map.
    pub dir_to_pos: HashMap<Direction, u8>,
    /// Whether the animation needs to be run forwards or backwards.
    pub reverse: bool,
}

impl Animation {
    pub fn new(dtal: HashMap<Direction, u8>,
               dtf: HashMap<Direction, u8>,
               dto: HashMap<Direction, Point>,
               dtp: HashMap<Direction, u8>,
               reverse: bool) -> Self {
        Animation {
            sc: 1,
            dir_to_anim_len: dtal,
            ac: 0,
            dir_to_frames: dtf,
            dir_to_offset: dto,
            dir_to_pos: dtp,
            reverse: reverse,
        }
    }
}

/// A game entity that moves and is animated.
pub struct MoveableEntity {
    pub en: Entity,
    pub dir: Direction,
    /// The last `Direction` the entity was going.
    pub l_dir: Direction,
    pub v: Velocity,
    pub a: Acceleration,
    pub anim: Option<Animation>,
}

impl MoveableEntity {
    /// Create a new `MoveableEntity`.
    /// Number of frames for `Up`, `Down`, `Left` and `Right`
    /// animations are passed through `uc`, `dc`, `lc`,
    /// `rc`.
    pub fn new(p: Point,
               cr: Rect,
               t: Rc<Texture>,
               dr: Option<Rect>,
               d: Direction,
               v: Velocity,
               a: Acceleration,
               anim: Option<Animation>) -> Self {
        MoveableEntity {
            en: Entity::new(p, cr, t, dr),
            dir: d.clone(),
            l_dir: d,
            v: v,
            a: a,
            anim: anim,
        }
    }

    pub fn keep_on_screen(&mut self, w: u32, h: u32) {
        if (self.en.collision_rect.x() as i64 + self.en.pos.x) < 0 {
            self.en.pos.x = -self.en.collision_rect.x() as i64;
        } else if (self.en.collision_rect.x() as i64 + self.en.pos.x + self.en.collision_rect.width() as i64) > w as i64 {
            self.en.pos.x = w as i64 - (self.en.collision_rect.width() as i64 + self.en.collision_rect.x() as i64);
        }
        if self.en.pos.y < 0 {
            self.en.pos.y = 0;
        } else if (self.en.pos.y + self.en.collision_rect.height() as i64) > h as i64 {
            self.en.pos.y = h as i64 - self.en.collision_rect.height() as i64;
            match self.dir {
                Direction::Up | Direction::DoubleUp => self.change_dir(Direction::Landed),
                _ => (),
            }
        }
    }

    fn reset_anim(&mut self) {
        if let &mut Some(ref mut anim) = &mut self.anim {
            anim.sc = 1;
            anim.ac = 0;
        }
    }

    pub fn change_dir(&mut self, d: Direction) {
        if d == Direction::Landed {
            self.dir = self.l_dir.clone();
            self.l_dir = d;
            return;
        } else if self.dir == Direction::Up || self.dir == Direction::DoubleUp {
            return;
        }

        self.l_dir = self.dir.clone();
        self.dir = d.clone();

        if d == Direction::StillLeft || d == Direction::StillRight {
            self.reset_anim();
        }
    }
}

/// Specialised version of `MoveableEntity` to allow for
/// player-specific mechanics and methods.
pub struct Player {
    pub me: MoveableEntity,
}

impl Player {
    pub fn new(p: Point,
               cr: Rect,
               t: Rc<Texture>,
               dr: Option<Rect>,
               d: Direction,
               dtp: HashMap<Direction, u8>,
               dtal: HashMap<Direction, u8>,
               dtf: HashMap<Direction, u8>,
               dto: HashMap<Direction, Point>,
               reverse: bool) -> Self {
        Player {
            me: MoveableEntity::new(
                p,
                cr,
                t,
                dr,
                d,
                Velocity::zero(),
                Acceleration::zero(),
                Some(Animation::new(
                    dtal,
                    dtf,
                    dto,
                    dtp,
                    reverse
                ))
            ),
        }
    }

    pub fn keep_on_screen(&mut self, w: u32, h: u32) {
        self.me.keep_on_screen(w, h);
    }

    pub fn jump(&mut self) {
        match self.me.dir {
            Direction::DoubleUp => return,
            Direction::Up => {
                self.me.dir = Direction::DoubleUp;
            },
            _ => {
                self.me.change_dir(Direction::Up);
            },
        }
        self.me.v.y = -55.0;
    }
}

/// Holds pure game data, as opposed to `System`,
/// which holds system data like the frame counter.
pub struct Game<'a> {
    pub running: bool,
    pub debug: bool,
    pub current_map: Option<&'a mut map::Map>,
    pub camera: Camera,
    pub player: Player,
}

impl<'a> Game<'a> {
    /// Create a new `Game`.
    pub fn new(db: bool, current_map: Option<&'a mut map::Map>, cam: Camera, p: Player) -> Self {
        Game {
            running: true,
            debug: db,
            current_map: current_map,
            camera: cam,
            player: p,
        }
    }

    pub fn set_map(&mut self, map: &'a mut map::Map) {
        self.current_map = Some(map);
    }

    pub fn clear(&self, r: &mut Renderer) {
        r.clear();
    }

    pub fn flip_buffer(&self, r: &mut Renderer) {
        r.present();
    }

    pub fn keep_on_screen(&mut self, w: u32, h: u32) {
        self.player.keep_on_screen(w, h);
    }

    fn update_camera(&mut self) {
        let (camera_left, camera_right, camera_top, camera_bottom) = (
            self.camera.pos.x + self.camera.collision_rect.x() as i64,
            self.camera.pos.x + self.camera.collision_rect.x() as i64 + self.camera.collision_rect.width() as i64,
            self.camera.pos.y + self.camera.collision_rect.y() as i64,
            self.camera.pos.y + self.camera.collision_rect.y() as i64 + self.camera.collision_rect.height() as i64,
        );
        let (player_left, player_right, player_top, player_bottom) = (
            self.player.me.en.pos.x + self.player.me.en.collision_rect.x() as i64,
            self.player.me.en.pos.x + self.player.me.en.collision_rect.x() as i64 + self.player.me.en.collision_rect.width() as i64,
            self.player.me.en.pos.y + self.player.me.en.collision_rect.y() as i64,
            self.player.me.en.pos.y + self.player.me.en.collision_rect.y() as i64 + self.player.me.en.collision_rect.height() as i64,
        );

        let map = self.current_map.as_ref().unwrap().clone();

        if player_right > camera_right {
            self.camera.pos.x = player_right - self.camera.collision_rect.width() as i64 - self.camera.collision_rect.x() as i64;
        } else if player_left < camera_left {
            self.camera.pos.x = player_left - self.camera.collision_rect.x() as i64;
        }

        if player_bottom > camera_bottom {
            self.camera.pos.y = player_bottom - self.camera.collision_rect.height() as i64;
        } else if player_top < camera_top {
            self.camera.pos.y = player_top;
        }

        // keep the camera within the map
        if self.camera.pos.x+self.camera.width > map.pixel_width() as i64 {
            self.camera.pos.x = map.pixel_width() as i64 - self.camera.width;
        } else if self.camera.pos.x < 0 {
            self.camera.pos.x = 0;
        }

        if self.camera.pos.y+self.camera.height > map.pixel_height() as i64 {
            self.camera.pos.y = map.pixel_height() as i64 - self.camera.height;
        } else if self.camera.pos.y < 0 {
            self.camera.pos.y = 0;
        }
    }
}

/// Contains system data like the renderer,
/// frame counter, fps timer, etc...
pub struct System<'a> {
    pub game: Game<'a>,
    pub r: Renderer<'a>,
    pub fc: u8,
    pub fps: u8,
    pub timer: Receiver<()>,
    pub ev_pump: EventPump,
    pub assets: &'a Path,
}

impl<'a> System<'a> {
    /// Create a new `System`.
    pub fn new(g: Game<'a>, r: Renderer<'a>, fps: u8, ep: EventPump, a: &'a Path) -> Self {
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

pub trait DebugDrawable {
    fn draw_debug(&mut self, r: &mut Renderer);
}

impl<'a> DebugDrawable for Game<'a> {
    fn draw_debug(&mut self, r: &mut Renderer) {
        self.camera.draw_debug(r);
        self.player.draw_debug(r, &self.camera);
    }
}

impl DebugDrawable for Camera {
    fn draw_debug(&mut self, r: &mut Renderer) {
        let rect = &self.collision_rect;
        let draw_col = r.draw_color();
        r.set_draw_color(Color::RGB(255, 0, 0));
        r.draw_rect(Rect::new_unwrap(
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height()
        ));
        r.set_draw_color(draw_col);
    }
}

pub trait CameraDebugDrawable {
    fn draw_debug(&mut self, r: &mut Renderer, c: &Camera);
}

impl CameraDebugDrawable for Player {
    fn draw_debug(&mut self, r: &mut Renderer, c: &Camera) {
        self.me.draw_debug(r, c);
    }
}

impl CameraDebugDrawable for MoveableEntity {
    fn draw_debug(&mut self, r: &mut Renderer, c: &Camera) {
        self.en.draw_debug(r, c);
    }
}

impl CameraDebugDrawable for Entity {
    fn draw_debug(&mut self, r: &mut Renderer, c: &Camera) {
        let rect = &self.collision_rect;
        let draw_col = r.draw_color();
        r.set_draw_color(Color::RGB(255, 0, 0));
        r.draw_rect(Rect::new_unwrap(
            rect.x() + self.pos.x as i32 - c.pos.x as i32,
            rect.y() + self.pos.y as i32 - c.pos.y as i32,
            rect.width(),
            rect.height()
        ));
        r.set_draw_color(draw_col);
    }
}

/// The `Drawable` trait should be implemented by
/// anything that needs to do something during the
/// rendering process.
pub trait Drawable {
    fn draw(&mut self, r: &mut Renderer);
}

impl<'a> Drawable for Game<'a> {
    /// `Game`'s `draw` method calls the draw methods
    /// for all entities that are currently onscreen.
    fn draw(&mut self, r: &mut Renderer) {
        if let Some(ref mut map) = self.current_map {
            map.draw(r, &self.camera);
        }
        self.player.draw(r, &self.camera);
    }
}

/// The `CameraDrawable` trait should be implemented by
/// anything that requires camera data during the
/// rendering process.
pub trait CameraDrawable {
    fn draw(&mut self, r: &mut Renderer, c: &Camera);
}

impl CameraDrawable for Entity {
    fn draw(&mut self, r: &mut Renderer, c: &Camera) {
        let (w, h) = if let Some(dr) = self.draw_rect {
            (dr.width(), dr.height())
        } else {
            let q = self.sprite_map.query();
            (q.width, q.height)
        };

        // calculate screen x, y, using camera coordinates
        let (screen_x, screen_y) = (
            self.pos.x - c.pos.x,
            self.pos.y - c.pos.y
        );
        r.copy(&self.sprite_map, self.draw_rect,
            Rect::new(screen_x as i32, screen_y as i32, w, h).unwrap());
    }
}

impl CameraDrawable for MoveableEntity {
    fn draw(&mut self, r: &mut Renderer, c: &Camera) {
        if let (Some(dr), &Some(ref anim)) = (self.en.draw_rect, &self.anim) {
            // Calculate draw_rect
            let off = anim.dir_to_offset.get(&self.dir).unwrap();
            let frames = *anim.dir_to_frames.get(&self.dir).unwrap();
            let sc = if anim.reverse && frames > 1 {
                (frames - anim.sc) as u32
            } else {
                anim.sc as u32
            };
            let dir_pos = *anim.dir_to_pos.get(&self.dir).unwrap() as u32;
            self.en.draw_rect = Some(Rect::new_unwrap(
                (off.x as u32 + sc * dr.width()) as i32,
                (off.y as u32 + dir_pos * dr.height()) as i32,
                dr.width(),
                dr.height()
            ));
        }

        self.en.draw(r, c);
    }
}

impl CameraDrawable for Player {
    fn draw(&mut self, r: &mut Renderer, c: &Camera) {
        self.me.draw(r, c);
    }
}

pub trait Updateable {
    fn update(&mut self);
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
                Event::KeyDown{keycode: Some(Keycode::Space), ..} => self.game.player.jump(),
                _ => ()
            }
        }

        {
            let me = &mut self.game.player.me;
            const HORIZONTAL_ACCELERATION: f64 = 9.5;
            if self.ev_pump.keyboard_state().is_scancode_pressed(Scancode::Left) {
                me.a.x -= HORIZONTAL_ACCELERATION;
                me.change_dir(Direction::Left);
            } else if self.ev_pump.keyboard_state().is_scancode_pressed(Scancode::Right) {
                me.a.x += HORIZONTAL_ACCELERATION;
                me.change_dir(Direction::Right);
            }
        }

        self.game.update();
        if self.game.current_map.is_some() {
            let (mut w, mut h) = (0, 0);
            {
                match self.game.current_map {
                    Some(ref map) => {
                        w = map.width*map.tile_width;
                        h = map.height*map.tile_height;
                    },
                    None => {}
                };
            }
            self.game.keep_on_screen(w, h);
        }
    }
}

impl<'a> Updateable for Game<'a> {
    fn update(&mut self) {
        self.player.update();

        if self.current_map.is_some() {
            self.update_camera();
        }
    }
}


impl Updateable for MoveableEntity {
    fn update(&mut self) {
        if self.v.x == 0.0 {
            match self.dir {
                Direction::Right => self.change_dir(Direction::StillRight),
                Direction::Left => self.change_dir(Direction::StillLeft),
                _ => (),
            }
        }

        if self.en.draw_rect == None {
            return;
        }

        if let &mut Some(ref mut anim) = &mut self.anim {
            let anim_len = *anim.dir_to_anim_len.get(&self.dir).unwrap();
            let frame_count = *anim.dir_to_frames.get(&self.dir).unwrap();
            let change_every = anim_len / frame_count;
            if anim.ac % change_every == 0 {
                anim.sc += 1;
                if anim.sc > (frame_count-1) {
                    anim.sc = 0;
                }
            }

            anim.ac += 1;
            if anim.ac > anim_len {
                anim.ac = 1;
            }
        }
    }
}

impl Updateable for Player {
    fn update(&mut self) {
        const MOVEABLE_VELOCITY_DECAY_FACTOR_X: f64 = 0.2;
        const MOVEABLE_VELOCITY_DECAY_FACTOR_Y: f64 = 0.7;
        const MOVEABLE_VELOCITY_CUTOFF: f64 = 2.0;
        const MOVEABLE_ACCELERATION_DECAY_FACTOR_X: f64 = 0.80;
        const MOVEABLE_ACCELERATION_CUTOFF: f64 = 0.1;
        self.me.a.y = 9.8;
        self.me.en.pos.x += self.me.v.x as i64;
        self.me.en.pos.y += self.me.v.y as i64;
        self.me.v.x += self.me.a.x;
        self.me.v.y += self.me.a.y;

        self.me.v.x *= MOVEABLE_VELOCITY_DECAY_FACTOR_X;
        self.me.v.y *= MOVEABLE_VELOCITY_DECAY_FACTOR_Y;
        if self.me.v.x < MOVEABLE_VELOCITY_CUTOFF &&
           self.me.v.x > -MOVEABLE_VELOCITY_CUTOFF { self.me.v.x = 0.0; }
        if self.me.v.y < MOVEABLE_VELOCITY_CUTOFF &&
           self.me.v.y > -MOVEABLE_VELOCITY_CUTOFF { self.me.v.y = 0.0; }

        self.me.a.x *= MOVEABLE_ACCELERATION_DECAY_FACTOR_X;
        if self.me.a.x < MOVEABLE_ACCELERATION_CUTOFF &&
           self.me.a.x > -MOVEABLE_ACCELERATION_CUTOFF { self.me.a.x = 0.0; }

        self.me.update();
    }
}
