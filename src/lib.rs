extern crate sdl2;

use std::rc::Rc;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use sdl2::EventPump;
use sdl2::render::{Renderer, Texture};
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};

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
    Down,
    Left,
    Up,
    Right,
    len,
}

impl Direction {
    /// Used when calculating the draw_rect of
    /// an animated sprite.
    fn to_int(&self) -> u8 {
        match *self {
            Direction::Down  => 0,
            Direction::Left  => 1,
            Direction::Up    => 2,
            Direction::Right => 3,
            Direction::len   => 4,
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

/// Tells game what to do when this object
/// collides with another.
pub enum CollisionType {
    NoCollide,
    Collide,
    Damage(u8),
    Kill,
}

/// Contains all the information about an individual
/// tile in the scene.
pub struct Tile {
    pub en: Entity,
    pub collision: CollisionType,
}

impl Tile {
    /// Create a new `Tile`.
    pub fn new(p: Point, cr: Rect, t: Rc<Texture>, dr: Option<Rect>, ct: CollisionType) -> Self {
        Tile {
            en: Entity::new(p, cr, t, dr),
            collision: ct,
        }
    }
}

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
}

impl Animation {
    pub fn new(ul: u8,
               dl: u8,
               ll: u8,
               rl: u8,
               uc: u8,
               dc: u8,
               lc: u8,
               rc: u8) -> Self {
        let mut dtal = HashMap::with_capacity(Direction::len.to_int() as usize);
        dtal.insert(Direction::Up, ul);
        dtal.insert(Direction::Down, dl);
        dtal.insert(Direction::Left, ll);
        dtal.insert(Direction::Right, rl);
        let mut dtf = HashMap::with_capacity(Direction::len.to_int() as usize);
        dtf.insert(Direction::Up, uc);
        dtf.insert(Direction::Down, dc);
        dtf.insert(Direction::Left, lc);
        dtf.insert(Direction::Right, rc);
        Animation {
            sc: 1,
            dir_to_anim_len: dtal,
            ac: 0,
            dir_to_frames: dtf,
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
        if self.en.pos.x < 0 {
            self.en.pos.x = 0;
        } else if (self.en.pos.x + self.en.collision_rect.width() as i64) > w as i64 {
            self.en.pos.x = w as i64 - self.en.collision_rect.width() as i64;
        }
        if self.en.pos.y < 0 {
            self.en.pos.y = 0;
        } else if (self.en.pos.y + self.en.collision_rect.height() as i64) > h as i64 {
            self.en.pos.y = h as i64 - self.en.collision_rect.height() as i64;
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
               ul: u8,
               dl: u8,
               ll: u8,
               rl: u8,
               uc: u8,
               dc: u8,
               lc: u8,
               rc: u8) -> Self {
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
                    ul,
                    dl,
                    ll,
                    rl,
                    uc,
                    dc,
                    lc,
                    rc
                ))
            ),
        }
    }

    pub fn keep_on_screen(&mut self, w: u32, h: u32) {
        self.me.keep_on_screen(w, h);
    }

    pub fn jump(&mut self) {
        self.me.v.y = -55.0;
    }
}

/// Holds pure game data, as opposed to `System`,
/// which holds system data like the frame counter.
pub struct Game {
    pub running: bool,
    pub player: Player,
}

impl Game {
    /// Create a new `Game`.
    pub fn new(p: Player) -> Self {
        Game {
            running: true,
            player: p,
        }
    }

    pub fn keep_on_screen(&mut self, w: u32, h: u32) {
        self.player.keep_on_screen(w, h);
    }
}

/// Contains system data like the renderer,
/// frame counter, fps timer, etc...
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
    /// Create a new `System`.
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

    pub fn update(&mut self) {
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

        if self.ev_pump.keyboard_state().is_scancode_pressed(Scancode::Left) {
            self.game.player.me.a.x -= 9.5;
            self.game.player.me.dir = Direction::Left;
        } else if self.ev_pump.keyboard_state().is_scancode_pressed(Scancode::Right) {
            self.game.player.me.a.x += 9.5;
            self.game.player.me.dir = Direction::Right;
        }

        self.game.update(self.fc);
        let (w, h) = self.r.window().unwrap().size();
        self.game.keep_on_screen(w, h);
    }
}

/// The `Drawable` trait should be implemented by
/// anything that needs to do something during the
/// rendering process.
pub trait Drawable {
    fn draw(&mut self, r: &mut Renderer);
}

impl Drawable for Game {
    /// `Game`'s `draw` method calls the draw methods
    /// for all entities that are currently onscreen.
    fn draw(&mut self, r: &mut Renderer) {
        r.clear();
        self.player.draw(r);
        r.present();
    }
}

impl Drawable for Entity {
    fn draw(&mut self, r: &mut Renderer) {
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
    fn draw(&mut self, r: &mut Renderer) {
        if let (Some(dr), &Some(ref anim)) = (self.en.draw_rect, &self.anim) {
            // Calculate draw_rect
            self.en.draw_rect = Some(Rect::new_unwrap(
                (anim.sc as u32 * dr.width()) as i32,
                (self.dir.to_int() as u32 * dr.height()) as i32,
                dr.width(),
                dr.height()
            ));
        }

        self.en.draw(r);
    }
}

impl Drawable for Player {
    fn draw(&mut self, r: &mut Renderer) {
        self.me.draw(r);
    }
}

pub trait Updateable {
    fn update(&mut self, fc: u8);
}

impl Updateable for Game {
    fn update(&mut self, fc: u8) {
        self.player.update(fc);
    }
}


impl Updateable for MoveableEntity {
    fn update(&mut self, fc: u8) {
        if self.en.draw_rect == None {
            return;
        }

        if let &mut Some(ref mut anim) = &mut self.anim {
            let frame_count = *anim.dir_to_frames.get(&self.dir).unwrap();
            let anim_len = *anim.dir_to_anim_len.get(&self.dir).unwrap();
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
    fn update(&mut self, fc: u8) {
        const MOVEABLE_VELOCITY_DECAY_FACTOR_X: f64 = 0.2;
        const MOVEABLE_VELOCITY_DECAY_FACTOR_Y: f64 = 0.7;
        const MOVEABLE_VELOCITY_CUTOFF: f64 = 0.1;
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

        self.me.update(fc);
    }
}
