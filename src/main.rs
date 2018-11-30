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

use std::time::{Instant, Duration};
use log::info;
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
pub mod auth;
pub mod model;
pub mod entity;

use std::sync::{Arc, RwLock, Mutex};
use std::rc::Rc;
use std::marker::PhantomData;
use std::thread;
use std::sync::mpsc;
use crate::protocol::mojang;
use glutin;
use glutin::GlContext;

const CL_BRAND: console::CVar<String> = console::CVar {
    ty: PhantomData,
    name: "cl_brand",
    description: "cl_brand has the value of the clients current 'brand'. e.g. \"Steven\" or \
                  \"Vanilla\"",
    mutable: false,
    serializable: false,
    default: &|| "Steven".to_owned(),
};

pub struct Game {
    renderer: render::Renderer,
    screen_sys: screen::ScreenSystem,
    resource_manager: Arc<RwLock<resources::Manager>>,
    console: Arc<Mutex<console::Console>>,
    vars: Rc<console::Vars>,
    should_close: bool,

    server: server::Server,
    focused: bool,
    chunk_builder: chunk_builder::ChunkBuilder,

    connect_reply: Option<mpsc::Receiver<Result<server::Server, protocol::Error>>>,
    dpi_factor: f64,
}

impl Game {
    pub fn connect_to(&mut self, address: &str) {
        let (tx, rx) = mpsc::channel();
        self.connect_reply = Some(rx);
        let address = address.to_owned();
        let resources = self.resource_manager.clone();
        let profile = mojang::Profile {
            username: self.vars.get(auth::CL_USERNAME).clone(),
            id: self.vars.get(auth::CL_UUID).clone(),
            access_token: self.vars.get(auth::AUTH_TOKEN).clone(),
        };
        thread::spawn(move || {
            tx.send(server::Server::connect(resources, profile, &address)).unwrap();
        });
    }

    pub fn tick(&mut self, delta: f64) {
        if !self.server.is_connected() {
            self.renderer.camera.yaw += 0.005 * delta;
            if self.renderer.camera.yaw > ::std::f64::consts::PI * 2.0 {
                self.renderer.camera.yaw = 0.0;
            }
        }

        if let Some(disconnect_reason) = self.server.disconnect_reason.take() {
            self.screen_sys.replace_screen(Box::new(screen::ServerList::new(
                Some(disconnect_reason)
            )));
        }
        if !self.server.is_connected() {
            self.focused = false;
        }

        let mut clear_reply = false;
        if let Some(ref recv) = self.connect_reply {
            if let Ok(server) = recv.try_recv() {
                clear_reply = true;
                match server {
                    Ok(val) => {
                        self.screen_sys.pop_screen();
                        self.focused = true;
                        self.server.remove(&mut self.renderer);
                        self.server = val;
                    },
                    Err(err) => {
                        let msg = match err {
                            protocol::Error::Disconnect(val) => val,
                            err => {
                                let mut msg = format::TextComponent::new(&format!("{}", err));
                                msg.modifier.color = Some(format::Color::Red);
                                format::Component::Text(msg)
                            },
                        };
                        self.screen_sys.replace_screen(Box::new(screen::ServerList::new(
                            Some(msg)
                        )));
                    }
                }
            }
        }
        if clear_reply {
            self.connect_reply = None;
        }
    }
}

