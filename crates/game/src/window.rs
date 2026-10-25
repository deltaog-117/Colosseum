//! The OS window and its event loop.

use std::time::Instant;

use sim::tick::TickRate;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::game_loop::GameLoop;

const TITLE: &str = "fps-3d-game";

pub(crate) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    // Poll: the game loop paces itself, so the OS must not park the thread
    // between frames.
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::new())?;
    Ok(())
}

struct App {
    window: Option<Window>,
    game_loop: GameLoop,
    rate: TickRate,
    ticks: u64,
}

impl App {
    fn new() -> Self {
        let rate = TickRate::DEFAULT;
        Self {
            window: None,
            game_loop: GameLoop::new(rate, Instant::now()),
            rate,
            ticks: 0,
        }
    }

    fn run_frame(&mut self) {
        let advance = self.game_loop.frame(Instant::now());
        let hz = u64::from(self.rate.hz());
        let seconds_before = self.ticks / hz;
        self.ticks += u64::from(advance.ticks);

        // The title doubles as a heartbeat until there is anything to render.
        if let Some(window) = &self.window
            && self.ticks / hz != seconds_before
        {
            window.set_title(&format!("{TITLE} - tick {}", self.ticks));
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        match event_loop.create_window(Window::default_attributes().with_title(TITLE)) {
            Ok(window) => self.window = Some(window),
            Err(err) => {
                eprintln!("could not create the window: {err}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.run_frame(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
