#version 450
layout (push_constant) uniform PushConsts {
  mat4 mvp;
} push;

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 vert_uv;
layout (location = 2) in vec3 color;
//instanced
//layout (location = 3) in mat4 mvp;

layout (location = 0) out gl_PerVertex {
  vec4 gl_Position;
};
layout (location = 1) out vec3 frag_color;
layout (location = 2) out vec2 frag_uv;

void main() {
  gl_Position = push.mvp * vec4(position, 1.0);
  frag_color = color;
  frag_uv = vert_uv;
}
