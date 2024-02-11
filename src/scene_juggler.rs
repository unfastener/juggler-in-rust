// scene.rs - A scene of the classic Amiga Juggler demo

use std::time::Duration;
use vecmath::{vec3_add, vec3_scale, vec3_sub, Vector3};

use crate::renderer::{
    Camera, Light, Scene, SceneOptions, Sphere,
    Texture::{CheckerXZ, Color, GradientY},
};

const SKY_COLOR: (f64, f64, f64) = (0.1, 0.1, 1.0); // Color when nothing hit

const BOUNCE_CYCLE_S: f64 = 1.0;
const CAMERA_CYCLE_S: f64 = 15.0;

pub fn populate_scene(
    scene: &mut Scene,
    duration_since_start: Duration,
    scene_options: &SceneOptions,
) {
    let secs = duration_since_start.as_secs_f64();

    let bounce_secs = secs * scene_options.speed_0;
    let bounce_phase = (bounce_secs % BOUNCE_CYCLE_S) / BOUNCE_CYCLE_S;
    let body_bounce = 0.15 * (bounce_phase * std::f64::consts::TAU).sin();
    let body_bounce_90 = 0.15 * (bounce_phase * std::f64::consts::TAU).cos();

    // Color when nothing hit
    scene.sky_color = SKY_COLOR;

    // Scene to render
    scene.spheres = vec![
        // Ground
        Sphere {
            pos: [0.0, -5000.0, 0.0],
            r: 5000.0,
            texture: CheckerXZ {
                color1: (1.0, 1.0, 0.0), // Yellow
                color2: (0.0, 1.0, 0.0), // Green
                scale: 4.0,
            }, // Yellow-magenta checkered texture, ground
            specular: -1.0,       // Dull, not shiny
            reflective: 0.0,      // Not reflective
            skip_lighting: false, // Regular lighting calculations
        },
        // Sky sphere
        Sphere {
            pos: [0.0, 0.0, 0.0],
            r: 10000.0,
            texture: GradientY {
                color1: (0.1, 0.1, 1.0), // Top: deep blue
                color2: (0.7, 0.7, 1.0), // Bottom: light blue
            }, // Blue sky
            specular: -1.0,      // Dull, not shiny
            reflective: 0.0,     // Not reflective
            skip_lighting: true, // Sky is always fully bright
        },
    ];

    // Juggling ball material
    let juggling_sphere = Sphere {
        pos: [0.0, 0.0, 0.0],          // Ignored
        r: 0.0,                        // Ignored
        texture: Color(0.9, 0.9, 0.9), // White
        specular: 100.0,               // Shiny
        reflective: 0.8,               // Very reflective
        skip_lighting: false,          // Regular lighting calculations
    };

    // Body material
    let body_sphere = Sphere {
        pos: [0.0, 0.0, 0.0],          // Ignored
        r: 0.0,                        // Ignored
        texture: Color(1.0, 0.1, 0.1), // Red
        specular: 100.0,               // Shiny
        reflective: 0.0,               // Not reflective
        skip_lighting: false,          // Regular lighting calculations
    };

    // "Extra" body material
    let extra_body_sphere = Sphere {
        pos: [0.0, 0.0, 0.0],          // Ignored
        r: 0.0,                        // Ignored
        texture: Color(1.0, 0.1, 0.1), // Red
        specular: 100.0,               // Shiny
        reflective: 0.3,               // A little reflective
        skip_lighting: false,          // Regular lighting calculations
    };

    // Limbs and face material
    let skin_sphere = Sphere {
        pos: [0.0, 0.0, 0.0],          // Ignored
        r: 0.0,                        // Ignored
        texture: Color(1.0, 0.7, 0.7), // Pink
        specular: 100.0,               // Shiny
        reflective: 0.0,               // Not reflective
        skip_lighting: false,          // Regular lighting calculations
    };

    // Hair material
    let hair_sphere = Sphere {
        pos: [0.0, 0.0, 0.0],          // Ignored
        r: 0.0,                        // Ignored
        texture: Color(0.2, 0.1, 0.1), // Very dark brown
        specular: 100.0,               // Shiny
        reflective: 0.0,               // Not reflective
        skip_lighting: false,          // Regular lighting calculations
    };

    // Eyes material
    let eye_sphere = Sphere {
        pos: [0.0, 0.0, 0.0],          // Ignored
        r: 0.0,                        // Ignored
        texture: Color(0.1, 0.1, 1.0), // Blue
        specular: 100.0,               // Shiny
        reflective: 0.0,               // Not reflective
        skip_lighting: false,          // Regular lighting calculations
    };

    // Head, face and neck spheres
    scene.spheres.push(make_sphere(
        &skin_sphere,
        [0.0, 6.1 + body_bounce, 0.2 + body_bounce_90],
        0.5,
    )); // Head
    scene.spheres.push(make_sphere(
        &hair_sphere,
        [0.0, 6.12 + body_bounce, 0.22 + body_bounce_90],
        0.5,
    )); // Hair
    scene.spheres.push(make_sphere(
        &skin_sphere,
        [0.0, 5.5 + body_bounce, 0.2 + body_bounce_90],
        0.2,
    )); // Neck
    scene.spheres.push(make_sphere(
        &eye_sphere,
        [-0.2, 6.1 + body_bounce, -0.2 + body_bounce_90],
        0.15,
    )); // Left eye
    scene.spheres.push(make_sphere(
        &eye_sphere,
        [0.2, 6.1 + body_bounce, -0.2 + body_bounce_90],
        0.15,
    )); // Right eye

    // Body spheres
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(
            &body_sphere,
            [0.0, 4.6 + body_bounce, 0.2 + body_bounce_90],
            0.8,
        ),
        &make_sphere(&body_sphere, [0.0, 3.3 + body_bounce, 0.0], 0.6),
        8,
        true,
    );

    if scene_options.option_1 == true {
        // Bite my shiny metal ...
        scene.spheres.push(make_sphere(
            &extra_body_sphere,
            [-0.2, 3.2 + body_bounce, 0.2],
            0.5,
        ));
        scene.spheres.push(make_sphere(
            &extra_body_sphere,
            [0.2, 3.2 + body_bounce, 0.2],
            0.5,
        ));
    }

    let left_hand = [-2.0, 3.1, -1.0];
    let right_hand = [1.9, 3.8, -1.0];

    // Left arm spheres
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(
            &skin_sphere,
            [-0.7, 5.1 + body_bounce, 0.2 + body_bounce_90],
            0.2,
        ),
        &make_sphere(
            &skin_sphere,
            [
                -1.2 + body_bounce / 1.4,
                4.2 + body_bounce,
                -0.2 + body_bounce_90,
            ],
            0.2,
        ),
        9,
        false,
    );
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(
            &skin_sphere,
            [
                -1.2 + body_bounce / 1.4,
                4.2 + body_bounce,
                -0.2 + body_bounce_90,
            ],
            0.2,
        ),
        &make_sphere(
            &skin_sphere,
            vec3_add(
                left_hand,
                [-body_bounce_90 * 1.5, body_bounce, 0.0 + body_bounce_90],
            ),
            0.1,
        ),
        8,
        true,
    );

    // Right arm spheres
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(
            &skin_sphere,
            [0.7, 5.1 + body_bounce, 0.2 + body_bounce_90],
            0.2,
        ),
        &make_sphere(
            &skin_sphere,
            [
                1.2 + body_bounce / 1.4,
                4.2 + body_bounce,
                -0.2 + body_bounce_90,
            ],
            0.2,
        ),
        9,
        false,
    );
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(
            &skin_sphere,
            [
                1.2 + body_bounce / 1.4,
                4.2 + body_bounce,
                -0.2 + body_bounce_90,
            ],
            0.2,
        ),
        &make_sphere(
            &skin_sphere,
            vec3_add(
                right_hand,
                [body_bounce_90 * 1.5, body_bounce, 0.0 + body_bounce_90],
            ),
            0.1,
        ),
        8,
        true,
    );

    // Left leg spheres
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(&skin_sphere, [-0.6, 2.9 + body_bounce, 0.0], 0.2),
        &make_sphere(
            &skin_sphere,
            [-0.7, 1.6 + body_bounce / 2.0, -0.6 + body_bounce / 1.4],
            0.2,
        ),
        8,
        false,
    );
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(
            &skin_sphere,
            [-0.7, 1.6 + body_bounce / 2.0, -0.6 + body_bounce / 1.4],
            0.2,
        ),
        &make_sphere(&skin_sphere, [-0.6, 0.0, 0.0], 0.1),
        8,
        true,
    );

    // Right leg spheres
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(&skin_sphere, [0.6, 2.9 + body_bounce, 0.0], 0.2),
        &make_sphere(
            &skin_sphere,
            [0.7, 1.6 + body_bounce / 2.0, -0.6 + body_bounce / 1.4],
            0.2,
        ),
        8,
        false,
    );
    line_of_spheres(
        &mut scene.spheres,
        &make_sphere(
            &skin_sphere,
            [0.7, 1.6 + body_bounce / 2.0, -0.6 + body_bounce / 1.4],
            0.2,
        ),
        &make_sphere(&skin_sphere, [0.6, 0.0, 0.0], 0.1),
        8,
        true,
    );

    // Juggling balls
    let diff_right_left = vec3_sub(right_hand, left_hand);

    // Ball 1: low arch
    let phase = bounce_phase;
    let mut pos = vec3_add(left_hand, vec3_scale(diff_right_left, phase));
    pos[1] += 2.1 * (phase * std::f64::consts::PI).sin() + 0.4;
    pos[2] -= 0.3;
    scene.spheres.push(make_sphere(&juggling_sphere, pos, 0.6));

    // Ball 2: first half (rising) of high arch
    let phase = bounce_phase / 2.0;
    let mut pos = vec3_add(right_hand, vec3_scale(diff_right_left, -phase));
    pos[1] += 4.2 * (phase * std::f64::consts::PI).sin() + 0.4;
    pos[2] -= 0.3;
    scene.spheres.push(make_sphere(&juggling_sphere, pos, 0.6));

    // Ball 3: second half (falling) of high arch
    let phase = bounce_phase / 2.0 + 0.5;
    let mut pos = vec3_add(right_hand, vec3_scale(diff_right_left, -phase));
    pos[1] += 4.2 * (phase * std::f64::consts::PI).sin() + 0.4;
    pos[2] -= 0.3;
    scene.spheres.push(make_sphere(&juggling_sphere, pos, 0.6));

    // Lights
    scene.lights = vec![
        Light::Ambient { intensity: 0.45 },
        Light::Point {
            intensity: 0.55,
            pos: [50.0, 150.0, -100.0],
        },
    ];

    // Camera
    let camera_distance = 10.0;
    let camera_secs = secs * scene_options.speed_1;
    let camera_phase = (camera_secs % CAMERA_CYCLE_S) / CAMERA_CYCLE_S;
    let camera_angle = camera_phase * std::f64::consts::TAU;
    // DEBUG: let camera_angle = std::f64::consts::TAU / 8.0;
    scene.camera = Camera {
        pos: [
            camera_distance * camera_angle.sin(),
            4.0,
            -camera_distance * camera_angle.cos(),
        ],
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        forward: [0.0, 0.0, 1.0],
    };
    scene.camera.look_at([0.0, 4.0, 0.0]);
}

