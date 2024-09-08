//! Helper functions around `winit` library

use winit::window::WindowBuilder;
use winit::event_loop::EventLoopBuilder;

#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;
#[cfg(target_os = "linux")]
use winit::platform::wayland::EventLoopBuilderExtWayland;

#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;

use std::fmt;

pub type EventLoop = winit::event_loop::EventLoop<()>;
pub type Window = winit::window::Window;

#[derive(Debug)]
pub enum WindowError {
    EventLoop,
    Window
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            WindowError::EventLoop => {
                "Failed to create eventloop (winit error)"
            },
            WindowError::Window => {
                "Failed to create window (winit error)"
            },
        };

        write!(f, "{:?}", err_msg)
    }
}

#[cfg(target_os = "linux")]
/// Create new eventloop
///
/// Event loop can be used in different thread (unlike original winit event loop)
pub fn eventloop() -> Result<winit::event_loop::EventLoop<()>, WindowError> {
    let mut builder = EventLoopBuilder::new();
    EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);

    let result = EventLoopBuilderExtX11::with_any_thread(&mut builder, true).build();

    match result {
        Ok(result) => Ok(result),
        Err(_) => Err(WindowError::EventLoop)
    }
}

#[cfg(target_os = "windows")]
/// Create new eventloop
///
/// Event loop can be used in different thread (unlike original winit event loop)
pub fn eventloop() -> Result<winit::event_loop::EventLoop<()>, WindowError> {
    let mut builder = EventLoopBuilder::new();
    let result = EventLoopBuilderExtWindows::with_any_thread(&mut builder, true).build();

    match result {
        Ok(result) => Ok(result),
        Err(_) => Err(WindowError::EventLoop)
    }
}

pub fn create_window(eventloop: &EventLoop) -> Result<Window, WindowError> {
    match WindowBuilder::new().build(&eventloop) {
        Ok(result) => Ok(result),
        Err(_) => Err(WindowError::Window)
    }
}