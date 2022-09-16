// glslangValidator -V examples/shaders/vertex_input.vert -o examples/compiled_shaders/vertex_input.spv
#version 450

layout (location=0) in vec4 position;

layout (location=0) out vec4 colourdata;

void main() {
    gl_Position = position;
    colourdata = vec4(1.0, 1.0, 1.0, 1.0) - position;
}