fn make_sphere(prototype: &Sphere, pos: Vector3<f64>, r: f64) -> Sphere {
    let mut sphere = prototype.clone();
    sphere.pos = pos;
    sphere.r = r;
    sphere
}

fn line_of_spheres(
    spheres: &mut Vec<Sphere>,
    start: &Sphere,
    end: &Sphere,
    num_spheres: usize,
    inclusive: bool,
) {
    let dir = vec3_sub(end.pos, start.pos);
    let last_pos = num_spheres - 1;
    for n in 0..=last_pos {
        if !inclusive && n == last_pos {
            break;
        }

        // Position and radius will be interpolated, other
        // attributes are from the first sphere
        let mut new_sphere = start.clone();
        let scale = n as f64 / last_pos as f64;
        new_sphere.pos = vec3_add(start.pos, vec3_scale(dir, scale));
        new_sphere.r = start.r + (end.r - start.r) * scale;
        spheres.push(new_sphere);
    }
}

// robots.dat from the "Ray Tracer 1.0" disk
// -----------------------------------------
//
// (-10,-4,5.5)
// [-10,20]
// 35
//
//
//   <.9,.9,.9> 2  (-0.9,-2.1,5.3):0.6;
//   <.9,.9,.9> 2  (-1.1,1.9,5.9):0.6;
//   <.9,.9,.9> 2  (-0.4,-1.2,6.8):0.6;
//   <1,.7,.7>  1  (0,0,6.1):0.5;
//   <.2,.1,.1> 1  (0.02,0,6.12):0.5;
//   <.1,.1,1.> 1  (-0.4,0.2,6.1):0.15;
//   <.1,.1,1.> 1  (-0.4,-0.2,6.1):0.15;
//   <1,.7,.7>  1  (0,0,5.5):0.2;
//   <1,.1,.1>  1  (0,0,4.6):0.8 5 (0,0,3.3):0.6;
//   <1,.7,.7>  1  (0,0.6,2.9):0.2 6 (-0.6,0.6,1.6):0.2
//               7 (-0.4,0.6,0):0.1;
//   <1,.7,.7>  1  (0,-0.6,2.9):0.2 6 (0.2,-0.6,1.6):0.2
//               7 (0.4,-0.6,0):0.1;
//   <1,.7,.7>  1  (0,-0.7,5.1):0.2 6 (-0.2,-1.2,4.2):0.2
//               7 (-1.1,-2.0,4.1):0.1;
//   <1,.7,.7>  1  (0,0.7,5.1):0.2 6 (-0.2,1.2,4.2):0.2
//               7 (-1.0,1.9,4.8):0.1;;
//
// 1
//    (-100,50,150):15 <1,1,1>
//
// <1.5,1.5,0>  <0,1.5,0> <.25,.25,.25> <.1,.1,1> <.7,.7,1>
