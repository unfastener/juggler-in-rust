// renderer.rs - Open a resizable window and allow rendering pixels to it

use softbuffer::{Context, Surface};
use std::cmp::min;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Fullscreen, WindowBuilder};

use crate::renderer::{SceneOptions, SharedRenderer};

const WINDOW_REDRAW_PERIOD: f64 = 0.5; // Window redraw period in seconds
const FPS_REFRESH_PERIOD: f64 = 0.25; // Update FPS counter this often

#[derive(Debug, Clone, Copy)]
enum UserEvent {
    RequestRedraw,
}

pub struct Window {
    renderer: SharedRenderer,
    size: Option<(usize, usize)>,
    title: String,
    default_color: u32,
}

impl Window {
    pub fn new(renderer: &SharedRenderer) -> Self {
        let renderer = renderer.clone();

        // Store default color for window
        let default_color;
        {
            // Read top-left pixel from render buffer
            let buffer = renderer.get_buffer();
            let buffer = buffer.lock().unwrap();
            default_color = buffer[0];
        }

        Self {
            renderer,
            size: None,
            title: "".to_string(),
            default_color,
        }
    }

    pub fn _set_initial_size(&mut self, size: Option<(usize, usize)>) {
        self.size = size;
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn run(&self) {
        let (mut width, mut height) = match self.size {
            Some(size) => size,
            None => self.renderer.get_size(),
        };

        // Create a Winit Window and everything else needed to draw to it and handle events
        let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
            .build()
            .unwrap();
        let winit_window = Arc::new(
            WindowBuilder::new()
                .with_min_inner_size(PhysicalSize::new(width as u32, height as u32))
                .with_inner_size(PhysicalSize::new(width as u32, height as u32))
                .with_title(&self.title)
                .build(&event_loop)
                .unwrap(),
        );

        // Scale window 2x, 3x, ... depending on display resolution
        // NOTE: Wayland returns None for current_monitor(), so use
        // the first entry in available_monitors() instead
        for monitor in winit_window.available_monitors() {
            let max_size = monitor.size();

            let max_w_scale = max_size.width as usize / width;
            let max_h_scale = max_size.height as usize / height;
            let max_scale = min(max_w_scale, max_h_scale);

            if max_scale > 0 {
                // Scale window size by an integer multiple
                width *= max_scale;
                height *= max_scale;
            } else {
                // Render is too large for the display, use maximum window size
                (width, height) = (max_size.width as usize, max_size.height as usize);
            }

            // Even if None is returned, it is not an error
            let _ = winit_window.request_inner_size(PhysicalSize::new(width as u32, height as u32));

            // Only consider the first monitor
            break;
        }

        // Create a SoftBuffer Context and Surface for drawing pixels
        let context = Context::new(winit_window.clone()).unwrap();
        let mut surface = Surface::new(&context, winit_window.clone()).unwrap();

        // Set completion callback to send a redraw request to the Winit window
        {
            let event_loop_proxy = event_loop.create_proxy();
            self.renderer.set_completion_callback(move |frame_time| {
                event_loop_proxy
                    .send_event(UserEvent::RequestRedraw)
                    .unwrap();

                if false {
                    // DEBUG: Print render duration
                    println!("{:.03} s", frame_time.as_secs_f64())
                }
            });
        }

        let mut start_time = Instant::now(); // Set in StartCause::Init event handler
        let mut fps_counter = FPSCounter::new();
        let timer_duration = Duration::from_secs_f64(WINDOW_REDRAW_PERIOD);

        // Default scene options
        let mut scene_options = SceneOptions {
            speed_0: 1.0,
            speed_1: 1.0,
            option_0: false,
            option_1: false,
        };

        let mut initialized = false;

        // Run event loop
        event_loop
            .run(move |event, elwt| {
                if false {
                    // DEBUG: Print events
                    println!("{:?}", event);
                }

                match event {
                    // Handle start event
                    Event::NewEvents(StartCause::Init) => {
                        // Just started
                        if false {
                            elwt.set_control_flow(ControlFlow::WaitUntil(
                                Instant::now() + timer_duration,
                            ));
                        } else {
                            // DEBUG: No timer required, for now
                            elwt.set_control_flow(ControlFlow::Wait);
                        }

                        // Start rendering the first frame
                        start_time = Instant::now();
                        fps_counter.reset();
                        self.renderer.start_render(Duration::ZERO, &scene_options);
                        initialized = true;
                    }
                    // Handle timer event
                    Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                        // DEBUG: No timer required, for now
                        if false {
                            // Event timeout expired
                            elwt.set_control_flow(ControlFlow::WaitUntil(
                                Instant::now() + timer_duration,
                            ));
                            winit_window.request_redraw();
                        }
                    }
                    // Handle requests from other threads
                    Event::UserEvent(_) => {
                        winit_window.request_redraw();
                    }
                    // Handle window redraw request event
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::RedrawRequested,
                    } if window_id == winit_window.id() => {
                        // Redraw requested
                        if let (Some(width), Some(height)) = {
                            let size = winit_window.inner_size();
                            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                        } {
                            // Resize surface if needed
                            surface.resize(width, height).unwrap();

                            if initialized {
                                // Wait for all threads to complete
                                self.renderer.wait_for_completion(false);
                            }

                            // Update title with new FPS every once in a while
                            if let Some(fps) = fps_counter.new_frame(self.renderer.get_duration()) {
                                let (render_width, render_height) = self.renderer.get_size();
                                winit_window.set_title(
                                    format!(
                                        "{} - {}x{} - {:.1} fps",
                                        self.title, render_width, render_height, fps
                                    )
                                    .as_str(),
                                );
                            }

                            let fullscreen = winit_window.fullscreen().is_some();

                            // Scale and copy shared render buffer contents to surface
                            let mut buffer = surface.buffer_mut().unwrap();
                            self.redraw(
                                &mut buffer,
                                (width.get() as usize, height.get() as usize),
                                fullscreen,
                            );

                            // Update window contents with surface contents
                            buffer.present().unwrap();

                            if initialized {
                                // Start rendering another frame
                                let duration_since_start =
                                    Instant::now().duration_since(start_time);
                                self.renderer
                                    .start_render(duration_since_start, &scene_options);
                            }
                        }
                    }
                    // Handle window close request event
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == winit_window.id() => {
                        // Window closed, exit event loop
                        elwt.exit();
                    }
                    // Handle keyboard events
                    Event::WindowEvent {
                        event:
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        logical_key,
                                        state: ElementState::Pressed,
                                        ..
                                    },
                                ..
                            },
                        window_id,
                    } if window_id == winit_window.id() => {
                        match logical_key.as_ref() {
                            Key::Named(NamedKey::Escape) | Key::Character("q") => {
                                // Exit event loop
                                elwt.exit();
                            }
                            Key::Named(NamedKey::F11) | Key::Character("f") => {
                                // Toggle fullscreen
                                let fullscreen = if winit_window.fullscreen().is_some() {
                                    None
                                } else {
                                    Some(Fullscreen::Borderless(None))
                                };
                                winit_window.set_fullscreen(fullscreen);
                            }
                            // Set (scene dependent) speed 0
                            Key::Character("1") => {
                                scene_options.speed_0 = 0.0;
                            }
                            Key::Character("2") => {
                                scene_options.speed_0 = 0.5;
                            }
                            Key::Character("3") => {
                                scene_options.speed_0 = 1.0;
                            }
                            Key::Character("4") => {
                                scene_options.speed_0 = 1.5;
                            }
                            Key::Character("5") => {
                                scene_options.speed_0 = 2.0;
                            }
                            // Set (scene dependent) speed 1
                            Key::Character("6") => {
                                scene_options.speed_1 = -2.0;
                            }
                            Key::Character("7") => {
                                scene_options.speed_1 = -1.0;
                            }
                            Key::Character("8") => {
                                scene_options.speed_1 = 0.0;
                            }
                            Key::Character("9") => {
                                scene_options.speed_1 = 1.0;
                            }
                            Key::Character("0") => {
                                scene_options.speed_1 = 2.0;
                            }
                            // Toggle (scene dependent) option 0
                            Key::Character("a") => {
                                scene_options.option_0 = !scene_options.option_0;
                            }
                            // Toggle (scene dependent) option 1
                            Key::Character("b") => {
                                scene_options.option_1 = !scene_options.option_1;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            })
            .unwrap();
    }

    fn redraw(&self, target_buffer: &mut [u32], target_size: (usize, usize), fullscreen: bool) {
        // Get default color for filling unused parts of the window
        let default_color = if !fullscreen {
            self.default_color
        } else {
            // Fullscreen, always black border
            0x0000_0000
        };

        let (target_width, target_height) = target_size;

        let fill_x = |buffer: &mut [u32], pad_x: usize, width: usize| {
            for target_y in 0..target_height {
                for target_x in 0..pad_x {
                    buffer[target_y * target_width + target_x] = default_color;
                }
                for target_x in (pad_x + width)..target_width {
                    buffer[target_y * target_width + target_x] = default_color;
                }
            }
        };

        let fill_y = |buffer: &mut [u32], pad_y: usize, height: usize| {
            for target_y in 0..pad_y {
                for target_x in 0..target_width {
                    buffer[target_y * target_width + target_x] = default_color;
                }
            }

            for target_y in (pad_y + height)..target_height {
                for target_x in 0..target_width {
                    buffer[target_y * target_width + target_x] = default_color;
                }
            }
        };

        // Get read access to shared buffer
        let (render_width, render_height) = self.renderer.get_size();
        let render_buffer = self.renderer.get_buffer();
        let source_buffer = render_buffer.lock().unwrap();

        let nearest_neighbor =
            |buffer: &mut [u32], left_x: usize, top_y: usize, width: usize, height: usize| {
                // Nearest neighbor scaling
                for target_y in top_y..(top_y + height) {
                    let source_y = ((target_y - top_y) * render_height) / height;
                    for target_x in left_x..(left_x + width) {
                        let source_x = ((target_x - left_x) * render_width) / width;
                        let value = source_buffer[source_y * render_width + source_x];
                        buffer[target_y * target_width + target_x] = value;
                    }
                }
            };

        // Preserve aspect ratio, fill with default_color outside rendered image
        if render_width * target_height <= render_height * target_width {
            // Window is wider than rendered image
            let pad_x = (target_width - target_height) / 2;
            nearest_neighbor(target_buffer, pad_x, 0, target_height, target_height);
            fill_x(target_buffer, pad_x, target_height);
        } else {
            // Window is taller than rendered image
            let pad_y = (target_height - target_width) / 2;
            nearest_neighbor(target_buffer, 0, pad_y, target_width, target_width);
            fill_y(target_buffer, pad_y, target_width);
        }
    }
}

struct FPSCounter {
    last_update_time: Option<Instant>,
    durations: Duration,
    num_frames: usize,
}

impl FPSCounter {
    fn new() -> Self {
        Self {
            last_update_time: None,
            durations: Duration::ZERO,
            num_frames: 0,
        }
    }

    fn reset(&mut self) {
        *self = Self {
            last_update_time: None,
            durations: Duration::ZERO,
            num_frames: 0,
        };
    }

    fn new_frame(&mut self, frame_duration: Duration) -> Option<f64> {
        let now = Instant::now();

        // Calculate FPS every FPS_REFRESH_PERIOD
        self.num_frames += 1;
        self.durations += frame_duration;

        let fps = match self.last_update_time {
            None => {
                // First frame, skip updating FPS
                self.last_update_time = Some(now);
                None
            }
            Some(last_time) => {
                if now.duration_since(last_time).as_secs_f64() < FPS_REFRESH_PERIOD {
                    // Not yet time to update FPS
                    None
                } else {
                    // Time to update, calculate FPS
                    let fps = self.num_frames as f64 / self.durations.as_secs_f64();

                    self.last_update_time = Some(now);
                    self.durations = Duration::ZERO;
                    self.num_frames = 0;

                    Some(fps)
                }
            }
        };

        fps
    }
}
