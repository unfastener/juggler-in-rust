// scene.rs - A simple scene with four spheres and three lights

use std::time::Duration;

use crate::renderer::{
    Camera, Light, Scene, SceneOptions, Sphere, Texture::CheckerXZ, Texture::Color,
};

const SKY_COLOR: (f64, f64, f64) = (0.15, 0.25, 0.35); // Color when nothing hit

pub fn populate_scene(
    scene: &mut Scene,
    duration_since_start: Duration,
    scene_options: &SceneOptions,
) {
    let secs = duration_since_start.as_secs_f64();

    // Color when nothing hit
    scene.sky_color = SKY_COLOR;

    // Scene to render
    scene.spheres = vec![
        Sphere {
            pos: [0.0, -1.0, -1.0],
            r: 1.0,
            texture: Color(1.0, 0.0, 0.0), // Red
            specular: 500.0,               // Shiny
            reflective: 0.2,               // A bit reflective
            skip_lighting: false,          // Regular lighting calculations
        },
        Sphere {
            pos: [2.0, 0.0, 0.0],
            r: 1.0,
            texture: Color(0.0, 0.0, 1.0), // Blue
            specular: 500.0,               // Shiny
            reflective: 0.3,               // A bit more reflective
            skip_lighting: false,          // Regular lighting calculations
        },
        Sphere {
            pos: [-2.0, 0.0, 0.0],
            r: 1.0,
            texture: Color(0.0, 1.0, 0.0), // Green
            specular: 10.0,                // Somewhat shiny
            reflective: 0.4,               // Even more reflective
            skip_lighting: false,          // Regular lighting calculations
        },
        Sphere {
            pos: [0.0, -5001.0, 0.0],
            r: 5000.0,
            texture: CheckerXZ {
                color1: (1.0, 1.0, 0.0),
                color2: (1.0, 0.0, 1.0),
                scale: 1.0,
            }, // Yellow-magenta checkered texture, ground
            specular: 1000.0,     // Very shiny
            reflective: 0.5,      // Half reflective
            skip_lighting: false, // Regular lighting calculations
        },
    ];

    // Lights
    scene.lights = vec![
        Light::Ambient { intensity: 0.2 },
        Light::Point {
            intensity: 0.6,
            pos: [2.0, 1.0, -4.0],
        },
        Light::Directional {
            intensity: 0.2,
            dir: [1.0, 4.0, 0.0],
        },
    ];

    // Camera
    const CAMERA_CYCLE_S: f64 = 15.0;
    let camera_distance = 5.0;
    let camera_secs = secs * scene_options.speed_1;
    let camera_phase = (camera_secs % CAMERA_CYCLE_S) / CAMERA_CYCLE_S;
    let camera_angle = camera_phase * std::f64::consts::TAU;
    // DEBUG: let camera_angle = std::f64::consts::TAU / 8.0;
    scene.camera = Camera {
        pos: [
            camera_distance * camera_angle.sin(),
            0.0,
            -camera_distance * camera_angle.cos(),
        ],
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        forward: [0.0, 0.0, 1.0],
    };
    scene.camera.look_at([0.0, 0.0, 0.0]);
}
