
use crate::render;
use crate::render::model;
use cgmath::{Vector3, Matrix4, Decomposed, Rotation3, Rad, Quaternion};

pub struct SunModel {
    sun: model::ModelKey,
}

const SIZE: f32 = 50.0;

impl SunModel {

    pub fn new(renderer: &mut render::Renderer) -> SunModel {
        SunModel {
            sun: SunModel::generate_sun(renderer),
        }
    }

    pub fn tick(&mut self, renderer: &mut render::Renderer, world_time: f64, _world_age: i64) {
        use std::f64::consts::PI;
        let time = 0.0;
        let ox = (time * PI).cos() * 300.0;
        let oy = (time * PI).sin() * 300.0;

        {
            let sun = renderer.model.get_model(self.sun).unwrap();
            sun.matrix[0] = Matrix4::from(Decomposed {
                scale: 1.0,
                rot: Quaternion::from_angle_z(Rad(-(time * PI) as f32)),
                disp: Vector3::new(
                    (renderer.camera.pos.x + ox) as f32,
                    -(renderer.camera.pos.y + oy) as f32,
                    renderer.camera.pos.z as f32,
                ),
            });
        }
    }

    pub fn remove(&mut self, renderer: &mut render::Renderer) {
        renderer.model.remove_model(self.sun);
    }

    pub fn generate_sun(renderer: &mut render::Renderer) -> model::ModelKey {
        let tex = render::Renderer::get_texture(renderer.get_textures_ref(), "environment/sun");
        renderer.model.create_model(
            model::SUN,
            vec![vec![
                model::Vertex{x: 0.0, y: -SIZE, z: -SIZE, texture_x: 0.0, texture_y: 1.0, texture: tex.clone(), r: 255, g: 255, b: 255, a: 0, id: 0},
                model::Vertex{x: 0.0, y: SIZE, z: -SIZE, texture_x: 0.0, texture_y: 0.0, texture: tex.clone(), r: 255, g: 255, b: 255, a: 0, id: 0},
                model::Vertex{x: 0.0, y: -SIZE, z: SIZE, texture_x: 1.0, texture_y: 1.0, texture: tex.clone(), r: 255, g: 255, b: 255, a: 0, id: 0},
                model::Vertex{x: 0.0, y: SIZE, z: SIZE, texture_x: 1.0, texture_y: 0.0, texture: tex.clone(), r: 255, g: 255, b: 255, a: 0, id: 0}
            ]]
        )
    }
}
