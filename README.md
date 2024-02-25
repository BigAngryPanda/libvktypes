![1x-badge](https://img.shields.io/github/stars/cutenode/1x.engineer.svg?color=purple&label=1x%20Engineers&logo=image%2Fpng%3Bbase64%2CiVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAADAElEQVRoQ%2B1YPZMNQRQ9RyYiUCVkI0J%2BASUi4xfYzYjsVsmXX2BFZFauigyJ4hcQEtkNVQmoEjvqqh7VM6%2B%2FZ%2BapqXodvtdz%2B56%2B59zT3cTCBxeePzYA%2FncFNxUoqYCkEwC2AbwjeVTyTemctVRA0iUAH11SxwBekLxfmmRq3roA3AHwxEvkLcnrSwJwCOC2l%2FBDkg%2BWBOAzgAtewjdIvlkEAElnAHwDYEK28RvAWZLflwLAuP7aS%2FYLyYtTJG8xZhexJOP6vpfwc5LWUicZ6wDwHsAVL9u7JJ9Okv2YCki6SfJVKhFnYL8AnPTmXSb5yf%2BuJFZsnaYKSHoEYBfAIcmdWPCBgdk0E64J2IT8d0h65lz6gORebWWqAEg6DeAlgKveQjskrc%2BvDElJA5NkWjAA3TC63SL5oxRIMQC3m5b8%2BUHwn%2FZbaFFJUQNzm2HnolODePabgejRbBSF3E4ZbawC%2FrBzjWkhuJikpIG5TTEdnRvEtQrsxSrrz81WwOP7cBM%2BuOSD5S41MFcJA%2BF3qm6trC6iACJ87wI%2FJmkijg5JVQYm6QDAvUDApC6CADJ83y0pbYuBOaoakGJdrACwnuw6QxXfhzsnqcnAMrqwjtfznhCAr4FOk%2BR7IHk7uGUNLMa%2FhC6OSG4lRSzJ2tiwK1T15xIDy%2Bgn5Df2yTHJXhuPUcj6dzEPAxVovoFl9LedpZAlM7Y%2Fpwwss%2FPmzFV%2Bk2ujTf05Z2AhEK1%2BU2JkVf251MA6EGP9JgvAUcpKW9SfawxsCr8pAlCgi63uMFdqYG7nrWWP8ptiAA6ELTbURe84XWNggeN0ld9YTlUAPN52uujdb0tvYL6IvY6VPV%2BFxN8EwFVj5UrZamBrv1ImjgDNBpbyh9R%2FzRWI9PLZnhBjIKYGMNsT4uwAag2slTLD7yarQI2BTZV8cxuN8H%2FWJ8R1UKjpBja2GlNSyMzNrqPdZWjlCXFsspMaWcIL7MZ0zT07%2FntCnCP5STUwV4K5uJNRKLfQXP9vAMy1s6VxF1%2BBPxWSokDSvlDHAAAAAElFTkSuQmCC&style=for-the-badge&link=https://1x.engineer&link=https://github.com/cutenode/1x.engineer/stargazers)

# libvktypes

Vulkan based library which aims to make interaction with graphics API easier

However library does not perform any extra validation so it is your responsibility to use it correctly

## Overview

Library provides wrapper around Vulkan API to make work easier

`libvktypes` uses [ash](https://github.com/ash-rs/ash) as bindings to the raw Vulkan API

General idea as in Vulkan is creating functional objects by providing related configuration structs

## Using in project

As library in alpha stage only git is available

```
[dependencies]
libvktypes = { git = "https://github.com/BigAngryPanda/libvktypes", branch = "main"}
```

## Tests

```
cargo test (module name) (-- --nocapture)
```

Example
```
cargo test hw -- --nocapture
```

## Docs

```
cargo doc --no-deps
```

```--no-deps``` is optional (ignore it if you want generate docs for underlying crates)

## Dependencies

Library uses `ash` for bindings

`winit` as window system

`shaderc` as `glsl` compiler

While direct dependencies desctibed in [Cargo.toml](Cargo.toml)
they have their own dependencies as well

See more:

- [ash](https://github.com/ash-rs/ash)
- [winit](https://github.com/rust-windowing/winit)
- [shaderc](https://github.com/google/shaderc-rs)

Example for Linux Mint 21 (amd64/nvidia) packets for apt:

```
libvulkan-dev
libvulkan1
vulkan-validationlayers
vulkan-validationlayers-dev
g++
cmake
libfontconfig
libfontconfig-dev
build-essential
```

Also maybe you will need following packages
```
vulkan-icd
```

(virtual package which maps to `mesa-vulkan-drivers`)

## Platforms

Theoretically following platforms are supported:

- Windows (not tested)
- MacOS, iOS (not tested)
- Linux: X11 (tested), Wayland (not tested)
- Android (not tested)

## Examples

Examples provide gentle introduction how to use library

From simple to more sophisticated

### `single_shader_triangle`

Basic example with bare minimum from library loading to displaying image

Note: examples aim to show you how to work with library not the Vulkan API, graphics itself, math etc.

You will learn how to:

1. Load library
2. Pick physical device
3. Create
	1. logical device
	2. surface and swapchain
	3. shader modules (include shaders compiling)
	4. framebuffer
	5. render pass
	6. pipeline
4. Allocate command pool and command buffer
5. Control flow with `winit` library
6. Execute pipeline

### `vertex_buffer`

You will learn how to:

1. Allocate, fill and bind vertex memory
2. Pass information about vertex struct to the pipeline

### `depth_buffer`

Shows how to create depth buffer and use it

### `two_triangles`

Example of usage geometry shader

### `uniform`

How to use descriptors for binding resources for shaders

In this example we use uniform buffers

### `texture`

How to create and use:
1. Samplers
2. Images as texture buffers

### `cube`

Complex example with matrix transformations

Shows how to add animation and organize render loop

## Contributing

Feel free to fork/create pull request/discussion and so on

All help are appreciated

## License

BSD 3-Clause