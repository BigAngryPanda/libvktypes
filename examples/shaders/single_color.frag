// glslangValidator -V examples/shaders/single_color.frag -o examples/compiled_shaders/single_color.spv
#version 450

layout (location=0) out vec4 color;

void main(){
    color = vec4(0.5,0.5,0.5,0.0);
}