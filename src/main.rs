// juggler-in-rust - Drawing a simple raytraced scene in a resizable window
// v0.1.0 2024-02-18

mod renderer;
mod scene_juggler;
mod window;

use std::sync::Arc;
use std::time::Duration;

use renderer::{Renderer, SceneOptions};

const WINDOW_TITLE: &str = "Juggler in Rust"; // Window title
const TARGET_FPS: f64 = 24.0; // The best framerate, agreed by the world
const FPS_TEST_ROUNDS: usize = 3; // Test render three times per resolution

fn main() {
    // Create a raytracing renderer
    let renderer = renderer::Renderer::new();

    // Select render size according to the desired frame rate
    find_optimal_render_size(&renderer);

    // Create a window
    let mut window = window::Window::new(&renderer);

    window.set_title(&format!("{WINDOW_TITLE}"));

    // Run event loop
    window.run();

    // This part is not reached on all platforms
}

fn find_optimal_render_size(renderer: &Arc<Renderer>) {
    let try_sizes = [80, 128, 160, 256, 320, 512, 640, 1024, 1280];

    // Default scene options
    let scene_options = SceneOptions {
        speed_0: 1.0,
        speed_1: 1.0,
        option_0: false,
        option_1: false,
    };

    for n in 1..try_sizes.len() {
        let size = try_sizes[n];

        // Do a few test renders with the given size
        renderer.set_size((size, size));

        let mut total_duration = Duration::ZERO;
        for _ in 0..FPS_TEST_ROUNDS {
            renderer.start_render(Duration::ZERO, &scene_options);
            renderer.wait_for_completion(false);
            total_duration += renderer.get_duration();
        }

        // Calculate FPS
        let fps = 1.0 / (total_duration.as_secs_f64() / FPS_TEST_ROUNDS as f64);
        if fps < TARGET_FPS {
            // FPS is lower than target, use the previous size
            let size = try_sizes[n - 1];
            renderer.set_size((size, size));
            break;
        }
    }
}
