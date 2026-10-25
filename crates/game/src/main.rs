//! Game executable: window, fixed-timestep loop and input sampling.

mod game_loop;
mod window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    window::run()
}
