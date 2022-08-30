// glslangValidator -V tests/shaders/single_color.frag -o tests/compiled_shaders/single_color.spv
#version 450

layout (location=0) out vec4 color;

void main(){
    color = vec4(1.0,0.0,0.0,1.0);
}