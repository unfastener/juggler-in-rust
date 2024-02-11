// renderer.rs - A simple raytracing renderer

use core::option::Option;
use num_cpus;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use vecmath::{vec3_add, vec3_cross, vec3_dot, vec3_len, vec3_normalized, vec3_scale, vec3_sub, Vector3};

use crate::scene_juggler::populate_scene;

const DEFAULT_COLOR: (f64, f64, f64) = (0.5, 0.5, 0.5); // Window color at start

const RENDER_SPAN: usize = 64; // Number of pixels to render in one go
const RENDER_EPSILON: f64 = 0.0001; // Small distance away from a surface

#[derive(Clone)]
pub struct Camera {
    pub pos: Vector3<f64>,
    pub right: Vector3<f64>,
    pub up: Vector3<f64>,
    pub forward: Vector3<f64>,
}

impl Camera {
    pub fn look_at(self: &mut Self, look_at: Vector3<f64>) {
        self.forward = vec3_normalized(vec3_sub(look_at, self.pos));
        self.right = vec3_normalized(vec3_cross(self.up, self.forward));
        self.up = vec3_normalized(vec3_cross(self.forward, self.right));
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Texture {
    Color(f64, f64, f64),
    CheckerXZ {
        color1: (f64, f64, f64),
        color2: (f64, f64, f64),
        scale: f64,
    },
    GradientY {
        color1: (f64, f64, f64),
        color2: (f64, f64, f64),
    },
}

#[derive(Clone)]
pub struct Sphere {
    pub pos: Vector3<f64>,
    pub r: f64,
    pub texture: Texture,
    pub specular: f64,
    pub reflective: f64,
    pub skip_lighting: bool,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Light {
    Ambient { intensity: f64 },
    Point { intensity: f64, pos: Vector3<f64> },
    Directional { intensity: f64, dir: Vector3<f64> },
}

#[derive(Clone)]
pub struct Scene {
    pub camera: Camera,
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
    pub sky_color: (f64, f64, f64),
}

pub struct SceneOptions {
    pub speed_0: f64, // Scene decides what these mean
    pub speed_1: f64,
    pub option_0: bool,
    pub option_1: bool,
}

// Public name for the shared Renderer type
pub type SharedRenderer = Arc<Renderer>;

// Shared render buffer wrapped in Arc and Mutex
pub type SharedBuffer = Arc<Mutex<Vec<u32>>>;

// Shared private data wrapped in Arc and Mutex
struct SharedData {
    width: usize,
    height: usize,
    scene: Scene,
    buffer_0_active: bool, // true: Rendering to buffer_0, false: buffer_1
    next_pixel: usize,
    num_pixels: usize,
    start_time: Instant,
    duration: Duration,
    threads: Vec<JoinHandle<()>>,
}

pub struct Renderer {
    buffer_0: SharedBuffer,
    buffer_1: SharedBuffer,
    data: Arc<Mutex<SharedData>>,
    completion_callback: Arc<Mutex<Box<dyn Fn(Duration) -> () + Send + 'static>>>,
}

impl Renderer {
    pub fn new() -> SharedRenderer {
        // Create two shared render buffers, wrapped in Arc and Mutex
        let buffer_0: SharedBuffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_1: SharedBuffer = Arc::new(Mutex::new(Vec::new()));

        // Empty callback closure on heap
        let empty_callback: Box<dyn Fn(Duration) + Send + 'static> = Box::new(|_| {});

        // Create data fields, wrapped in Arc and Mutex
        let data = Arc::new(Mutex::new(SharedData {
            width: 0,
            height: 0,
            // Dummy defaults, set later
            scene: Scene {
                camera: Camera {
                    pos: [0.0, 0.0, 0.0],
                    right: [1.0, 0.0, 0.0],
                    up: [0.0, 1.0, 0.0],
                    forward: [0.0, 0.0, 1.0],
                },
                spheres: Vec::new(),
                lights: Vec::new(),
                sky_color: (0.0, 0.0, 0.0),
            },
            buffer_0_active: true,
            next_pixel: 0,
            num_pixels: 0,
            start_time: Instant::now(),
            duration: Duration::ZERO,
            threads: vec![],
        }));

        Arc::new(Renderer {
            buffer_0,
            buffer_1,
            data,
            completion_callback: Arc::new(Mutex::new(empty_callback)),
        })
    }

    pub fn get_buffer(self: &SharedRenderer) -> SharedBuffer {
        // Get access to shared variables
        let data = self.data.lock().unwrap();
        if data.buffer_0_active {
            // Currently rendering to buffer_0
            Arc::clone(&self.buffer_1)
        } else {
            // Currently rendering to buffer_1
            Arc::clone(&self.buffer_0)
        }
    }

    pub fn get_size(self: &SharedRenderer) -> (usize, usize) {
        let data = self.data.lock().unwrap();
        (data.width, data.height)
    }

    pub fn set_size(self: &SharedRenderer, size: (usize, usize)) {
        // Only square renders supported for now
        let (width, mut height) = size;
        if height > width {
            height = width;
        }

        // Get access to shared variables
        let mut data = self.data.lock().unwrap();
        let mut buffer_0 = self.buffer_0.lock().unwrap();
        let mut buffer_1 = self.buffer_1.lock().unwrap();

        // Set variables
        data.width = width;
        data.height = height;
        data.num_pixels = width * height;
        data.next_pixel = data.num_pixels; // End threads quickly

        // Resize buffers and clear them to a default color
        buffer_0.clear();
        buffer_0.resize(data.num_pixels, color_to_u32(DEFAULT_COLOR));
        buffer_1.clear();
        buffer_1.resize(data.num_pixels, color_to_u32(DEFAULT_COLOR));
    }

    pub fn set_completion_callback<F>(self: &SharedRenderer, callback: F)
    where
        F: Fn(Duration) + Send + 'static,
    {
        let mut completion_callback = self.completion_callback.lock().unwrap();
        *completion_callback = Box::new(callback);
    }

    pub fn start_render(
        self: &SharedRenderer,
        duration_since_start: Duration,
        scene_options: &SceneOptions,
    ) {
        // First, wait for all threads to end
        self.wait_for_completion(true);

        let mut data = self.data.lock().unwrap();

        data.next_pixel = 0; // Start over
        data.start_time = Instant::now(); // Record start of render
        data.duration = Duration::ZERO;

        // Get a scene to render
        populate_scene(&mut data.scene, duration_since_start, scene_options);

        // Start as many render threads as there are logical CPUs
        for _ in 0..num_cpus::get() {
            let thread_self: SharedRenderer = Arc::clone(&self);
            data.threads.push(thread::spawn(move || {
                thread_self.thread_func();
            }));
        }
    }

    pub fn wait_for_completion(self: &SharedRenderer, flush: bool) {
        // Read/write shared data
        let mut data = self.data.lock().unwrap();

        if flush {
            // Starting over, end threads quickly
            data.next_pixel = data.num_pixels;
        }

        // Atomically copy and clear thread IDs
        let threads = std::mem::take(&mut data.threads);

        // Release lock
        drop(data);

        // Wait for all threads to be completed
        for thread in threads {
            thread.join().unwrap();
        }
    }

    pub fn get_duration(self: &SharedRenderer) -> Duration {
        let data = self.data.lock().unwrap();
        return data.duration;
    }

    fn thread_func(self: SharedRenderer) {
        let mut span_buffer = vec![0x0000_0000; RENDER_SPAN];
        let (width, height);
        let scene;
        let buffer_0_active;

        {
            // Read shared data
            let data = self.data.lock().unwrap();

            // Get render buffer width and height
            (width, height) = (data.width, data.height);

            // Get thread local copies of scene elements (Camera, Spheres, Lights)
            scene = data.scene.clone();

            // Get currently active buffer (i.e., the buffer to render)
            buffer_0_active = data.buffer_0_active;
        }

        let mut done = false;
        while !done {
            let pixel: usize; // Next buffer index to render

            {
                // Read/write shared data
                let mut data = self.data.lock().unwrap();

                // Get next pixel to render
                pixel = data.next_pixel;
                if pixel >= data.num_pixels {
                    // All done, exit thread
                    break;
                }

                // Update next pixel
                data.next_pixel += RENDER_SPAN;
                if data.next_pixel >= data.num_pixels {
                    // When this last render span is finished, call completion callback
                    data.next_pixel = data.num_pixels;
                    done = true;
                }
            }

            // TODO: Last span may be short. Currently, num_pixels must be
            // divisible by RENDER_SPAN, otherwise there is an overflow

            // Render a span of pixels
            for n in 0..RENDER_SPAN {
                // Get pixel coordinates x and y
                let x = (pixel + n) % width;
                let y = (pixel + n) / width;

                // Scale x and y to viewport coordinates
                let vx = (x as f64 / (width - 1) as f64) - 0.5;
                let vy = 0.5 - (y as f64 / (height - 1) as f64);

                // Set up camera and viewport for shooting rays
                let ray_origin = scene.camera.pos;
                let ray_dir = vec3_add(
                    vec3_add(scene.camera.forward, vec3_scale(scene.camera.right, vx)),
                    vec3_scale(scene.camera.up, vy),
                );

                let t_min = vec3_len(ray_dir);
                let t_max = f64::INFINITY;
                let recursion_depth = 3;

                // Trace a ray from the camera through the viewport
                let color = trace_ray(&scene, ray_origin, ray_dir, t_min, t_max, recursion_depth);

                // Plot a pixel to span buffer
                {
                    let color = color_to_u32(color);
                    span_buffer[n] = color;
                }
            }

            {
                // Get write access to shared buffer and copy rendered span to it
                let mut shared_buffer;
                if buffer_0_active {
                    shared_buffer = self.buffer_0.lock().unwrap()
                } else {
                    shared_buffer = self.buffer_1.lock().unwrap()
                }
                let slice = &mut shared_buffer[pixel..(pixel + RENDER_SPAN)];
                slice.copy_from_slice(&span_buffer);
            }

            if done {
                // This thread completed the render
                let duration;

                {
                    // Read/write shared data
                    let mut data = self.data.lock().unwrap();

                    // Swap buffers
                    data.buffer_0_active = !data.buffer_0_active;

                    // Record duration of render
                    duration = Instant::now().duration_since(data.start_time);
                    data.duration = duration;
                }

                // Call completion callback
                self.completion_callback.lock().unwrap()(duration);
            }
        }
    }
}

fn color_to_u32(color: (f64, f64, f64)) -> u32 {
    let (mut r, mut g, mut b) = color;

    if r > 1.0 {
        r = 1.0
    };
    if g > 1.0 {
        g = 1.0
    };
    if b > 1.0 {
        b = 1.0
    };

    ((255.0 * r) as u32) << 16 | ((255.0 * g) as u32) << 8 | ((255.0 * b) as u32)
}

fn trace_ray(
    scene: &Scene,
    ray_origin: Vector3<f64>,
    ray_dir: Vector3<f64>,
    t_min: f64,
    t_max: f64,
    recursion_depth: usize,
) -> (f64, f64, f64) {
    if false {
        // DEBUG: Simulate a slow computer
        thread::sleep(Duration::from_millis(1));
    }

    let (closest_sphere, closest_t) =
        intersect_ray_closest_sphere(scene, ray_origin, ray_dir, t_min, t_max);

    if let Some(sphere) = closest_sphere {
        // Ray hit a sphere, calculate hit position and normal
        let hit_pos: Vector3<f64> = vec3_add(ray_origin, vec3_scale(ray_dir, closest_t));
        let hit_normal: Vector3<f64> = vec3_normalized(vec3_sub(hit_pos, sphere.pos));

        // Sum light intensities at hit position, taking normal into account
        let intensity = if sphere.skip_lighting {
            // Full brightness (e.g., sky sphere)
            1.0
        } else {
            compute_lighting(scene, ray_dir, hit_pos, hit_normal, sphere.specular)
        };

        // Get color from sphere texture
        let (mut r, mut g, mut b) = match sphere.texture {
            // Solid color
            Texture::Color(r, g, b) => (r, g, b),

            // Checker pattern on X-Z plane
            Texture::CheckerXZ {
                color1,
                color2,
                scale,
            } => {
                let scale_05x = scale / 2.0;
                let scale_2x = scale * 2.0;
                let (x, z) = (hit_pos[0] - scale_05x, hit_pos[2] - scale_05x);
                let x_toggle = ((x % scale_2x).abs() >= scale) ^ (x < 0.0);
                let z_toggle = ((z % scale_2x).abs() >= scale) ^ (z < 0.0);
                if x_toggle ^ z_toggle == true {
                    color2
                } else {
                    color1
                }
            }

            // Vertical gradient (e.g., sky sphere)
            Texture::GradientY { color1, color2 } => {
                let mut y = (hit_pos[1] - sphere.pos[1]) / sphere.r;

                if y > 1.0 {
                    y = 1.0;
                } else if y < -1.0 {
                    y = -1.0;
                }

                let ny = 1.0 - y;

                (
                    color1.0 * y + color2.0 * ny,
                    color1.1 * y + color2.1 * ny,
                    color1.2 * y + color2.2 * ny,
                )
            }
        };

        // Apply total light intensity to texture color
        (r, g, b) = (r * intensity, g * intensity, b * intensity);

        // Calculate reflections
        let reflective = sphere.reflective;
        if recursion_depth > 0 && reflective > 0.0 {
            let (t_min, t_max) = (RENDER_EPSILON, f64::INFINITY);

            // Calculate reflection recursively
            let refl_dir = reflect_ray(vec3_scale(ray_dir, -1.0), hit_normal);
            let (refl_r, refl_g, refl_b) =
                trace_ray(scene, hit_pos, refl_dir, t_min, t_max, recursion_depth - 1);

            // Mix object color and reflected color together in proportion
            r = r * (1.0 - reflective) + refl_r * reflective;
            g = g * (1.0 - reflective) + refl_g * reflective;
            b = b * (1.0 - reflective) + refl_b * reflective;
        }

        (r, g, b)
    } else {
        // Ray did not hit anything
        scene.sky_color
    }
}

fn intersect_ray_closest_sphere(
    scene: &Scene,
    ray_origin: Vector3<f64>,
    ray_dir: Vector3<f64>,
    t_min: f64,
    t_max: f64,
) -> (Option<&Sphere>, f64) {
    let mut closest_t: f64 = f64::INFINITY;
    let mut closest_sphere: Option<&Sphere> = None;

    // See if ray hits any of the spheres
    for sphere in &scene.spheres {
        let (t1, t2) = intersect_ray_sphere(ray_origin, ray_dir, sphere);

        if t1 >= t_min && t1 <= t_max && t1 < closest_t {
            closest_t = t1;
            closest_sphere = Some(sphere);
        }

        if t2 >= t_min && t2 <= t_max && t2 < closest_t {
            closest_t = t2;
            closest_sphere = Some(sphere);
        }
    }

    (closest_sphere, closest_t)
}

fn intersect_ray_sphere(
    ray_origin: Vector3<f64>,
    ray_dir: Vector3<f64>,
    sphere: &Sphere,
) -> (f64, f64) {
    let r = sphere.r;
    let co = vec3_sub(ray_origin, sphere.pos);

    let a = vec3_dot(ray_dir, ray_dir);
    let b = 2.0 * vec3_dot(co, ray_dir);
    let c = vec3_dot(co, co) - r * r;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        // No hit
        return (f64::INFINITY, f64::INFINITY);
    }

    let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
    let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

    return (t1, t2);
}

fn compute_lighting(
    scene: &Scene,
    ray_dir: Vector3<f64>,
    hit_pos: Vector3<f64>,
    hit_normal: Vector3<f64>,
    specular: f64,
) -> f64 {
    let mut total_intensity = 0.0;

    // Iterate over lights in the scene and add their intensities together
    for light in &scene.lights {
        let light_intensity;
        let light_dir: Vector3<f64>;
        let t_min = RENDER_EPSILON;
        let t_max;

        match light {
            Light::Ambient { intensity } => {
                // Ambient light is non-directional
                total_intensity += intensity;
                continue;
            }
            Light::Point { intensity, pos } => {
                light_intensity = *intensity;
                light_dir = vec3_sub(*pos, hit_pos);
                t_max = 1.0;
            }
            Light::Directional { intensity, dir } => {
                light_intensity = *intensity;
                light_dir = *dir; // Just the light direction directly
                t_max = f64::INFINITY;
            }
        }

        // Shadow check
        let (shadow_sphere, _) =
            intersect_ray_closest_sphere(scene, hit_pos, light_dir, t_min, t_max);
        if let Some(_) = shadow_sphere {
            // Sphere hit, so in shadow
            continue;
        }

        let n_dot_l = vec3_dot(hit_normal, light_dir);

        // Calculate direction-dependent intensity for diffuse lighting
        if n_dot_l > 0.0 {
            let n_dot_l_norm = n_dot_l / (vec3_len(hit_normal) * vec3_len(light_dir));
            total_intensity += light_intensity * n_dot_l_norm;
        }

        // Calculate direction-dependent specular highlights
        if specular >= 0.0 {
            let view_dir = vec3_scale(ray_dir, -1.0);
            let reflection_dir = reflect_ray(light_dir, hit_normal);
            let r_dot_v = vec3_dot(reflection_dir, view_dir);
            if r_dot_v > 0.0 {
                let r_dot_v_norm = r_dot_v / (vec3_len(reflection_dir) * vec3_len(ray_dir));
                total_intensity += r_dot_v_norm.powf(specular);
            }
        }
    }

    total_intensity
}

fn reflect_ray(ray: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    let n_dot_r = vec3_dot(normal, ray);
    return vec3_sub(vec3_scale(normal, 2.0 * n_dot_r), ray);
}
