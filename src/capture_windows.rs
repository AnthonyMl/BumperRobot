use winapi::{shared::windef, um::winuser as win};

use crate::{capture::Capture, img::Rectangle};


pub trait FindWindow {
    fn find_window(window_name: &str) -> Option<Rectangle<usize>>;
}

impl FindWindow for Capture {
    fn find_window(window_name: &str) -> Option<Rectangle<usize>> {
        let window_name = std::ffi::CString::new(window_name).unwrap();

        unsafe {
            let handle = win::FindWindowA(std::ptr::null_mut(), window_name.as_ptr());

            if handle.is_null() { return None }

            win::SetForegroundWindow(handle);
            win::ShowWindow(handle, 9);

            let mut rect = windef::RECT { left: 0, top: 0, right: 0, bottom: 0 };
            let lprect = &mut rect as *mut windef::RECT;
            win::GetWindowRect(handle, lprect);

            Some( Rectangle {
                top:     rect.top                 as usize,
                left:    rect.left                as usize,
                width:  (rect.right  - rect.left) as usize,
                height: (rect.bottom - rect.top)  as usize,
            })
        }
    }
}

enum MouseAction {
    Move { x: usize, y: usize },
    Press,
    Release,
}

impl MouseAction {
    fn send(&self) {
        // TODO: fetch dynamically from Capture.Capturer.width or elsewhere
        const SCREEN_WIDTH: usize = 3840;
        const SCREEN_HEIGHT: usize = 2160;

        let input_union = {
            let mut data = unsafe { std::mem::zeroed::<win::INPUT_u>() };
            let binding = unsafe { data.mi_mut() };

            binding.dwFlags = win::MOUSEEVENTF_ABSOLUTE;

            match self {
                MouseAction::Move { x, y } => {
                    binding.dx = (x * u16::MAX as usize / SCREEN_WIDTH) as i32;
                    binding.dy = (y * u16::MAX as usize / SCREEN_HEIGHT) as i32;

                    binding.dwFlags |= win::MOUSEEVENTF_MOVE;
                },
                MouseAction::Press => {
                    binding.dwFlags |= win::MOUSEEVENTF_LEFTDOWN;
                },
                MouseAction::Release => {
                    binding.dwFlags |= win::MOUSEEVENTF_LEFTUP;
                },
            }
            data
        };

        let mut input = win::INPUT {
            type_: win::INPUT_MOUSE,
            u: input_union,
        };

        unsafe {
            win::SendInput(
                1,
                &mut input as *mut win::INPUT,
                std::mem::size_of::<win::INPUT>() as i32
            );
        }
    }
}

pub fn mouse_move(x: usize, y: usize) {
    MouseAction::Move{ x, y }.send()
}

pub fn mouse_press() {
    MouseAction::Press.send()
}

pub fn mouse_release() {
    MouseAction::Release.send()
}
