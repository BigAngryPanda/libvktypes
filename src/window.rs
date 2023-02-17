//! Helper functions around `winit` library

use winit::event_loop::EventLoopBuilder;
use winit::platform::unix::EventLoopBuilderExtUnix;
use winit::error::OsError;

pub type EventLoop = winit::event_loop::EventLoop<()>;
pub type Window = winit::window::Window;

#[cfg(target_os = "linux")]
/// Only for Linux with X11
pub fn eventloop() -> EventLoop {
    EventLoopBuilder::new().with_x11().with_any_thread(true).build()
}

pub fn create_window(eventloop: &EventLoop) -> Result<Window, OsError> {
    winit::window::Window::new(&eventloop)
}