fn main() {
    let con = Arc::new(Mutex::new(console::Console::new()));
    let (vars, mut vsync) = {
        let mut vars = console::Vars::new();
        vars.register(CL_BRAND);
        auth::register_vars(&mut vars);
        settings::register_vars(&mut vars);
        vars.load_config();
        vars.save_config();
        let vsync = *vars.get(settings::R_VSYNC);
        (Rc::new(vars), vsync)
    };

    let proxy = console::ConsoleProxy::new(con.clone());

    log::set_boxed_logger(Box::new(proxy)).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    info!("Starting steven");

    let (res, mut resui) = resources::Manager::new();
    let resource_manager = Arc::new(RwLock::new(res));

    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Steven")
        .with_dimensions(glutin::dpi::LogicalSize::new(854.0, 480.0));
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let mut window = glutin::GlWindow::new(window_builder, context, &events_loop)
        .expect("Could not create glutin window.");

    unsafe {
        window.make_current().expect("Could not set current context.");
    }

    /* TODO
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_stencil_size(0);
    gl_attr.set_depth_size(24);
    gl_attr.set_context_major_version(3);
    gl_attr.set_context_minor_version(2);
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    */

    gl::init(&window);

    //TODO sdl_video.gl_set_swap_interval(if vsync { 1 } else { 0 });


    let renderer = render::Renderer::new(resource_manager.clone());
    let mut ui_container = ui::Container::new();

    let mut last_frame = Instant::now();
    let frame_time = 1e9f64 / 60.0;

    let mut screen_sys = screen::ScreenSystem::new();
    screen_sys.add_screen(Box::new(screen::Login::new(vars.clone())));

    let textures = renderer.get_textures();
    let dpi_factor = window.get_current_monitor().get_hidpi_factor();
    let mut game = Game {
        server: server::Server::dummy_server(resource_manager.clone()),
        focused: false,
        renderer,
        screen_sys,
        resource_manager: resource_manager.clone(),
        console: con,
        vars,
        should_close: false,
        chunk_builder: chunk_builder::ChunkBuilder::new(resource_manager, textures),
        connect_reply: None,
        dpi_factor,
    };
    game.renderer.camera.pos = cgmath::Point3::new(0.5, 13.2, 0.5);

    while !game.should_close {

        let now = Instant::now();
        let diff = now.duration_since(last_frame);
        last_frame = now;
        let delta = (diff.subsec_nanos() as f64) / frame_time;
        let (width, height) = window.get_inner_size().unwrap().into();
        let (physical_width, physical_height) = window.get_inner_size().unwrap().to_physical(game.dpi_factor).into();

        let version = {
            let mut res = game.resource_manager.write().unwrap();
            res.tick(&mut resui, &mut ui_container, delta);
            res.version()
        };

        let vsync_changed = *game.vars.get(settings::R_VSYNC);
        if vsync != vsync_changed {
            vsync = vsync_changed;
            //TODO sdl_video.gl_set_swap_interval(if vsync { 1 } else { 0 });
        }
        let fps_cap = *game.vars.get(settings::R_MAX_FPS);

        game.tick(delta);
        game.server.tick(&mut game.renderer, delta);

        game.renderer.update_camera(physical_width, physical_height);
        game.server.world.compute_render_list(&mut game.renderer);
        game.chunk_builder.tick(&mut game.server.world, &mut game.renderer, version);

        game.screen_sys.tick(delta, &mut game.renderer, &mut ui_container);
        game.console
            .lock()
            .unwrap()
            .tick(&mut ui_container, &game.renderer, delta, width as f64);
        ui_container.tick(&mut game.renderer, delta, width as f64, height as f64);
        game.renderer.tick(&mut game.server.world, delta, width, height);


        if fps_cap > 0 && !vsync {
            let frame_time = now.elapsed();
            let sleep_interval = Duration::from_millis(1000 / fps_cap as u64);
            if frame_time < sleep_interval {
                thread::sleep(sleep_interval - frame_time);
            }
        }
        window.swap_buffers().expect("Failed to swap GL buffers");

        events_loop.poll_events(|event| {
            handle_window_event(&mut window, &mut game, &mut ui_container, event);
        });
    }
}

