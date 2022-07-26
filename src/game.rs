use crate::lib::kinput::*;
use crate::lib::kmath::*;
use crate::krenderer::*;

use glutin::event::VirtualKeyCode;

// yea maybe the event system cleans up the spawning situation

// procedural clouds!! should be easy, rect for straight bottom and variably sized and offset circles
// fade and parallax

// Can't trigger more than once per frame

pub struct RngSequence {
    seed: u32,
}

impl RngSequence {
    pub fn new(seed: u32) -> RngSequence {
        RngSequence {
            seed
        }
    }
    pub fn sample(&mut self) -> u32 {
        let res = khash(self.seed);
        self.seed = khash(self.seed + 394712377);
        res
    }
    pub fn peek(&self) -> u32 {
        khash(self.seed)
    }
}

pub struct RepeatTimer {
    t: f64,
    t_next: f64,
    period: f64,
}

impl RepeatTimer {
    pub fn new(period: f64) -> RepeatTimer {
        return RepeatTimer { 
            t: 0.0, 
            t_next: period, // nb
            period: period,
        };
    }

    pub fn tick(&mut self, dt: f64) -> bool {
        self.t += dt;
        if self.t >= self.t_next {
            self.t_next += self.period;
            return true;
        }
        return false;
    }
}

pub struct Game {
    player_position: f32,
    player_velocidad: f32,

    grav_dir: f32,
    
    t: f64,

    score: f64,

    wall_sequence: RngSequence,
    wall_spawn_timer: RepeatTimer,

    walls: Vec<Rect>,
    pickups: Vec<Vec2>,

    clouds_far: Vec<(u32, f32)>,
    clouds_mid: Vec<(u32, f32)>,
    clouds_near: Vec<(u32, f32)>,

    cloud_spawn_timer: RepeatTimer,

    pub paused: bool,
    dead: bool,
}

impl Game {
    pub fn new(seed: u32) -> Game {
        Game {
            player_position: 0.3,
            player_velocidad: 0.0,

            grav_dir: 1.0,

            t: 0.0,

            paused: false,

            score: 0.0,

            wall_sequence: RngSequence::new(seed * 34982349),
            wall_spawn_timer: RepeatTimer::new(2.0),
            walls: Vec::new(),
            pickups: Vec::new(),

            clouds_far: Vec::new(),
            clouds_mid: Vec::new(),
            clouds_near: Vec::new(),

            cloud_spawn_timer: RepeatTimer::new(1.0),

            dead: false,
        }
    }
    
