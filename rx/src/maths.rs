#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

fn screen2ndc(x: f32, y: f32, w: f32, h: f32) -> glm::Vec2 {
    let x = (2. * x) / w - 1.;
    let y = (2. * y) / h - 1.;
    glm::vec2(x, y)
}

fn clip2eye(clip: &glm::Vec4, projection: &glm::Mat4) -> glm::Vec4 {
    let inverted = glm::inverse(projection);
    let eye: glm::Vec4 = &inverted * clip;
    glm::vec4(eye.x, eye.y, -1., 0.)
}

fn eye2world(eye: &glm::Vec4, view: &glm::Mat4) -> glm::Vec3 {
    let inverted = glm::inverse(view);
    let ray_world = &inverted * eye;
    glm::normalize(&glm::vec4_to_vec3(&ray_world))
}

pub fn screen2world(
    coords: (f32, f32),
    dimensions: (u32, u32),
    view: &glm::Mat4,
    proj: &glm::Mat4,
) -> glm::Vec3 {
    //Note: for top left coordinate system
    let y = (dimensions.1 as f32 - coords.1).abs();
    let ndc_v2 = screen2ndc(coords.0, y, dimensions.0 as f32, dimensions.1 as f32);
    let clip = glm::vec4(ndc_v2.x, ndc_v2.y, 0., 1.);
    let eye = clip2eye(&clip, proj);
    eye2world(&eye, view)
}

pub const F32_PRECISION: f32 = 0.000001;

//maybe bug [plane_p]!
pub fn intersection(plane_n: &glm::Vec3, plane_p: &glm::Vec3, line_vec: &glm::Vec3, line_p: &glm::Vec3) -> Option<glm::Vec3> {
    let d = glm::dot(plane_n, plane_p);

    if glm::dot(plane_n, line_vec) == 0. {
        return None;
    }
    let x = (d - glm::dot(plane_n, line_p)) / glm::dot(plane_n, line_vec);
    Some(line_p + glm::normalize(&line_vec) * x)
}

pub fn frustum_planes(vp: &glm::Mat4) -> [glm::Vec4; 6]{
    let mut vec0 = glm::vec4(0.,0.,0.,0.);
    vec0.x = vp.m41 + vp.m11;
    vec0.y = vp.m42 + vp.m12;
    vec0.z = vp.m43 + vp.m13;
    vec0.w = vp.m44 + vp.m14;
    let mut vec1 = glm::vec4(0.,0.,0.,0.);
    vec1.x = vp.m41 - vp.m11;
    vec1.y = vp.m42 - vp.m12;
    vec1.z = vp.m43 - vp.m13;
    vec1.w = vp.m44 - vp.m14;
    let mut vec2 = glm::vec4(0., 0., 0., 0.);
    vec2.x = vp.m41 - vp.m21;
    vec2.y = vp.m42 - vp.m22;
    vec2.z = vp.m43 - vp.m23;
    vec2.w = vp.m44 - vp.m24;
    let mut vec3 = glm::vec4(0., 0., 0., 0.);
    vec3.x = vp.m41 + vp.m21;
    vec3.y = vp.m42 + vp.m22;
    vec3.z = vp.m43 + vp.m23;
    vec3.w = vp.m44 + vp.m24;
    let mut vec4 = glm::vec4(0., 0., 0., 0.);
    vec4.x = vp.m41 + vp.m31;
    vec4.y = vp.m42 + vp.m32;
    vec4.z = vp.m43 + vp.m33;
    vec4.w = vp.m44 + vp.m34;
    let mut vec5 = glm::vec4(0., 0., 0., 0.);
    vec5.x = vp.m41 - vp.m31;
    vec5.y = vp.m42 - vp.m32;
    vec5.z = vp.m43 - vp.m33;
    vec5.w = vp.m44 - vp.m34;
    [vec0, vec1, vec2, vec3, vec4, vec5]
}