//! Wrapper around native window

use crate::on_error_ret;

use winit::event_loop::EventLoopBuilder;
use winit::platform::unix::EventLoopBuilderExtUnix;

pub type EventLoop = winit::event_loop::EventLoop<()>;

#[derive(Debug)]
pub enum WindowError {
    Initialization,
}

pub struct Window {
    i_window: winit::window::Window,
    i_evloop: EventLoop,
}

impl Window {
    pub fn new() -> Result<Window, WindowError> {
        let eventloop = EventLoopBuilder::new().with_x11().with_any_thread(true).build();

        Ok(
            Window {
                i_window: on_error_ret!(
                    winit::window::Window::new(&eventloop),
                    WindowError::Initialization),
                i_evloop: eventloop,
            }
        )
    }

    /// Return reference to internal event loop
    ///
    #[doc = "See more <https://docs.rs/winit/latest/winit/window/struct.Window.html>"]
    pub fn event_loop(&mut self) -> &mut EventLoop {
        &mut self.i_evloop
    }

    #[doc(hidden)]
    pub fn window(&self) -> &winit::window::Window {
        &self.i_window
    }
}