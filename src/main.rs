// juggler-in-rust - Drawing a simple raytraced scene in a resizable window
// v0.2.0 2024-02-23

mod renderer;
mod scene_juggler;
mod window;

use std::fs::File;
use std::time::Duration;
use std::{io::Write, sync::Arc};

use renderer::{Renderer, SceneOptions};

const WINDOW_TITLE: &str = "Juggler in Rust"; // Window title
const TARGET_FPS: f64 = 24.0; // The best framerate, agreed by the world
const FPS_TEST_ROUNDS: usize = 3; // Test render three times per resolution

fn main() {
    // Create a raytracing renderer
    let renderer = renderer::Renderer::new();

    let to_files = false;
    if to_files {
        // Render to files instead of displaying on-screen
        render_to_files(&renderer);
    } else {
        // Select render size according to the desired frame rate
        find_optimal_render_size(&renderer);

        // Create a window
        let mut window = window::Window::new(&renderer);

        window.set_title(&format!("{WINDOW_TITLE}"));

        // Run event loop
        window.run();

        // This part is not reached on all platforms
    }
}

fn find_optimal_render_size(renderer: &Arc<Renderer>) {
    let try_sizes = [
        80, 128, 160, 200, 256, 320, 400, 480, 512, 640, 720, 800, 960, 1024, 1280,
    ];

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

fn render_to_files(renderer: &Arc<Renderer>) {
    // Render to files in a high resolution
    let size = 720;
    renderer.set_size((size, size));

    // Default scene options
    let scene_options = SceneOptions {
        speed_0: 1.0,
        speed_1: 1.0,
        option_0: false,
        option_1: false,
    };

    let ppm_header = format!("P6\n{size} {size}\n255\n");

    let fps = TARGET_FPS;
    let num_frames = (fps * 15.0) as usize; // 15 seconds

    for frame in 0..num_frames {
        let secs = frame as f64 / fps;
        println!("Frame {frame} @ {secs:.3} s");

        // Render image
        let duration = Duration::from_secs_f64(secs);
        renderer.start_render(duration, &scene_options);
        renderer.wait_for_completion(false);

        // Write image to a Portable Pixmap (PPM) file
        {
            let filename = format!("img{:03}.ppm", frame);
            let mut file = File::create(filename).unwrap();

            // Write header
            file.write_all(ppm_header.as_bytes()).unwrap();

            // Write pixel data
            let render_buffer = renderer.get_buffer();
            let buffer = render_buffer.lock().unwrap();
            for offset in 0..(size * size) {
                let pixel = buffer[offset];
                let pixel_conv = [
                    (pixel >> 16 & 0xff) as u8, // R
                    (pixel >> 8 & 0xff) as u8,  // G
                    (pixel >> 0 & 0xff) as u8,  // B
                ];
                file.write_all(&pixel_conv).unwrap();
            }
        }
    }
}
