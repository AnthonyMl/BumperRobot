use std::{collections::VecDeque, ops::Sub};

use crate::img::Rectangle;


// everything in this file assumes L8 image input

const UNCOLORED_BACKGROUND: u8 = u8::MIN;
const COLORED_BACKGROUND: u8 = 1;
const UNCOLORED_REGION: u8 = u8::MAX;

#[derive(Clone)]
pub struct Component<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub area: T,
    pub id: u8,
}

impl Component<usize> {
    // private
    // initializes to extreme values for use with min() and max()
    fn new(id: u8) -> Self {
        Component {
            left: usize::MAX,
            top: usize::MAX,
            right: usize::MIN,
            bottom: usize::MIN,
            area: 0,
            id,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Component<T> {
    pub fn bounding_box(&self) -> Rectangle<T> {
        Rectangle {
            left: self.left,
            top: self.top,
            width: self.right - self.left,
            height: self.bottom - self.top,
        }
    }
}

fn neighbors(img: &[u8], width: usize, height: usize, x: usize, y: usize) -> Vec<(usize, usize)> {
    let idx = y * width + x;

    let mut r = vec![];

    if x > 0 {
        let v1 = img[idx - 1];
        if v1 == UNCOLORED_REGION || v1 == UNCOLORED_BACKGROUND { r.push((x - 1, y)) }
    }
    if x < width - 1 {
        let v1 = img[idx + 1];
        if v1 == UNCOLORED_REGION || v1 == UNCOLORED_BACKGROUND { r.push((x + 1, y)) }
    }
    if y > 0 {
        let v1 = img[idx - width];
        if v1 == UNCOLORED_REGION || v1 == UNCOLORED_BACKGROUND { r.push((x , y - 1)) }
    }
    if y < height - 1 {
        let v1 = img[idx + width];
        if v1 == UNCOLORED_REGION || v1 == UNCOLORED_BACKGROUND { r.push((x , y + 1)) }
    }
    r
}

fn dfs_cc(img: &mut[u8], q: &mut VecDeque<(usize, usize)>, width: usize, height: usize, id: u8, x: usize, y: usize) -> Component<usize>{
    let mut stack: Vec<(usize, usize)> = Vec::new();

    let mut component: Component<usize> = Component::new(id);

    stack.push((x, y));

    while let Some((x, y)) = stack.pop() {
        let idx = y * width + x;

        match img[idx] {
            UNCOLORED_REGION => {
                img[idx] = id;

                component.area += 1;
                component.left   = component.left.min(x);
                component.top    = component.top.min(y);
                component.right  = component.right.max(x);
                component.bottom = component.bottom.max(y);

                for neighbor in neighbors(img, width, height, x, y) {
                    stack.push(neighbor)
                }
            },
            UNCOLORED_BACKGROUND => { q.push_back((x, y)) },
            COLORED_BACKGROUND => {},
            other_id if other_id == id => {},
            _ => unreachable!("dfs_cc: encountered another region while coloring")
        }
    }
    component
}

// recolors the binary (0/255) input image
// 0 -> 1 (background)
// 255 -> 2.. for each component
pub fn connected_components(img: &mut [u8], width: usize, height: usize) -> Vec<Component<usize>> {
    let mut id: u8 = COLORED_BACKGROUND;
    let mut q: VecDeque<(usize, usize)> = VecDeque::new();
    let mut components = vec![];

    q.push_back((0, 0));

    while let Some((x, y)) = q.pop_front() {
        let idx = y * width + x;

        match img[idx] {
            UNCOLORED_REGION => {
                id += 1;
                components.push(dfs_cc(img, &mut q, width, height, id, x, y))
            },
            UNCOLORED_BACKGROUND => {
                img[idx] = COLORED_BACKGROUND;
                for neighbor in neighbors(img, width, height, x, y) {
                    q.push_back(neighbor)
                }
            },
            _ => {}
        }
    }
    components
}
