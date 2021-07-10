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

        let fireball_rects = fireballs(&img, width, height);
        let crystal_rects = crystals(&img, width, height);

        // response

        let (x, y) = centroid(&active_pixel_to_screen(&h_target, active_area));

        // FIXME: default values not in window
        // Causes loss of focus if detection fails on first frame
        mouse_move(x, y);

        let mut r = vec![h_target, h_other];

        r.extend(fireball_rects);
        r.extend(crystal_rects);

        r
    }
}

fn crystals(img: &[u8], width: usize, height: usize) -> Vec<Rectangle<usize>> {
    const RGB_THRESHOLDS: [(u8, u8); 3] = [(10,255), (30,204), (84,255)];

    projectiles(img, width, height, &RGB_THRESHOLDS)
}

fn fireballs(img: &[u8], width: usize, height: usize) -> Vec<Rectangle<usize>> {
    const RGB_THRESHOLDS: [(u8, u8); 3] = [(160,255), (14,250), (1,255)];

    projectiles(img, width, height, &RGB_THRESHOLDS)
}

fn projectiles(img: &[u8], width: usize, height: usize, rgb_thresholds: &[(u8, u8); 3]) -> Vec<Rectangle<usize>> {
    const MIN_AREA: usize = 28;
    const MAX_AREA: usize = 62;
    const ROUNDNESS: f32 = 0.25;

    let img = img::threshold(img, rgb_thresholds);

    let mut img = img::median3x3(&img, width, height);

    connected_components(&mut img, width, height)
        .into_iter()
        .filter(|c|{
            let width = c.right - c.left;
            let height = c.bottom - c.top;
            let ratio = width as f32 / height as f32;

            c.area > MIN_AREA &&
            c.area < MAX_AREA &&
            ratio > (1.0 - ROUNDNESS) &&
            ratio < (1.0 + ROUNDNESS)
        })
        .map(|c| c.bounding_box())
        .collect()
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
        const BLUE_THRESHOLD_UPPER: u8 = 203;

        let img = img::threshold_blue(img, BLUE_THRESHOLD_LOWER, BLUE_THRESHOLD_UPPER);

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