fn handle_window_event(window: &mut glutin::GlWindow,
                       game: &mut Game,
                       ui_container: &mut ui::Container,
                       event: glutin::Event) {
    match event {
        glutin::Event::WindowEvent{event, ..} => match event {
            glutin::WindowEvent::CloseRequested => game.should_close = true,
            glutin::WindowEvent::Resized(logical_size) => {
                game.dpi_factor = window.get_hidpi_factor();
                window.resize(logical_size.to_physical(game.dpi_factor));
            },

            glutin::WindowEvent::MouseInput{device_id, state, button, modifiers} => {
                println!("MouseInput {:?} {:?} {:?} {:?}", device_id, state, button, modifiers);
                match state {
                    glutin::ElementState::Released => {
                        // TODO: get x, y
                        /* TODO
                        Event::MouseButtonUp{mouse_btn: MouseButton::Left, x, y, ..} => {
                            let (width, height) = window.size();

                            if game.server.is_connected() && !game.focused && !game.screen_sys.is_current_closable() {
                                game.focused = true;
                                if !mouse.relative_mouse_mode() {
                                    mouse.set_relative_mouse_mode(true);
                                }
                                return;
                            }
                            if !game.focused {
                                if mouse.relative_mouse_mode() {
                                    mouse.set_relative_mouse_mode(false);
                                }
                                ui_container.click_at(game, x as f64, y as f64, width as f64, height as f64);
                            }
                        }
                        */
                    },
                    glutin::ElementState::Pressed => {
                        if button == glutin::MouseButton::Right {
                            if game.focused {
                                game.server.on_right_click(&mut game.renderer);
                            }
                        }
                    },
                }
            },
            glutin::WindowEvent::CursorMoved{device_id: _, position, modifiers: _} => {
                let (x, y) = position.into();

                if !game.focused {
                    let (width, height) = window.get_inner_size().unwrap().into();
                    ui_container.hover_at(game, x, y, width, height);
                }
            },
            _ => ()
        },

        glutin::Event::DeviceEvent{event, ..} => match event {
            glutin::DeviceEvent::MouseMotion{delta:(xrel, yrel)} => {
                use std::f64::consts::PI;

                if game.focused {
                    /* TODO
                    if !mouse.relative_mouse_mode() {
                        mouse.set_relative_mouse_mode(true);
                    }
                    */
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
                    /* TODO
                    if mouse.relative_mouse_mode() {
                        mouse.set_relative_mouse_mode(false);
                    }
                    */
                }
            },
            _ => ()
        },

        /* TODO
        Event::MouseWheel{x, y, ..} => {
            game.screen_sys.on_scroll(x as f64, y as f64);
        }
        Event::KeyUp{keycode: Some(Keycode::Escape), ..} => {
            if game.focused {
                mouse.set_relative_mouse_mode(false);
                game.focused = false;
                game.screen_sys.replace_screen(Box::new(screen::SettingsMenu::new(game.vars.clone(), true)));
            } else if game.screen_sys.is_current_closable() {
                mouse.set_relative_mouse_mode(true);
                game.focused = true;
                game.screen_sys.pop_screen();
            }
        }
        Event::KeyDown{keycode: Some(Keycode::Backquote), ..} => {
            game.console.lock().unwrap().toggle();
        }
        Event::KeyDown{keycode: Some(Keycode::F11), ..} => { // TODO: configurable binding in settings::Stevenkey
            let state = match window.fullscreen_state() {
                sdl2::video::FullscreenType::Off => sdl2::video::FullscreenType::Desktop,
                sdl2::video::FullscreenType::True => sdl2::video::FullscreenType::Off,
                sdl2::video::FullscreenType::Desktop => sdl2::video::FullscreenType::Off,
            };

            window.set_fullscreen(state).expect(&format!("failed to set fullscreen to {:?}", state));
        }
        Event::KeyDown{keycode: Some(key), keymod, ..} => {
            if game.focused {
                if let Some(steven_key) = settings::Stevenkey::get_by_keycode(key, &game.vars) {
                    game.server.key_press(true, steven_key);
                }
            } else {
                let ctrl_pressed = keymod.intersects(keyboard::LCTRLMOD | keyboard::RCTRLMOD);
                ui_container.key_press(game, key, true, ctrl_pressed);
            }
        }
        Event::KeyUp{keycode: Some(key), keymod, ..} => {
            if game.focused {
                if let Some(steven_key) = settings::Stevenkey::get_by_keycode(key, &game.vars) {
                    game.server.key_press(false, steven_key);
                }
            } else {
                let ctrl_pressed = keymod.intersects(keyboard::LCTRLMOD | keyboard::RCTRLMOD);
                ui_container.key_press(game, key, false, ctrl_pressed);
            }
        }
        Event::TextInput{text, ..} => {
            if !game.focused {
                for c in text.chars() {
                    ui_container.key_type(game, c);
                }
            }
        }
        */
        _ => (),
    }
}
