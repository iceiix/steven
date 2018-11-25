
use crate::render;
use crate::render::model;
use cgmath::{Vector3, Matrix4, Decomposed, Rotation3, Rad, Quaternion};

pub struct SunModel {
    moon: model::ModelKey,
    last_phase: i32,
}

const SIZE: f32 = 50.0;

impl SunModel {

    pub fn new(renderer: &mut render::Renderer) -> SunModel {
        SunModel {
            moon: SunModel::generate_moon(renderer, 0),
            last_phase: 0,
        }
    }

    pub fn tick(&mut self, renderer: &mut render::Renderer, world_time: f64, world_age: i64) {
        use std::f64::consts::PI;
        let phase = ((world_age / 24000) % 8) as i32;
        if phase != self.last_phase {
            renderer.model.remove_model(self.moon);
            self.moon = SunModel::generate_moon(renderer, phase);
            self.last_phase = phase;
        }

        let time = world_time / 12000.0;
        let ox = (time * PI).cos() * 300.0;
        let oy = (time * PI).sin() * 300.0;

        {
            let moon = renderer.model.get_model(self.moon).unwrap();
            moon.matrix[0] = Matrix4::from(Decomposed {
                scale: 1.0,
                rot: Quaternion::from_angle_z(Rad((PI - (time * PI)) as f32)),
                disp: Vector3::new(
                    (renderer.camera.pos.x - ox) as f32,
                    -(renderer.camera.pos.y - oy) as f32,
                    renderer.camera.pos.z as f32,
                ),
            });
        }
    }

    pub fn remove(&mut self, renderer: &mut render::Renderer) {
        renderer.model.remove_model(self.moon);
    }

    pub fn generate_moon(renderer: &mut render::Renderer, phase: i32) -> model::ModelKey {
        let tex = render::Renderer::get_texture(renderer.get_textures_ref(), "environment/moon_phases");
        let mpx = (phase % 4) as f64 * (1.0 / 4.0);
        let mpy = (phase / 4) as f64 * (1.0 / 2.0);
        renderer.model.create_model(
            model::SUN,
            vec![vec![
                model::Vertex{x: 0.0, y: -SIZE, z: -SIZE, texture_x: mpx, texture_y: mpy + (1.0 / 2.0), texture: tex.clone(), r: 255, g: 255, b: 255, a: 0, id: 0},
                model::Vertex{x: 0.0, y: SIZE, z: -SIZE, texture_x: mpx, texture_y: mpy, texture: tex.clone(), r: 255, g: 255, b: 255, a: 0, id: 0},
                model::Vertex{x: 0.0, y: -SIZE, z: SIZE, texture_x: mpx + (1.0 / 4.0), texture_y: mpy + (1.0 / 2.0), texture: tex.clone(), r: 255, g: 255, b: 255, a: 0, id: 0},
                model::Vertex{x: 0.0, y: SIZE, z: SIZE, texture_x: mpx + (1.0 / 4.0), texture_y: mpy, texture: tex.clone(), r: 255, g: 255, b: 255, a: 0, id: 0}
            ]]
        )
    }
}
