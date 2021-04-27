#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::glm;

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
    let ndc_v2 = screen2ndc(coords.0, coords.1, dimensions.0 as f32, dimensions.1 as f32);
    let clip = glm::vec4(ndc_v2.x, ndc_v2.y, 0., 1.);
    let eye = clip2eye(&clip, proj);
    eye2world(&eye, view)
}

pub fn intersection(plane_n: &glm::Vec3, _plane_p: &glm::Vec3, line_vec: &glm::Vec3, line_p: &glm::Vec3) -> Option<glm::Vec3> {
    let d = glm::dot(plane_n, plane_n);

    if glm::dot(plane_n, line_vec) == 0. {
        return None;
    }

    let x = (d - glm::dot(plane_n, line_p)) / glm::dot(plane_n, line_vec);
    Some(line_p + glm::normalize(&line_vec) * x)
}