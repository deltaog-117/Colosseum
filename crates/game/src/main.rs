//! Game executable: window, fixed-timestep loop and input sampling.

mod device_input;
mod game_loop;
mod window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    window::run()
}
