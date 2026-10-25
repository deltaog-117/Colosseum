//! Game executable: window, fixed-timestep loop and input sampling.

mod device_input;
mod game_loop;
mod hud;
mod level;
mod window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    window::run()
}
