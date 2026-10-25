//! The OS window and its event loop.

use std::time::Instant;

use sim::input::{Axis, Command, Event, InputState};
use sim::tick::TickRate;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::device_input;
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
    input: InputState,
    command: Command,
    grabbed: bool,
}

/// The part of a command that is visible in the title.
fn intent(command: &Command) -> (Axis, Axis, bool) {
    (command.forward(), command.strafe(), command.jump())
}

impl App {
    fn new() -> Self {
        let rate = TickRate::DEFAULT;
        Self {
            window: None,
            game_loop: GameLoop::new(rate, Instant::now()),
            rate,
            ticks: 0,
            input: InputState::new(),
            command: Command::default(),
            grabbed: false,
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
        let hz = u64::from(self.rate.hz());
        let seconds_before = self.ticks / hz;
        self.ticks += u64::from(advance.ticks);

        let intent_before = intent(&self.command);
        for _ in 0..advance.ticks {
            self.command = self.input.take_command();
        }

        // The title doubles as a heartbeat and input readout until there is
        // anything to render or move.
        if let Some(window) = &self.window
            && (self.ticks / hz != seconds_before || intent(&self.command) != intent_before)
        {
            let (forward, strafe, jump) = intent(&self.command);
            window.set_title(&format!(
                "{TITLE} - tick {} - fwd {forward:?} strafe {strafe:?} jump {jump}",
                self.ticks
            ));
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
            WindowEvent::KeyboardInput { event, .. } => self.on_key(&event),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } if !self.grabbed => self.grab_cursor(),
            WindowEvent::Focused(false) => {
                self.input.apply(Event::FocusLost);
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
