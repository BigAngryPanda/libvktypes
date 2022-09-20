// glslangValidator -V examples/shaders/depth_buffer.frag -o examples/compiled_shaders/depth_buffer.frag.spv
#version 450

layout (location=0) flat in vec4 position;

layout (location=0) out vec4 color;

void main(){
    color = position;
}