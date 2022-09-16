// glslangValidator -V examples/shaders/color_from_vertex.frag -o examples/compiled_shaders/color_from_vertex.spv
#version 450

layout (location=0) in vec4 position;

layout (location=0) out vec4 color;

void main(){
    color = position;
}