    pub fn frame(&mut self, inputs: &FrameInputState, kc: &mut KRCanvas) {
        let gravity = 1.8;
        let player_x = 0.5;
        let player_radius = 0.02;
        let forgive_radius = 0.01;
        let pickup_radius = 0.02;
        let pickup_score = 1000.0;

        let wall_speed = 0.45;
        let gap_h = 0.4;
        let wall_w = 0.2;

        
        if inputs.just_pressed(VirtualKeyCode::Space) {
            // self.player_velocidad = -1.0;
            self.grav_dir *= -1.0;
        }
        
        
        if !self.paused && !self.dead {
            self.t += inputs.dt;
            self.score += inputs.dt * 100.0;
            self.player_velocidad += gravity * inputs.dt as f32 * self.grav_dir;
            self.player_position += self.player_velocidad * inputs.dt as f32;
            for wall in self.walls.iter_mut() {
                wall.x -= wall_speed * inputs.dt as f32;
            }
            for pickup in self.pickups.iter_mut() {
                pickup.x -= wall_speed * inputs.dt as f32;
            }

            // spawn clouds
            if self.cloud_spawn_timer.tick(inputs.dt) {
                if chance(inputs.seed * 1295497987, 0.1) {
                    self.clouds_near.push((inputs.seed * 982894397, inputs.screen_rect.right() + 0.2));
                }
                if chance(inputs.seed * 35873457, 0.2) {
                    self.clouds_mid.push((inputs.seed * 3842348749, inputs.screen_rect.right() + 0.2));
                }
                if chance(inputs.seed * 576345763, 0.3) {
                    self.clouds_far.push((inputs.seed * 934697577, inputs.screen_rect.right() + 0.2));
                }

            }

            // move clouds
            for i in 0..self.clouds_near.len() {
                let (seed, pos) = self.clouds_near[i];
                self.clouds_near[i] = (seed, pos - inputs.dt as f32 * 0.1);
            }
            for i in 0..self.clouds_mid.len() {
                let (seed, pos) = self.clouds_mid[i];
                self.clouds_mid[i] = (seed, pos - inputs.dt as f32 * 0.05);
            }
            for i in 0..self.clouds_far.len() {
                let (seed, pos) = self.clouds_far[i];
                self.clouds_far[i] = (seed, pos - inputs.dt as f32 * 0.025);
            }
        }


        if self.wall_spawn_timer.tick(inputs.dt) {
            let h = kuniform(self.wall_sequence.sample(), 0.0, inputs.screen_rect.bot() - gap_h);
            self.walls.push(Rect::new(inputs.screen_rect.right(), -10.0, wall_w, 10.0 + h));
            self.walls.push(Rect::new(inputs.screen_rect.right(), h + gap_h, wall_w, 10.4));
            
            let halfway = ((self.wall_spawn_timer.period / 2.0) * wall_speed as f64) as f32;
            if chance(self.wall_sequence.peek() * 3458793547, 0.5) {
                // place a pickup
                let h =  if chance(inputs.seed * 123891, 0.5) {inputs.screen_rect.top() + 0.2} else {inputs.screen_rect.bot() - 0.2};
                let new_pickup = Vec2::new(inputs.screen_rect.right() + pickup_radius + halfway + wall_w/2.0, h);
                self.pickups.push(new_pickup);
            } else {
                // place an intermediate wall
                if chance(self.wall_sequence.peek() * 548965757, 0.1) {
                    let next_h = kuniform(self.wall_sequence.peek(), 0.0, inputs.screen_rect.bot() - gap_h);
                    let h = (h + next_h)/2.0;
                    self.walls.push(Rect::new(inputs.screen_rect.right() + halfway, -10.0, wall_w, 10.0 + h));
                    self.walls.push(Rect::new(inputs.screen_rect.right() + halfway, h + gap_h, wall_w, 10.4));
                }
            }
        }

        if inputs.just_pressed(VirtualKeyCode::R) {
            *self = Game::new(inputs.seed);
            return;
        }

        // player collides with walls
        let player_pos = Vec2::new(player_x, self.player_position);
        for wall in self.walls.iter() {
            let closest_point = wall.snap(player_pos);
            let penetration = player_radius - (closest_point - player_pos).magnitude();
            if penetration > 0.0 {
                self.dead = true;
            }
        }
        
        if self.player_position < inputs.screen_rect.top() || self.player_position > inputs.screen_rect.bot() {
            self.dead = true;
        }

        let mut i = self.pickups.len();
        while i > 0 {
            i = i - 1;
            if self.pickups[i].dist(player_pos) < player_radius + pickup_radius + forgive_radius {
                self.score += pickup_score;
                self.pickups.swap_remove(i);
            } else {
                if self.pickups[i].x - pickup_radius < 0.0 {
                    self.pickups.swap_remove(i);
                }
            }
        }
        
        self.walls.retain(|w| w.right() > 0.0);

        // bg
        kc.set_camera(inputs.screen_rect);
        kc.set_depth(1.0);
        kc.set_colour(Vec4::new(0.3, 0.3, 0.7, 1.0));
        kc.rect(inputs.screen_rect);
        
        kc.set_depth(1.05);
        kc.set_colour(Vec4::new(0.1, 0.1, 0.8, 1.0));
        kc.rect(inputs.screen_rect.child(0.0, 0.7, 1.0, 1.0));

        // clouds
        kc.set_depth(1.1);
        kc.set_colour(Vec4::new(0.6, 0.6, 0.7, 1.0));
        for (seed, xpos) in &self.clouds_far {
            kc.cloud(Rect::new(*xpos, 0.6, 0.1, 0.05), *seed)            
        }
        kc.set_depth(1.2);
        kc.set_colour(Vec4::new(0.7, 0.7, 0.8, 1.0));
        for (seed, xpos) in &self.clouds_mid {
            kc.cloud(Rect::new(*xpos, 0.533, 0.15, 0.07), *seed)            
        }
        kc.set_depth(1.3);
        kc.set_colour(Vec4::new(0.8, 0.8, 0.9, 1.0));
        for (seed, xpos) in &self.clouds_near {
            kc.cloud(Rect::new(*xpos, 0.467, 0.2, 0.09), *seed)            
        }
        
        // player
        kc.set_depth(1.5);
        kc.set_colour(Vec4::new(1.0, 1.0, 0.0, 1.0));
        kc.circle(player_pos, player_radius + forgive_radius);

        // walls
        kc.set_colour(Vec4::new(0.0, 0.7, 0.0, 1.0));
        for wall in self.walls.iter() {
            kc.rect(*wall);
        }
        // pickups
        kc.set_colour(Vec4::new(0.8, 0.0, 0.0, 1.0));
        for pickup in self.pickups.iter() {
            kc.circle(*pickup, 0.02);
        }

        kc.set_depth(2.0);
        kc.set_colour(Vec4::new(1.0, 1.0, 1.0, 1.0));
        if self.dead {
            let sr = inputs.screen_rect.dilate_pc(-0.6);
            kc.text_center(format!("{:.0}", self.score).as_bytes(), sr);
            kc.text_center("you died, press R to reset".as_bytes(), sr.translate(Vec2::new(0.0, sr.h))); // bug ???
        } else {
            kc.text_center(format!("{:.0}", self.score).as_bytes(), inputs.screen_rect.child(0.0, 0.0, 1.0, 0.05));
        }
        
        if self.paused {
            kc.set_colour(Vec4::new(1.0, 1.0, 1.0, 0.5));
            kc.set_depth(10.0);
            kc.rect(inputs.screen_rect);
        }

    }
}