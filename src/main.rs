#![allow(clippy::option_map_unit_fn)]

use std::time::{Duration, Instant};

use capture_windows::mouse_press;
use img::Rectangle;
use robot::Robot;
use winit::{event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}};

use crate::{capture::{Capture}, overlay::Overlay};


pub mod robot;
pub mod img;
pub mod img_connected_components;
pub mod overlay;
pub mod capture;
pub mod capture_windows;
pub mod rectangle_renderer;
pub mod rectangle_data;
pub mod background;
pub mod kalman;


enum Execution {
    Headless(Duration),
    UseOverlay {
        event_loop: Box<EventLoop<()>>,
        overlay: Box<Overlay>,
    },
}

fn main() {
    let mut capture = Capture::new("Ys Chronicles+: Ancient Ys Vanished - Omen")
        .expect("Couldn't capture window");

    let mut robot = Robot::new().expect("Unable to initialize robot");

    let execution_mode =
        if let Some("-o" | "--overlay") = std::env::args().nth(1).as_deref() {
            let event_loop = Box::new(EventLoop::new());

            let overlay = Box::new(futures::executor::block_on(Overlay::new(
                &event_loop,
                &capture.window
            )));

            Execution::UseOverlay { event_loop, overlay }
        } else {
            Execution::Headless(Duration::from_secs(10))
        };

    match execution_mode {
        Execution::Headless(time_limit) => {
            let t0 = Instant::now();

            while (Instant::now() - t0) < time_limit {
                tick(&mut capture, &mut robot);

                // FIXME: workaround for the overlay taking focus on input
                mouse_press();
            }
        },
        Execution::UseOverlay{ event_loop, mut overlay } => {
            event_loop.run(move |event, _, control_flow| match event {
                Event::WindowEvent { event, window_id } if window_id == overlay.window.id() =>
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    },
                    WindowEvent::KeyboardInput { input: KeyboardInput { state: ElementState::Pressed, virtual_keycode, ..}, ..} =>
                    match virtual_keycode {
                        Some(VirtualKeyCode::Escape) => {
                            *control_flow = ControlFlow::Exit
                        },
                        Some(_) => { /*
                            FIXME: silence clippy until more inputs are added
                            or until #[allow(clippy::single_match)] on expressions becomes stable
                            */
                        },
                        _ => {}
                    },
                    _ => {}
                },
                Event::RedrawRequested(_) => {
                    overlay.update();

                    if let Err(e) = overlay.render() {
                        eprintln!("fatal rendering error {}", e);
                        *control_flow = ControlFlow::Exit;
                    }
                },
                Event::MainEventsCleared => {
                    if let Some(rectangles) = tick(&mut capture, &mut robot) {

                        overlay.clear();
                        for r in &rectangles {
                            let r = capture::active_pixel_to_window_normalized(r, &capture.window);
                            overlay.add(&r);
                        }
                        overlay.window.request_redraw()
                    }
                },
                _ => {}
            });
        }
    }
}

fn tick(capture: &mut Capture, robot: &mut Robot) -> Option<Vec<Rectangle<usize>>> {
    if let Some((frame, active_area)) = capture.try_frame() {

        let rectangles = robot.process_frame(frame, &active_area);

        return Some(rectangles)
    }
    None
}
