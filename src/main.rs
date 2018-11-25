// Copyright 2016 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![recursion_limit="300"]

extern crate steven_shared as shared;

#[macro_use]
pub mod macros;

pub mod ecs;
pub mod protocol;
pub mod format;
pub mod nbt;
pub mod item;
pub mod gl;
pub mod types;
pub mod resources;
pub mod render;
pub mod ui;
pub mod screen;
pub mod settings;
pub mod console;
pub mod server;
pub mod world;
pub mod chunk_builder;
pub mod model;
pub mod entity;

use std::sync::{Arc, RwLock};
use sdl2::Sdl;

pub struct Game {
    renderer: render::Renderer,
    should_close: bool,

    server: server::Server,
    focused: bool,


    sdl: Sdl,
}

impl Game {
    pub fn connect_to(&mut self, _address: &str) {
    }
}

fn main() {
    println!("Starting steven");

    let (res, _resui) = resources::Manager::new();
    let resource_manager = Arc::new(RwLock::new(res));

    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    let window = sdl2::video::WindowBuilder::new(&sdl_video, "Steven", 854, 480)
                            .opengl()
                            .resizable()
                            .build()
                            .expect("Could not create sdl window.");
    sdl2::hint::set_with_priority("SDL_MOUSE_RELATIVE_MODE_WARP", "1", &sdl2::hint::Hint::Override);
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_stencil_size(0);
    gl_attr.set_depth_size(24);
    gl_attr.set_context_major_version(3);
    gl_attr.set_context_minor_version(2);
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).expect("Could not set current context.");

    gl::init(&sdl_video);

    let vsync = true;
    sdl_video.gl_set_swap_interval(if vsync { 1 } else { 0 });


    let renderer = render::Renderer::new(resource_manager.clone());
    let mut ui_container = ui::Container::new();

    let mut game = Game {
        server: server::Server::dummy_server(resource_manager.clone()),
        focused: false,
        renderer,
        should_close: false,
        sdl,
    };
    game.renderer.camera.pos = cgmath::Point3::new(0.5, 13.2, 0.5);

    let mut events = game.sdl.event_pump().unwrap();
    while !game.should_close {

        let delta = 0f64;
        let (width, height) = window.size();

        game.server.tick(&mut game.renderer, delta);

        game.renderer.update_camera(width, height);
        game.server.world.compute_render_list(&mut game.renderer);

        game.renderer.camera.yaw = -7.2697720829739465;
        game.renderer.camera.pitch = 2.9733976253414633;
        game.renderer.camera.pos.x = -208.76533603647485;
        game.renderer.camera.pos.y = 65.62010000000001;
        game.renderer.camera.pos.z = 90.9279311085242;
 
        game.renderer.tick(&mut game.server.world, delta, width, height);

        window.gl_swap_window();

        for event in events.poll_iter() {
            handle_window_event(&window, &mut game, &mut ui_container, event);
        }
    }
}

fn handle_window_event(window: &sdl2::video::Window,
                       game: &mut Game,
                       ui_container: &mut ui::Container,
                       event: sdl2::event::Event) {
    use sdl2::event::Event;
    use sdl2::mouse::MouseButton;
    use std::f64::consts::PI;

    let mouse = window.subsystem().sdl().mouse();

    match event {
        Event::Quit{..} => game.should_close = true,

        Event::MouseMotion{x, y, xrel, yrel, ..} => {
            let (width, height) = window.size();
            if game.focused {
                if !mouse.relative_mouse_mode() {
                    mouse.set_relative_mouse_mode(true);
                }
                if let Some(player) = game.server.player {
                    let s = 2000.0 + 0.01;
                    let (rx, ry) = (xrel as f64 / s, yrel as f64 / s);
                    let rotation = game.server.entities.get_component_mut(player, game.server.rotation).unwrap();
                    rotation.yaw -= rx;
                    rotation.pitch -= ry;
                    if rotation.pitch < (PI/2.0) + 0.01 {
                        rotation.pitch = (PI/2.0) + 0.01;
                    }
                    if rotation.pitch > (PI/2.0)*3.0 - 0.01 {
                        rotation.pitch = (PI/2.0)*3.0 - 0.01;
                    }
                }
            } else {
                if mouse.relative_mouse_mode() {
                    mouse.set_relative_mouse_mode(false);
                }
                ui_container.hover_at(game, x as f64, y as f64, width as f64, height as f64);
            }
        }
        Event::MouseButtonUp{mouse_btn: MouseButton::Left, x, y, ..} => {
            let (width, height) = window.size();

            if !game.focused {
                if mouse.relative_mouse_mode() {
                    mouse.set_relative_mouse_mode(false);
                }
                ui_container.click_at(game, x as f64, y as f64, width as f64, height as f64);
            }
        }
        Event::MouseButtonDown{mouse_btn: MouseButton::Right, ..} => {
            if game.focused {
                game.server.on_right_click(&mut game.renderer);
            }
        }
        Event::TextInput{text, ..} => {
            if !game.focused {
                for c in text.chars() {
                    ui_container.key_type(game, c);
                }
            }
        }
        _ => (),
    }
}
