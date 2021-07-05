use crate::{background, capture::active_pixel_to_screen, capture_windows::{mouse_move, mouse_release}, img::{self, IMAGE_DOWNSCALE_FACTOR}, img::Rectangle, img_connected_components::{connected_components}};


pub struct Robot {
    background: Vec<u8>,
    head_tracker: HeadTracker,
}

impl Drop for Robot {
    fn drop(&mut self) {
        mouse_release();
    }
}

impl Robot {
    pub fn new() -> Option<Robot> {
        let background = background::fetch_background()?;

        let head_tracker = HeadTracker::new();

        Some( Robot {
            background,
            head_tracker,
        })
    }

    pub fn process_frame(&mut self, img: Vec<u8>, active_area: &Rectangle<usize>) -> Vec<Rectangle<usize>> {
        let (width, height, img) = img::shrink(img, active_area.width, active_area.height, IMAGE_DOWNSCALE_FACTOR);

        let img = img::remove_background(&img, &self.background);

        self.head_tracker.update(&img, width, height);

        let h_other  = self.head_tracker.bound.clone();
        let h_target = mirror_horizontal(&h_other, width); // target the non-blue head

        // response

        let (x, y) = centroid(&active_pixel_to_screen(&h_target, active_area));

        // FIXME: default values not in window
        // Causes loss of focus if detection fails on first frame
        mouse_move(x, y);

        return vec![h_target, h_other];
    }
}

fn centroid(r: &Rectangle<usize>) -> (usize, usize) {
    ( r.left + r.width / 2
    , r.top + r.height / 2 )
}

fn mirror_horizontal(r: &Rectangle<usize>, width: usize) -> Rectangle<usize> {
    Rectangle { left: width - r.left - r.width, .. *r }
}

// BlueHeadTracker?
struct HeadTracker {
    pub bound: Rectangle<usize>,
}

impl HeadTracker {
    fn update(&mut self, img: &[u8], width: usize, height: usize) {
        const BLUE_THRESHOLD_LOWER: u8 = 57;
        const BLUE_THRESHOLD_HIGHER: u8 = 203;

        let img = img::threshold_blue(img, BLUE_THRESHOLD_LOWER, BLUE_THRESHOLD_HIGHER);

        let mut img = img::median3x3(&img, width, height);

        connected_components(&mut img, width, height)
            .into_iter()
            .max_by(|a, b|{ a.area.cmp(&b.area) })
            .map(|c| self.bound = c.bounding_box());
    }

    // default instead ?
    fn new() -> Self {
        HeadTracker {
            bound: Rectangle { left: 0, top: 0, width: 0, height: 0 },
        }
    }
}

