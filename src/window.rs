//! Helper functions around `winit` library

use winit::event_loop::EventLoopBuilder;
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::platform::wayland::EventLoopBuilderExtWayland;
use winit::error::OsError;

pub type EventLoop = winit::event_loop::EventLoop<()>;
pub type Window = winit::window::Window;

#[cfg(target_os = "linux")]
/// Create new eventloop
///
/// Event loop can be used in different thread (unlike original winit event loop)
pub fn eventloop() -> winit::event_loop::EventLoop<()> {
    let mut builder = EventLoopBuilder::new();
    EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
    EventLoopBuilderExtX11::with_any_thread(&mut builder, true).build()
}

#[cfg(not(target_os = "linux"))]
/// Create new eventloop
///
/// Event loop can be used in different thread (unlike original winit event loop)
pub fn eventloop() -> winit::event_loop::EventLoop<()> {
    EventLoopBuilder::new().with_any_thread(true).build()
}

pub fn create_window(eventloop: &EventLoop) -> Result<Window, OsError> {
    winit::window::Window::new(&eventloop)
}