#![allow(dead_code)]
use libvktypes::{libvk, dev, extensions, hw, layers, surface, window, swapchain};

use std::sync::Once;
use std::mem::MaybeUninit;

static INIT_WINDOW: Once = Once::new();

static mut WINDOW: MaybeUninit<window::Window> = MaybeUninit::<window::Window>::uninit();

static INIT_GRAPHICS_INSTANCE: Once = Once::new();

static mut GRAPHICS_INSTANCE: MaybeUninit<libvk::Instance> = MaybeUninit::<libvk::Instance>::uninit();

static INIT_SURFACE: Once = Once::new();

static mut SURFACE: MaybeUninit<surface::Surface> = MaybeUninit::<surface::Surface>::uninit();

static INIT_GRAPHICS_HW: Once = Once::new();

static mut GRAPHICS_HW: MaybeUninit<hw::HWDevice> = MaybeUninit::<hw::HWDevice>::uninit();

static mut GRAPHICS_QUEUE: MaybeUninit<hw::QueueFamilyDescription> = MaybeUninit::<hw::QueueFamilyDescription>::uninit();

static INIT_PRESENT_QUEUE: Once = Once::new();

static mut PRESENT_QUEUE: MaybeUninit<hw::QueueFamilyDescription> = MaybeUninit::<hw::QueueFamilyDescription>::uninit();

static INIT_SURFACE_CAP: Once = Once::new();

static mut SURFACE_CAP: MaybeUninit<surface::Capabilities> = MaybeUninit::<surface::Capabilities>::uninit();

static INIT_GRAPHICS_DEV: Once = Once::new();

static mut GRAPHICS_DEV: MaybeUninit<dev::Device> = MaybeUninit::<dev::Device>::uninit();

static INIT_SWAPCHAIN: Once = Once::new();

static mut SWAPCHAIN: MaybeUninit<swapchain::Swapchain> = MaybeUninit::<swapchain::Swapchain>::uninit();

pub fn get_window() -> &'static window::Window {
    unsafe {
        INIT_WINDOW.call_once(|| {
            WINDOW.write(window::Window::new().expect("Failed to create window"));
        });

        WINDOW.assume_init_ref()
    }
}

#[cfg(target_os = "linux")]
pub fn get_graphics_instance() -> &'static libvk::Instance {
    unsafe {
        INIT_GRAPHICS_INSTANCE.call_once(|| {
            let lib_type = libvk::InstanceType {
                debug_layer: Some(layers::DebugLayer::default()),
                extensions: &[
                    extensions::DEBUG_EXT_NAME,
                    extensions::SURFACE_EXT_NAME,
                    extensions::XLIB_SURFACE_EXT_NAME,
                ],
                ..libvk::InstanceType::default()
            };

            GRAPHICS_INSTANCE.write(libvk::Instance::new(&lib_type).expect("Failed to init graphic instance"));
        });

        GRAPHICS_INSTANCE.assume_init_ref()
    }
}

pub fn get_surface() -> &'static surface::Surface {
    unsafe {
        INIT_SURFACE.call_once(|| {
            let surface_cfg = surface::SurfaceType {
                lib: get_graphics_instance(),
                window: get_window(),
            };

            SURFACE.write(surface::Surface::new(&surface_cfg).expect("Failed to create surface"));
        });

        SURFACE.assume_init_ref()
    }
}

pub fn get_graphics_hw() -> &'static hw::HWDevice {
    unsafe {
        INIT_GRAPHICS_HW.call_once(|| {
            let hw_list = hw::Description::poll(get_graphics_instance()).expect("Failed to list hardware");

            let (hw_dev, qf, _) = hw_list
                .find_first(
                    hw::HWDevice::is_discrete_gpu,
                    hw::QueueFamilyDescription::is_graphics,
                    |_| true,
                )
                .expect("Failed to find suitable hardware device");

            GRAPHICS_HW.write(hw_dev.clone());
            GRAPHICS_QUEUE.write(*qf);
        });

        GRAPHICS_HW.assume_init_ref()
    }
}

pub fn get_graphics_queue() -> &'static hw::QueueFamilyDescription {
    get_graphics_hw();

    unsafe { GRAPHICS_QUEUE.assume_init_ref() }
}

pub fn get_present_queue() -> &'static hw::QueueFamilyDescription {
    unsafe {
        INIT_PRESENT_QUEUE.call_once(|| {
            let hw = get_graphics_hw();
            let surface = get_surface();

            let present_queue = hw.find_first_queue(|q| q.support_surface(hw, surface));

            PRESENT_QUEUE.write(*present_queue.expect("Failed to find queue family with presentation capabilities"));
        });

        PRESENT_QUEUE.assume_init_ref()
    }
}

pub fn get_surface_capabilities() -> &'static surface::Capabilities {
    unsafe {
        INIT_SURFACE_CAP.call_once(|| {
            let cap_type = surface::CapabilitiesType {
                hw: get_graphics_hw(),
                surface: get_surface(),
            };

            SURFACE_CAP.write(surface::Capabilities::get(&cap_type).expect("Failed to query capabilities"));
        });

        SURFACE_CAP.assume_init_ref()
    }
}

pub fn get_graphics_device() -> &'static dev::Device {
    unsafe {
        INIT_GRAPHICS_DEV.call_once(|| {
            let dev_type = dev::DeviceType {
                lib: get_graphics_instance(),
                hw: get_graphics_hw(),
                queue_family_index: get_graphics_queue().index(),
                priorities: &[1.0_f32],
                extensions: &[extensions::SWAPCHAIN_EXT_NAME],
            };

            GRAPHICS_DEV.write(dev::Device::new(&dev_type).expect("Failed to create device"));
        });

        GRAPHICS_DEV.assume_init_ref()
    }
}

pub fn get_swapchain() -> &'static swapchain::Swapchain {
    unsafe {
        INIT_SWAPCHAIN.call_once(|| {
            let lib_ref = get_graphics_instance();

            let surface_ref = get_surface();

            let device = get_graphics_device();

            // Look at tests/swapchain.rs to understand why we have to do this
            let _ = get_present_queue();

            let capabilities = get_surface_capabilities();

            let swp_type = swapchain::SwapchainType {
                lib: lib_ref,
                dev: device,
                surface: surface_ref,
                num_of_images: 2,
                format: capabilities.formats().next().expect("No available formats").format,
                color: capabilities.formats().next().expect("No available formats").color_space,
                present_mode: *capabilities.modes().next().expect("No available modes"),
                flags: surface::UsageFlags::COLOR_ATTACHMENT,
                extent: capabilities.extent2d(),
                transform: capabilities.pre_transformation(),
                alpha: capabilities.alpha_composition(),
            };

            SWAPCHAIN.write(swapchain::Swapchain::new(&swp_type).expect("Failed to create swapchain"));
        });

        SWAPCHAIN.assume_init_ref()
    }
}