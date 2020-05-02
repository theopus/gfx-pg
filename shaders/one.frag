#version 450
layout (location = 0) in vec2 frag_uv;
layout (location = 1) in vec3 light_intensity;

layout (location = 0) out vec4 color;

void main() {
  vec4 tex_color = vec4(light_intensity, 1.0) * vec4(1, 1, 1, 0);
  color = tex_color;
}