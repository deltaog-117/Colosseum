//! The OS window and its event loop.

use std::sync::Arc;
use std::time::Instant;

use render::Renderer;
use render::camera::{Camera, Projection};
use render::hud::HudBatch;
use render::mesh::Mesh;
use render::resolution::RenderResolution;

use glam::Vec3;
use sim::bot::{Bot, Target};
use sim::collision::Aabb;
use sim::combat::{Health, Outcome, strike};
use sim::input::{Command, Event, InputState};
use sim::movement::{Player, Pose};
use sim::rules::{Pickup, PickupKind, Round, Transition};
use sim::tick::TickRate;
use sim::weapon::Launcher;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::device_input;
use crate::game_loop::GameLoop;
use crate::hud::{self, HudView, ScoreRow};
use crate::level::{self, Level};

const TITLE: &str = "fps-3d-game";

/// Opponents in the arena. Placeholder until there is a menu to set it.
const BOT_COUNT: usize = 3;
/// Spreads the bots' seeds apart; the golden-ratio constant of Knuth's
/// multiplicative hash, so neighbouring bots do not strafe in step.
const BOT_SEED_STRIDE: u32 = 0x9E37_79B9;
const BOT_COLOR: [f32; 3] = [0.85, 0.2, 0.2];

/// Half the edge of the cube that stands in for a rocket, until there is art.
const ROCKET_HALF_SIZE: f32 = 0.12;
const ROCKET_COLOR: [f32; 3] = [1.0, 0.45, 0.1];
const HEALTH_PICKUP_COLOR: [f32; 3] = [0.2, 0.9, 0.3];
const AMMO_PICKUP_COLOR: [f32; 3] = [0.95, 0.8, 0.2];

pub(crate) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    // Poll: the game loop paces itself, so the OS must not park the thread
    // between frames.
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::new()?)?;
    Ok(())
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    camera: Camera,
    level: Level,
    player: Player,
    launcher: Launcher,
    health: Health,
    round: Round,
    pickups: Vec<Pickup>,
    bots: Vec<Bot>,
    /// Where the player last fell, so the respawn can be far from it.
    death_at: Vec3,
    /// The player's pose one tick ago, the start of the interpolation.
    previous: Pose,
    game_loop: GameLoop,
    rate: TickRate,
    input: InputState,
    command: Command,
    grabbed: bool,
    /// The scoreboard key is held. Interface state, not a game command, so
    /// it never reaches the simulation.
    show_scoreboard: bool,
    /// Rebuilt every frame; kept so its memory is reused.
    hud: HudBatch,
}

/// One cube per pickup that is ready to be taken, standing on its base.
fn pickup_meshes(pickups: &[Pickup]) -> Vec<Mesh> {
    pickups
        .iter()
        .filter(|pickup| pickup.is_ready())
        .filter_map(|pickup| {
            let color = match pickup.kind() {
                PickupKind::Health => HEALTH_PICKUP_COLOR,
                PickupKind::Ammo => AMMO_PICKUP_COLOR,
            };
            let bounds = pickup.bounds();
            Mesh::cuboid(bounds.min(), bounds.max(), color)
        })
        .collect()
}

/// One box per living bot, until there is art.
fn bot_meshes(bots: &[Bot]) -> Vec<Mesh> {
    bots.iter()
        .filter_map(|bot| bot.active_body(true))
        .filter_map(|body| Mesh::cuboid(body.min(), body.max(), BOT_COLOR))
        .collect()
}

