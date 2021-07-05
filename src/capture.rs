use scrap::{Capturer, Display};

use crate::{capture_windows::FindWindow, img::{IMAGE_DOWNSCALE_FACTOR}, img::Rectangle};


// FIXME: doesnt belong here or in the overlay
pub const ACTIVE_RECTANGLE: Rectangle<f32> = Rectangle {
    left: 0.04939,
    top: 0.10709,
    width: 0.90061,
    height: 0.61402,
};

pub struct Capture {
    capturer: Capturer,

    pub window: Rectangle<usize>,
}

impl Capture {
    pub fn new(window_title: &str) -> Option<Self> {
        let capturer = Capturer::new(
            Display::primary().expect("Couldn't find primary display.")
        ).expect("Couldn't initialize screen capturing");

        Capture::find_window(window_title).map(|window| Capture{capturer, window})
    }

    pub fn try_frame(&mut self) -> Option<(Vec<u8>, Rectangle<usize>)> {
        let screen_width = self.capturer.width();
        let window = &self.window;

        self.capturer.frame().map_or(None, |buffer| {
            let active_rect = active_rectangle(window);

            let img = copy_rectangle_bgr_to_rgb(&buffer, screen_width, &active_rect);

            Some((img, active_rect))
        })
    }
}

// BGR -> RGB
// no bound checks
fn copy_rectangle_bgr_to_rgb(data: &[u8], stride: usize, rectangle: &Rectangle<usize>) -> Vec<u8> {
    let mut r: Vec<u8> = vec![0; 3 * rectangle.width * rectangle.height];

    for y in 0..rectangle.height {
        for x in 0..rectangle.width {
            let idx_dst = 3 * (rectangle.width * y + x);
            let idx_src = 4 * (stride * (rectangle.top as usize + y) + rectangle.left as usize + x);

            r[idx_dst    ] = data[idx_src + 2];
            r[idx_dst + 1] = data[idx_src + 1];
            r[idx_dst + 2] = data[idx_src    ];
        }
    }
    r
}

fn active_rectangle(window_rect: &Rectangle<usize>) -> Rectangle<usize> {
    let left   = window_rect.left + (window_rect.width  as f32 * ACTIVE_RECTANGLE.left) as usize;
    let top    = window_rect.top  + (window_rect.height as f32 * ACTIVE_RECTANGLE.top) as usize;
    let width  = (window_rect.width  as f32 * ACTIVE_RECTANGLE.width )  as usize;
    let height = (window_rect.height as f32 * ACTIVE_RECTANGLE.height)  as usize;

    Rectangle { top, left, width, height }
}

pub fn active_pixel_to_screen(rectangle: &Rectangle<usize>, active_area: &Rectangle<usize>) -> Rectangle<usize> {
    const S: usize = IMAGE_DOWNSCALE_FACTOR;

    let left = active_area.left + rectangle.left * S;

    let top = active_area.top + rectangle.top * S;

    let width  = S * rectangle.width;
    let height = S * rectangle.height;

    Rectangle { left, top, width, height }
}

pub fn active_pixel_to_window_normalized(rectangle: &Rectangle<usize>, window: &Rectangle<usize>) -> Rectangle<f32> {
    const S: usize = IMAGE_DOWNSCALE_FACTOR;

    let active_area = active_rectangle(window);

    let left = {
        let left = active_area.left - window.left + rectangle.left * S;
        left as f32 / window.width as f32
    };

    let top = {
        let top = active_area.top - window.top + rectangle.top * S;
        top as f32 / window.height as f32
    };

    let width  = (S * rectangle.width)  as f32 / window.width  as f32;
    let height = (S * rectangle.height) as f32 / window.height as f32;

    Rectangle { left, top, width, height }
}
