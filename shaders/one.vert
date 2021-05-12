#version 450
//layout (push_constant) uniform PushConsts {
//    mat4 view;
//} push;

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 vert_uv;
layout (location = 2) in vec3 normal;
//mvp
layout (location = 3) in vec4 mvp0;
layout (location = 4) in vec4 mvp1;
layout (location = 5) in vec4 mvp2;
layout (location = 6) in vec4 mvp3;
//model
layout (location = 7) in vec4 model0;
layout (location = 8) in vec4 model1;
layout (location = 9) in vec4 model2;
layout (location = 10) in vec4 model3;

layout (location = 0) out gl_PerVertex {
    vec4 gl_Position;
};
layout (location = 0) out vec2 frag_uv;
layout (location = 1) out vec3 light_intensity;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 view;
    vec4 light_position;
    vec4 light_intensity_1;
};


//const vec3 l_position = vec3(0, 100, 30);
//const vec3 intensity = vec3(1, 1, 1);

void main() {
    mat4 mvp = mat4 (
           mvp0,
           mvp1,
           mvp2,
           mvp3
    );
    mat4 model = mat4 (
           model0,
           model1,
           model2,
           model3
    );

    vec4 world_coords = model * vec4(position, 1.0);
    //vec4 camera_coords = push.view * world_coords;
    vec3 surface_normal = normalize((model * vec4(normal, 0.0)).xyz);
    vec3 to_light_vector = normalize(light_position.xyz - world_coords.xyz);
    light_intensity = light_intensity_1.xyz * max(dot(to_light_vector, surface_normal), 0.2);

    gl_Position = mvp * vec4(position, 1.0);
    frag_uv = vert_uv;
}