/// One small cube per rocket, placed between its last two ticks. A rocket
/// whose cube cannot be built is skipped rather than drawn wrong.
fn rocket_meshes(launcher: &Launcher, alpha: f32) -> Vec<Mesh> {
    launcher
        .projectiles()
        .iter()
        .filter_map(|rocket| {
            let center = rocket.position_at(alpha);
            let half = Vec3::splat(ROCKET_HALF_SIZE);
            Mesh::cuboid(center - half, center + half, ROCKET_COLOR)
        })
        .collect()
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let rate = TickRate::DEFAULT;
        let projection = Projection::try_new(90f32.to_radians(), 0.05, 200.0)?;
        let level = level::arena()?;
        let first_spawn = level.spawns().first();
        let player = Player::try_spawn(first_spawn, level.world(), rate)?;
        let pickups = level
            .pickups()
            .iter()
            .filter_map(|(kind, base)| Pickup::try_new(*kind, *base, rate))
            .collect();
        let pose = player.pose().ok_or("spawn pose is not finite")?;
        let camera = Camera::new(pose.eye(), projection).ok_or("spawn point is not finite")?;
        // The player holds the first spawn point, so the bots take the others in turn.
        let spawns = level.spawns().points();
        let bots = (0..BOT_COUNT)
            .filter_map(|index| {
                let home = spawns[(index + 1) % spawns.len()];
                let seed = BOT_SEED_STRIDE.wrapping_mul(index as u32 + 1);
                Bot::try_spawn(home, level.world(), rate, seed)
                    .map_err(|err| eprintln!("could not place bot {}: {err}", index + 1))
                    .ok()
            })
            .collect();
        Ok(Self {
            window: None,
            renderer: None,
            camera,
            level,
            player,
            launcher: Launcher::new(rate),
            health: Health::new(rate),
            round: Round::new(rate),
            pickups,
            bots,
            death_at: first_spawn,
            previous: pose,
            game_loop: GameLoop::new(rate, Instant::now()),
            rate,
            input: InputState::new(),
            command: Command::default(),
            grabbed: false,
            show_scoreboard: false,
            hud: HudBatch::new(),
        })
    }

    /// Applies this tick's blasts to everyone. A shooter takes only a share
    /// of the damage from their own rocket, and a kill goes to the shooter
    /// unless they killed themselves. Nothing lands once the round is over,
    /// so the final score cannot change after the bell.
    fn apply_blasts(&mut self) {
        if !self.round.is_playing() {
            return;
        }
        let world = self.level.world();

        // Copied out so the launcher's list is not borrowed while bodies change.
        let blasts = self.launcher.detonations().to_vec();
        for blast in &blasts {
            if strike(blast, world, &mut self.player, &mut self.health, true) == Outcome::Killed {
                self.round.record_death();
                self.death_at = self.player.position();
            }
            for bot in &mut self.bots {
                if bot.take_blast(blast, world, false) == Outcome::Killed {
                    self.round.record_kill();
                }
            }
        }

        for shooter in 0..self.bots.len() {
            let blasts = self.bots[shooter].launcher().detonations().to_vec();
            for blast in &blasts {
                if strike(blast, world, &mut self.player, &mut self.health, false)
                    == Outcome::Killed
                {
                    self.round.record_death();
                    self.death_at = self.player.position();
                    self.bots[shooter].record_kill();
                }
                for victim in 0..self.bots.len() {
                    let outcome = self.bots[victim].take_blast(blast, world, victim == shooter);
                    if outcome == Outcome::Killed && victim != shooter {
                        self.bots[shooter].record_kill();
                    }
                }
            }
        }
    }

    /// What the bots aim at: the player while they live.
    fn player_target(&self) -> Option<Target> {
        self.health
            .is_alive()
            .then(|| Target::try_new(self.player.bounds().center(), self.player.velocity()))
            .flatten()
    }

    /// Moves and arms every bot for one tick. Each fights the nearest living
    /// body but its own, and its rockets burst on every body but its own.
    fn step_bots(&mut self) {
        let playing = self.round.is_playing();
        // Index 0 is the player, then the bots in order; the dead are `None`.
        let bodies: Vec<Option<Aabb>> =
            std::iter::once(self.health.is_alive().then(|| *self.player.bounds()))
                .chain(self.bots.iter().map(|bot| bot.active_body(true)))
                .collect();
        let marks: Vec<Option<Target>> = std::iter::once(self.player_target())
            .chain(self.bots.iter().map(Bot::as_target))
            .collect();

        for (index, bot) in self.bots.iter_mut().enumerate() {
            let me = index + 1;
            let target = Target::nearest(
                bot.player().position(),
                marks
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != me)
                    .filter_map(|(_, mark)| mark.as_ref()),
            );
            let enemies: Vec<Aabb> = bodies
                .iter()
                .enumerate()
                .filter(|(other, _)| *other != me)
                .filter_map(|(_, body)| *body)
                .collect();
            bot.step(self.level.world(), playing, target, &enemies);
        }
    }

    /// Brings back the bots whose wait is over.
    fn respawn_bots(&mut self) {
        for bot in &mut self.bots {
            if let Err(err) = bot.step_health(self.level.spawns(), self.level.world()) {
                eprintln!("could not respawn a bot: {err}");
            }
        }
    }

    /// Puts the player at the spawn point farthest from where they fell,
    /// with a full launcher. If the spawn is somehow blocked the player
    /// stays where they fell.
    fn respawn(&mut self) {
        let spawn = self.level.spawns().farthest_from(self.death_at);
        match Player::try_spawn(spawn, self.level.world(), self.rate) {
            Ok(player) => {
                self.player = player;
                self.launcher = Launcher::new(self.rate);
            }
            Err(err) => eprintln!("could not respawn: {err}"),
        }
        if let Some(pose) = self.player.pose() {
            self.previous = pose;
        }
    }

    /// Starts the next round: everyone back at the first spawn, healed,
    /// reloaded, with every pickup standing again.
    fn restart_round(&mut self) {
        self.health = Health::new(self.rate);
        self.death_at = self.level.spawns().first();
        self.respawn();
        for bot in &mut self.bots {
            if let Err(err) = bot.reset(self.level.world()) {
                eprintln!("could not reset a bot: {err}");
            }
        }
        self.pickups.iter_mut().for_each(Pickup::reset);
    }

    /// Hands pickups out for one tick, to the player first and then to the
    /// bots in order. The dead and the finished take nothing, but the
    /// pickups' waits still run.
    fn collect_pickups(&mut self) {
        let playing = self.round.is_playing();
        let body = (self.health.is_alive() && playing).then(|| *self.player.bounds());
        for pickup in &mut self.pickups {
            let taken = pickup.step(body.as_ref(), |kind| {
                kind.is_wanted(&self.health, &self.launcher)
            });
            if let Some(kind) = taken {
                kind.give(&mut self.health, &mut self.launcher);
                continue;
            }
            for bot in &mut self.bots {
                // The wait counts down on every `step`, so a pickup still
                // waiting is not offered on: that would run its clock fast.
                if !pickup.is_ready() {
                    break;
                }
                if let Some(kind) =
                    pickup.step(bot.active_body(playing).as_ref(), |kind| bot.wants(kind))
                {
                    bot.receive(kind);
                }
            }
        }
    }

    fn grab_cursor(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        // Locked is missing on some platforms (X11), where Confined still works.
        let grab = window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
        match grab {
            Ok(()) => {
                window.set_cursor_visible(false);
                self.grabbed = true;
            }
            Err(err) => eprintln!("could not grab the cursor: {err}"),
        }
    }

    fn release_cursor(&mut self) {
        self.grabbed = false;
        if let Some(window) = &self.window {
            if let Err(err) = window.set_cursor_grab(CursorGrabMode::None) {
                eprintln!("could not release the cursor: {err}");
            }
            window.set_cursor_visible(true);
        }
    }

    fn on_key(&mut self, event: &KeyEvent) {
        if event.state == ElementState::Pressed
            && event.physical_key == PhysicalKey::Code(KeyCode::Escape)
        {
            self.release_cursor();
        }
        if event.physical_key == PhysicalKey::Code(KeyCode::Tab) {
            self.show_scoreboard = event.state == ElementState::Pressed;
        }
        // Held keys are already tracked; repeats add nothing.
        if event.repeat {
            return;
        }
        if let Some(input) = device_input::key_event(event.physical_key, event.state) {
            self.input.apply(input);
        }
    }

    fn run_frame(&mut self) {
        let advance = self.game_loop.frame(Instant::now());

        for _ in 0..advance.ticks {
            self.command = self.input.take_command();
            let transition = self.round.step();
            if transition == Some(Transition::Restarted) {
                self.restart_round();
            }
            if let Some(pose) = self.player.pose() {
                self.previous = pose;
            }
            let playing = self.health.is_alive() && self.round.is_playing();
            if playing {
                self.player.step(&self.command, self.level.world());
            }
            // Rockets burst on living bots as well as on walls.
            let bot_bodies: Vec<Aabb> = self
                .bots
                .iter()
                .filter_map(|bot| bot.active_body(true))
                .collect();
            self.launcher.step_against(self.level.world(), &bot_bodies);
            self.step_bots();
            self.apply_blasts();
            self.collect_pickups();
            if playing
                && self.health.is_alive()
                && self.command.fire()
                && let Some(aim) = self.player.pose()
            {
                // A denied shot (cooling down, empty) is normal play; the
                // reason will drive a click sound later.
                let _denied = self.launcher.try_fire(&aim);
            }
            if self.health.step() {
                self.respawn();
            }
            self.respawn_bots();
        }

        // Drawn between the last two ticks, so motion stays smooth when the
        // frame rate and the tick rate do not line up.
        if let Some(current) = self.player.pose() {
            let view = self.previous.lerp(&current, advance.alpha);
            self.camera.set_view(view.eye(), view.yaw(), view.pitch());
        }

        let score = self.round.score();
        let rows: Vec<ScoreRow> = self
            .bots
            .iter()
            .map(|bot| ScoreRow {
                kills: bot.score().kills(),
                deaths: bot.score().deaths(),
                frags: bot.score().frags(),
            })
            .collect();
        hud::draw(
            &mut self.hud,
            &HudView {
                health: self.health.points(),
                ammo: self.launcher.ammo(),
                seconds_left: self.round.seconds_left(),
                phase: self.round.phase(),
                kills: score.kills(),
                deaths: score.deaths(),
                frags: score.frags(),
                scoreboard: self.show_scoreboard,
                bots: &rows,
            },
        );

        if let Some(renderer) = &mut self.renderer {
            let mut transient = rocket_meshes(&self.launcher, advance.alpha);
            for bot in &self.bots {
                transient.extend(rocket_meshes(bot.launcher(), advance.alpha));
            }
            transient.extend(bot_meshes(&self.bots));
            transient.extend(pickup_meshes(&self.pickups));
            renderer.set_transient_meshes(&transient);
            renderer.set_hud(&self.hud);
            renderer.render(&self.camera);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = match event_loop.create_window(Window::default_attributes().with_title(TITLE))
        {
            Ok(window) => Arc::new(window),
            Err(err) => {
                eprintln!("could not create the window: {err}");
                event_loop.exit();
                return;
            }
        };
        let size = window.inner_size();
        match Renderer::new(
            Arc::clone(&window),
            (size.width, size.height),
            RenderResolution::DEFAULT,
        ) {
            Ok(mut renderer) => {
                for mesh in self.level.meshes() {
                    renderer.add_mesh(mesh);
                }
                self.renderer = Some(renderer);
            }
            Err(err) => {
                eprintln!("could not start the renderer: {err}");
                event_loop.exit();
                return;
            }
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.on_key(&event),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } if !self.grabbed => self.grab_cursor(),
            WindowEvent::MouseInput { button, state, .. } => {
                if let Some(input) = device_input::mouse_button(button, state) {
                    self.input.apply(input);
                }
            }
            WindowEvent::Focused(false) => {
                self.input.apply(Event::FocusLost);
                // The release would go to another window.
                self.show_scoreboard = false;
                self.release_cursor();
            }
            WindowEvent::RedrawRequested => self.run_frame(),
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        // Raw motion arrives even when the cursor is free; only a grabbed
        // cursor means the player is steering.
        if self.grabbed
            && let DeviceEvent::MouseMotion { delta } = event
        {
            self.input.apply(device_input::mouse_motion(delta));
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_sim_and_the_camera_agree_on_the_pitch_limit() {
        assert_eq!(
            sim::movement::tuning::PITCH_LIMIT,
            render::camera::PITCH_LIMIT
        );
    }
}
