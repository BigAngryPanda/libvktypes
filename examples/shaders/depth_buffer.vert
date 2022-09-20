// glslangValidator -V examples/shaders/depth_buffer.vert -o examples/compiled_shaders/depth_buffer.vert.spv
#version 450

layout (location=0) in vec4 position;

layout (location=0) out vec4 colourdata;

void main() {
    gl_Position = position;
    colourdata = vec4(1.0, 1.0, 1.0, 1.0) - position;
}