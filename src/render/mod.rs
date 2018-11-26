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

mod atlas;
pub mod glsl;
#[macro_use]
pub mod shaders;
pub mod ui;
pub mod model;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::io::Write;
use crate::resources;
use crate::gl;
use image;
use image::{GenericImage, GenericImageView};
use byteorder::{WriteBytesExt, NativeEndian};
use cgmath::prelude::*;
use collision;
use log::{error};

use std::hash::BuildHasherDefault;
use crate::types::hash::FNVHash;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::thread;
use std::sync::mpsc;

const ATLAS_SIZE: usize = 1024;

// TEMP
const NUM_SAMPLES: i32 = 2;

pub struct Camera {
    pub pos: cgmath::Point3<f64>,
    pub yaw: f64,
    pub pitch: f64,
}

pub struct Renderer {
    resource_version: usize,
    pub resources: Arc<RwLock<resources::Manager>>,
    textures: Arc<RwLock<TextureManager>>,
    pub ui: ui::UIState,
    pub model: model::Manager,

    gl_texture: gl::Texture,

    trans_shader: TransShader,

    element_buffer: gl::Buffer,
    element_buffer_size: usize,
    element_buffer_type: gl::Type,

    pub camera: Camera,
    perspective_matrix: cgmath::Matrix4<f32>,
    camera_matrix: cgmath::Matrix4<f32>,
    pub frustum: collision::Frustum<f32>,
    pub view_vector: cgmath::Vector3<f32>,

    pub frame_id: u32,

    trans: Option<TransInfo>,

    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct ChunkBuffer {
    solid: Option<ChunkRenderInfo>,
    trans: Option<ChunkRenderInfo>,
}

impl ChunkBuffer {
    pub fn new() -> ChunkBuffer { Default::default() }
}

struct ChunkRenderInfo {
    array: gl::VertexArray,
    buffer: gl::Buffer,
    buffer_size: usize,
    count: usize,
}

impl Renderer {
    pub fn new(res: Arc<RwLock<resources::Manager>>) -> Renderer {
        let version = {
            res.read().unwrap().version()
        };
        let tex = gl::Texture::new();
        tex.bind(gl::TEXTURE_2D_ARRAY);
        tex.image_3d(gl::TEXTURE_2D_ARRAY,
                     0,
                     ATLAS_SIZE as u32,
                     ATLAS_SIZE as u32,
                     1,
                     gl::RGBA,
                     gl::UNSIGNED_BYTE,
                     &[0; ATLAS_SIZE * ATLAS_SIZE * 4]);
        tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::NEAREST);
        tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::NEAREST);
        tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
        tex.set_parameter(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);

        let (textures, _skin_req, _skin_reply) = TextureManager::new(res.clone());
        let textures = Arc::new(RwLock::new(textures));

        let mut greg = glsl::Registry::new();
        shaders::add_shaders(&mut greg);
        let ui = ui::UIState::new(&greg, textures.clone(), res.clone());

        // Shaders
        let trans_shader = TransShader::new(&greg);

        Renderer {
            resource_version: version,
            model: model::Manager::new(&greg),
            textures,
            ui,
            resources: res,
            gl_texture: tex,

            trans_shader,

            element_buffer: gl::Buffer::new(),
            element_buffer_size: 0,
            element_buffer_type: gl::UNSIGNED_BYTE,

            width: 0,
            height: 0,

            camera: Camera {
                pos: cgmath::Point3::new(0.0, 0.0, 0.0),
                yaw: 0.0,
                pitch: ::std::f64::consts::PI,
            },
            perspective_matrix: cgmath::Matrix4::identity(),
            camera_matrix: cgmath::Matrix4::identity(),
            frustum: collision::Frustum::from_matrix4(cgmath::Matrix4::identity()).unwrap(),
            view_vector: cgmath::Vector3::zero(),

            frame_id: 1,

            trans: None,
        }
    }

    pub fn update_camera(&mut self) {
        use std::f64::consts::PI as PI64;

        let width = 854;
        let height = 480;
        self.width = width;
        self.height = height;
        gl::viewport(0, 0, width as i32, height as i32);

        self.perspective_matrix = cgmath::Matrix4::from(
            cgmath::PerspectiveFov {
                fovy: cgmath::Rad::from(cgmath::Deg(90f32)),
                aspect: (width as f32 / height as f32),
                near: 0.1f32,
                far: 500.0f32,
            }
        );

        self.init_trans(width, height);

        self.camera.yaw = -7.2697720829739465;
        self.camera.pitch = 2.9733976253414633;
        self.camera.pos.x = -208.76533603647485;
        self.camera.pos.y = 65.62010000000001;
        self.camera.pos.z = 90.9279311085242;

        self.view_vector = cgmath::Vector3::new(
            ((self.camera.yaw - PI64/2.0).cos() * -self.camera.pitch.cos()) as f32,
            (-self.camera.pitch.sin()) as f32,
            (-(self.camera.yaw - PI64/2.0).sin() * -self.camera.pitch.cos()) as f32
        );
        let camera = cgmath::Point3::new(-self.camera.pos.x as f32, -self.camera.pos.y as f32, self.camera.pos.z as f32);
        let camera_matrix = cgmath::Matrix4::look_at(
            camera,
            camera + cgmath::Point3::new(-self.view_vector.x, -self.view_vector.y, self.view_vector.z).to_vec(),
            cgmath::Vector3::new(0.0, -1.0, 0.0)
        );
        self.camera_matrix = camera_matrix * cgmath::Matrix4::from_nonuniform_scale(-1.0, 1.0, 1.0);
        self.frustum = collision::Frustum::from_matrix4(self.perspective_matrix * self.camera_matrix).unwrap();
    }

    pub fn tick(&mut self) {
        self.update_textures();

        let trans = self.trans.as_mut().unwrap();
        trans.main.bind();

        gl::clear_color(
             122.0 / 255.0,
             165.0 / 255.0,
             247.0 / 255.0,
             1.0
        );
        gl::clear(gl::ClearFlags::Color | gl::ClearFlags::Depth);

        // Model rendering
        self.model.draw(&self.frustum, &self.perspective_matrix, &self.camera_matrix);

        trans.trans.bind();
        gl::clear_buffer(gl::COLOR, 0, &[0.0, 0.0, 0.0, 1.0]);

        gl::check_framebuffer_status();
        gl::unbind_framebuffer();
        trans.draw(&self.trans_shader);

        gl::check_gl_error();

        self.frame_id = self.frame_id.wrapping_add(1);
    }

    fn ensure_element_buffer(&mut self, size: usize) {
        if self.element_buffer_size < size {
            let (data, ty) = self::generate_element_buffer(size);
            self.element_buffer_type = ty;
            self.element_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
            self.element_buffer.set_data(gl::ELEMENT_ARRAY_BUFFER, &data, gl::DYNAMIC_DRAW);
            self.element_buffer_size = size;
        }
    }

    pub fn update_chunk_solid(&mut self, buffer: &mut ChunkBuffer, data: &[u8], count: usize) {
        self.ensure_element_buffer(count);
        if count == 0 {
            if buffer.solid.is_some() {
                buffer.solid = None;
            }
            return;
        }
        let new = buffer.solid.is_none();
        if buffer.solid.is_none() {
            buffer.solid = Some(ChunkRenderInfo {
                array: gl::VertexArray::new(),
                buffer: gl::Buffer::new(),
                buffer_size: 0,
                count: 0,
            });
        }
        let info = buffer.solid.as_mut().unwrap();

        info.array.bind();

        self.element_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);

        info.buffer.bind(gl::ARRAY_BUFFER);
        if new || info.buffer_size < data.len() {
            info.buffer_size = data.len();
            info.buffer.set_data(gl::ARRAY_BUFFER, data, gl::DYNAMIC_DRAW);
        } else {
            info.buffer.re_set_data(gl::ARRAY_BUFFER, data);
        }

        info.count = count;
    }

    pub fn update_chunk_trans(&mut self, buffer: &mut ChunkBuffer, _data: &[u8], count: usize) {
        self.ensure_element_buffer(count);
        if count == 0 {
            if buffer.trans.is_some() {
                buffer.trans = None;
            }
            return;
        }
        if buffer.trans.is_none() {
            buffer.trans = Some(ChunkRenderInfo {
                array: gl::VertexArray::new(),
                buffer: gl::Buffer::new(),
                buffer_size: 0,
                count: 0,
            });
        }
        let info = buffer.trans.as_mut().unwrap();

        info.array.bind();

        info.count = count;
    }

    fn do_pending_textures(&mut self) {
        let len = {
            let tex = self.textures.read().unwrap();
            tex.pending_uploads.len()
        };
        if len > 0 {
            // Upload pending changes
            let mut tex = self.textures.write().unwrap();
            for upload in &tex.pending_uploads {
                let atlas = upload.0;
                let rect = upload.1;
                let img = &upload.2;
                self.gl_texture.sub_image_3d(gl::TEXTURE_2D_ARRAY,
                                             0,
                                             rect.x as u32,
                                             rect.y as u32,
                                             atlas as u32,
                                             rect.width as u32,
                                             rect.height as u32,
                                             1,
                                             gl::RGBA,
                                             gl::UNSIGNED_BYTE,
                                             &img[..]);
            }
            tex.pending_uploads.clear();
        }
    }

    fn update_textures(&mut self) {
        self.do_pending_textures();
    }

    fn init_trans(&mut self, width: u32, height: u32) {
        self.trans = None;
        self.trans = Some(TransInfo::new(width, height, &self.trans_shader));
    }

    pub fn get_textures(&self) -> Arc<RwLock<TextureManager>> {
        self.textures.clone()
    }

    pub fn get_textures_ref(&self) -> &RwLock<TextureManager> {
        &self.textures
    }

    pub fn check_texture(&self, tex: Texture) -> Texture {
        if tex.version == self.resource_version {
            tex
        } else {
            let mut new = Renderer::get_texture(&self.textures, &tex.name);
            new.rel_x = tex.rel_x;
            new.rel_y = tex.rel_y;
            new.rel_width = tex.rel_width;
            new.rel_height = tex.rel_height;
            new.is_rel = tex.is_rel;
            new
        }
    }

    pub fn get_texture(textures: &RwLock<TextureManager>, name: &str) -> Texture {
        let tex = {
            textures.read().unwrap().get_texture(name)
        };
        match tex {
            Some(val) => val,
            None => {
                let mut t = textures.write().unwrap();
                // Make sure it hasn't already been loaded since we switched
                // locks.
                if let Some(val) = t.get_texture(name) {
                    val
                } else {
                    t.load_texture(name);
                    t.get_texture(name).unwrap()
                }
            }
        }
    }

    pub fn get_skin(&self, textures: &RwLock<TextureManager>, url: &str) -> Texture {
        let tex = {
            textures.read().unwrap().get_skin(url)
        };
        match tex {
            Some(val) => val,
            None => {
                let t = textures.write().unwrap();
                // Make sure it hasn't already been loaded since we switched
                // locks.
                if let Some(val) = t.get_skin(url) {
                    val
                } else {
                    t.get_skin(url).unwrap()
                }
            }
        }
    }
}

struct TransInfo {
    main: gl::Framebuffer,
    fb_color: gl::Texture,
    _fb_depth: gl::Texture,
    trans: gl::Framebuffer,
    accum: gl::Texture,
    revealage: gl::Texture,
    _depth: gl::Texture,

    array: gl::VertexArray,
    _buffer: gl::Buffer,
}

init_shader! {
    Program TransShader {
        vert = "trans_vertex",
        frag = "trans_frag",
        attribute = {
            required position => "aPosition",
        },
        uniform = {
            required accum => "taccum",
            required revealage => "trevealage",
            required color => "tcolor",
            required samples => "samples",
        },
    }
}

impl TransInfo {
    pub fn new(width: u32, height: u32, shader: &TransShader) -> TransInfo {
        let trans = gl::Framebuffer::new();
        trans.bind();

        let accum = gl::Texture::new();
        accum.bind(gl::TEXTURE_2D);
        accum.image_2d_ex(gl::TEXTURE_2D, 0, width, height, gl::RGBA16F, gl::RGBA, gl::FLOAT, None);
        accum.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        accum.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::COLOR_ATTACHMENT_0, gl::TEXTURE_2D, &accum, 0);

        let revealage = gl::Texture::new();
        revealage.bind(gl::TEXTURE_2D);
        revealage.image_2d_ex(gl::TEXTURE_2D, 0, width, height, gl::R16F, gl::RED, gl::FLOAT, None);
        revealage.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        revealage.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::COLOR_ATTACHMENT_1, gl::TEXTURE_2D, &revealage, 0);

        let trans_depth = gl::Texture::new();
        trans_depth.bind(gl::TEXTURE_2D);
        trans_depth.image_2d_ex(gl::TEXTURE_2D, 0, width, height, gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE, None);
        trans_depth.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        trans_depth.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, gl::LINEAR);
        trans.texture_2d(gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, &trans_depth, 0);

        gl::check_framebuffer_status();

        let main = gl::Framebuffer::new();
        main.bind();

        let fb_color = gl::Texture::new();
        fb_color.bind(gl::TEXTURE_2D_MULTISAMPLE);
        fb_color.image_2d_sample(gl::TEXTURE_2D_MULTISAMPLE, NUM_SAMPLES, width, height, gl::RGBA8, false);
        main.texture_2d(gl::COLOR_ATTACHMENT_0, gl::TEXTURE_2D_MULTISAMPLE, &fb_color, 0);

        let fb_depth = gl::Texture::new();
        fb_depth.bind(gl::TEXTURE_2D_MULTISAMPLE);
        fb_depth.image_2d_sample(gl::TEXTURE_2D_MULTISAMPLE, NUM_SAMPLES, width, height, gl::DEPTH_COMPONENT24, false);
        main.texture_2d(gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D_MULTISAMPLE, &fb_depth, 0);
        gl::check_framebuffer_status();

        gl::unbind_framebuffer();

        shader.program.use_program();
        let array = gl::VertexArray::new();
        array.bind();
        let buffer = gl::Buffer::new();
        buffer.bind(gl::ARRAY_BUFFER);

        let mut data = vec![];
        for f in [-1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0].into_iter() {
            data.write_f32::<NativeEndian>(*f).unwrap();
        }
        buffer.set_data(gl::ARRAY_BUFFER, &data, gl::STATIC_DRAW);

        shader.position.enable();
        shader.position.vertex_pointer(2, gl::FLOAT, false, 8, 0);

        TransInfo {
            main,
            fb_color,
            _fb_depth: fb_depth,
            trans,
            accum,
            revealage,
            _depth: trans_depth,

            array,
            _buffer: buffer,
        }
    }

    fn draw(&mut self, shader: &TransShader) {
        gl::active_texture(0);
        self.accum.bind(gl::TEXTURE_2D);
        gl::active_texture(1);
        self.revealage.bind(gl::TEXTURE_2D);
        gl::active_texture(2);
        self.fb_color.bind(gl::TEXTURE_2D_MULTISAMPLE);

        shader.program.use_program();
        shader.accum.set_int(0);
        shader.revealage.set_int(1);
        shader.color.set_int(2);
        shader.samples.set_int(NUM_SAMPLES);
        self.array.bind();
        gl::draw_arrays(gl::TRIANGLES, 0, 6);
    }
}

pub struct TextureManager {
    textures: HashMap<String, Texture, BuildHasherDefault<FNVHash>>,
    version: usize,
    resources: Arc<RwLock<resources::Manager>>,
    atlases: Vec<atlas::Atlas>,

    pending_uploads: Vec<(i32, atlas::Rect, Vec<u8>)>,

    dynamic_textures: HashMap<String, (Texture, image::DynamicImage), BuildHasherDefault<FNVHash>>,
    free_dynamics: Vec<Texture>,

    skins: HashMap<String, AtomicIsize, BuildHasherDefault<FNVHash>>,

    _skin_thread: thread::JoinHandle<()>,
}

impl TextureManager {
    fn new(res: Arc<RwLock<resources::Manager>>) -> (TextureManager, mpsc::Sender<String>, mpsc::Receiver<(String, Option<image::DynamicImage>)>) {
        let (tx, rx) = mpsc::channel();
        let (stx, srx) = mpsc::channel();
        let skin_thread = thread::spawn(|| Self::process_skins(srx, tx));
        let mut tm = TextureManager {
            textures: HashMap::with_hasher(BuildHasherDefault::default()),
            version: {
                let ver = res.read().unwrap().version();
                ver
            },
            resources: res,
            atlases: Vec::new(),
            pending_uploads: Vec::new(),

            dynamic_textures: HashMap::with_hasher(BuildHasherDefault::default()),
            free_dynamics: Vec::new(),
            skins: HashMap::with_hasher(BuildHasherDefault::default()),

            _skin_thread: skin_thread,
        };
        tm.add_defaults();
        (tm, stx, rx)
    }

    fn add_defaults(&mut self) {
        self.put_texture("steven",
                         "missing_texture",
                         2,
                         2,
                         vec![
            0, 0, 0, 255,
            255, 0, 255, 255,
            255, 0, 255, 255,
            0, 0, 0, 255,
        ]);
        self.put_texture("steven",
                         "solid",
                         1,
                         1,
                         vec![
            255, 255, 255, 255,
        ]);
    }

    fn process_skins(recv: mpsc::Receiver<String>, reply: mpsc::Sender<(String, Option<image::DynamicImage>)>) {
        use reqwest;
        let client = reqwest::Client::new();
        loop {
            let hash = match recv.recv() {
                Ok(val) => val,
                Err(_) => return, // Most likely shutting down
            };
            match Self::obtain_skin(&client, &hash) {
                Ok(img) => {
                    let _ = reply.send((hash, Some(img)));
                },
                Err(err) => {
                    error!("Failed to get skin {:?}: {}", hash, err);
                    let _ = reply.send((hash, None));
                },
            }
        }
    }

    fn obtain_skin(client: &::reqwest::Client, hash: &str) -> Result<image::DynamicImage, ::std::io::Error> {
        use std::io::Read;
        use std::fs;
        use std::path::Path;
        use std::io::{Error, ErrorKind};
        let path = format!("skin-cache/{}/{}.png", &hash[..2], hash);
        let cache_path = Path::new(&path);
        fs::create_dir_all(cache_path.parent().unwrap())?;
        let mut buf = vec![];
        if fs::metadata(cache_path).is_ok() {
            // We have a cached image
            let mut file = fs::File::open(cache_path)?;
            file.read_to_end(&mut buf)?;
        } else {
            // Need to download it
            let url = &format!("http://textures.minecraft.net/texture/{}", hash);
            let mut res = match client.get(url).send() {
                Ok(val) => val,
                Err(err) => {
                    return Err(Error::new(ErrorKind::ConnectionAborted, err));
                }
            };
            let mut buf = vec![];
            match res.read_to_end(&mut buf) {
                Ok(_) => {},
                Err(err) => {
                    // TODO: different error for failure to read?
                    return Err(Error::new(ErrorKind::InvalidData, err));
                }
            }

            // Save to cache
            let mut file = fs::File::create(cache_path)?;
            file.write_all(&buf)?;
        }
        let mut img = match image::load_from_memory(&buf) {
            Ok(val) => val,
            Err(err) => {
                return Err(Error::new(ErrorKind::InvalidData, err));
            }
        };
        let (_, height) = img.dimensions();
        if height == 32 {
            // Needs changing to the new format
            let mut new = image::DynamicImage::new_rgba8(64, 64);
            new.copy_from(&img, 0, 0);
            for xx in 0 .. 4 {
                for yy in 0 .. 16 {
                    for section in 0 .. 4 {
                        let os = match section {
                            0 => 2,
                            1 => 1,
                            2 => 0,
                            3 => 3,
                            _ => unreachable!(),
                        };
                        new.put_pixel(16 + (3 - xx) + section * 4, 48 + yy, img.get_pixel(xx + os * 4, 16 + yy));
                        new.put_pixel(32 + (3 - xx) + section * 4, 48 + yy, img.get_pixel(xx + 40 + os * 4, 16 + yy));
                    }
                }
            }
            img = new;
        }
        // Block transparent pixels in blacklisted areas
        let blacklist = [
            // X, Y, W, H
            (0, 0, 32, 16),
            (16, 16, 24, 16),
            (0, 16, 16, 16),
            (16, 48, 16, 16),
            (32, 48, 16, 16),
            (40, 16, 16, 16),
        ];
        for bl in blacklist.into_iter() {
            for x in bl.0 .. (bl.0 + bl.2) {
                for y in bl.1 .. (bl.1 + bl.3) {
                    let mut col = img.get_pixel(x, y);
                    col.data[3] = 255;
                    img.put_pixel(x, y, col);
                }
            }
        }
        Ok(img)
    }

    fn get_skin(&self, url: &str) -> Option<Texture> {
        let hash = &url["http://textures.minecraft.net/texture/".len()..];
        if let Some(skin) = self.skins.get(hash) {
            skin.fetch_add(1, Ordering::Relaxed);
        }
        self.get_texture(&format!("steven-dynamic:skin-{}", hash))
    }

    pub fn release_skin(&self, url: &str) {
        let hash = &url["http://textures.minecraft.net/texture/".len()..];
        if let Some(skin) = self.skins.get(hash) {
            skin.fetch_sub(1, Ordering::Relaxed);
        }
    }

    fn get_texture(&self, name: &str) -> Option<Texture> {
        if let Some(_) = name.find(':') {
            self.textures.get(name).cloned()
        } else {
            self.textures.get(&format!("minecraft:{}", name)).cloned()
        }
    }

    fn load_texture(&mut self, name: &str) {
        let (plugin, name) = if let Some(pos) = name.find(':') {
            (&name[..pos], &name[pos + 1..])
        } else {
            ("minecraft", name)
        };
        let path = format!("textures/{}.png", name);
        let res = self.resources.clone();
        if let Some(mut val) = res.read().unwrap().open(plugin, &path) {
            let mut data = Vec::new();
            val.read_to_end(&mut data).unwrap();
            if let Ok(img) = image::load_from_memory(&data) {
                let (width, height) = img.dimensions();
                self.put_texture(plugin, name, width, height, img.to_rgba().into_vec());
                return;
            }
        }
        self.insert_texture_dummy(plugin, name);
    }

    fn put_texture(&mut self,
                   plugin: &str,
                   name: &str,
                   width: u32,
                   height: u32,
                   data: Vec<u8>)
                   -> Texture {
        let (atlas, rect) = self.find_free(width as usize, height as usize);
        self.pending_uploads.push((atlas, rect, data));

        let mut full_name = String::new();
        full_name.push_str(plugin);
        full_name.push_str(":");
        full_name.push_str(name);

        let tex = Texture {
            name: full_name.clone(),
            version: self.version,
            atlas,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            rel_x: 0.0,
            rel_y: 0.0,
            rel_width: 1.0,
            rel_height: 1.0,
            is_rel: false,
        };
        self.textures.insert(full_name, tex.clone());
        tex
    }

    fn find_free(&mut self, width: usize, height: usize) -> (i32, atlas::Rect) {
        let mut index = 0;
        for atlas in &mut self.atlases {
            if let Some(rect) = atlas.add(width, height) {
                return (index, rect);
            }
            index += 1;
        }
        let mut atlas = atlas::Atlas::new(ATLAS_SIZE, ATLAS_SIZE);
        let rect = atlas.add(width, height);
        self.atlases.push(atlas);
        (index, rect.unwrap())
    }

    fn insert_texture_dummy(&mut self, plugin: &str, name: &str) -> Texture {
        let missing = self.get_texture("steven:missing_texture").unwrap();

        let mut full_name = String::new();
        full_name.push_str(plugin);
        full_name.push_str(":");
        full_name.push_str(name);

        let t = Texture {
            name: full_name.to_owned(),
            version: self.version,
            atlas: missing.atlas,
            x: missing.x,
            y: missing.y,
            width: missing.width,
            height: missing.height,
            rel_x: 0.0,
            rel_y: 0.0,
            rel_width: 1.0,
            rel_height: 1.0,
            is_rel: false,
        };
        self.textures.insert(full_name.to_owned(), t.clone());
        t
    }

    pub fn put_dynamic(&mut self, name: &str, img: image::DynamicImage) -> Texture {
        use std::mem;
        let (width, height) = img.dimensions();
        let (width, height) = (width as usize, height as usize);
        let mut rect_pos = None;
        for (i, r) in self.free_dynamics.iter().enumerate() {
            if r.width == width && r.height == height {
                rect_pos = Some(i);
                break;
            } else if r.width >= width && r.height >= height {
                rect_pos = Some(i);
            }
        }
        let data = img.to_rgba().into_vec();

        if let Some(rect_pos) = rect_pos {
            let mut tex = self.free_dynamics.remove(rect_pos);
            let rect = atlas::Rect {
                x: tex.x,
                y: tex.y,
                width,
                height,
            };
            self.pending_uploads.push((tex.atlas, rect, data));
            let mut t = tex.relative(0.0, 0.0, (width as f32) / (tex.width as f32), (height as f32) / (tex.height as f32));
            let old_name = mem::replace(&mut tex.name, format!("steven-dynamic:{}", name));
            self.dynamic_textures.insert(name.to_owned(), (tex.clone(), img));
            // We need to rename the texture itself so that get_texture calls
            // work with the new name
            let mut old = self.textures.remove(&old_name).unwrap();
            old.name = format!("steven-dynamic:{}", name);
            t.name = old.name.clone();
            self.textures.insert(format!("steven-dynamic:{}", name), old);
            t
        } else {
            let tex = self.put_texture("steven-dynamic", name, width as u32, height as u32, data);
            self.dynamic_textures.insert(name.to_owned(), (tex.clone(), img));
            tex
        }
    }

    pub fn remove_dynamic(&mut self, name: &str) {
        let desc = self.dynamic_textures.remove(name).unwrap();
        self.free_dynamics.push(desc.0);
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub name: String,
    version: usize,
    pub atlas: i32,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    is_rel: bool, // Save some cycles for non-relative textures
    rel_x: f32,
    rel_y: f32,
    rel_width: f32,
    rel_height: f32,
}

impl Texture {
    pub fn get_x(&self) -> usize {
        if self.is_rel {
            self.x + ((self.width as f32) * self.rel_x) as usize
        } else {
            self.x
        }
    }

    pub fn get_y(&self) -> usize {
        if self.is_rel {
            self.y + ((self.height as f32) * self.rel_y) as usize
        } else {
            self.y
        }
    }

    pub fn get_width(&self) -> usize {
        if self.is_rel {
            ((self.width as f32) * self.rel_width) as usize
        } else {
            self.width
        }
    }

    pub fn get_height(&self) -> usize {
        if self.is_rel {
            ((self.height as f32) * self.rel_height) as usize
        } else {
            self.height
        }
    }

    pub fn relative(&self, x: f32, y: f32, width: f32, height: f32) -> Texture {
        Texture {
            name: self.name.clone(),
            version: self.version,
            x: self.x,
            y: self.y,
            atlas: self.atlas,
            width: self.width,
            height: self.height,
            is_rel: true,
            rel_x: self.rel_x + x * self.rel_width,
            rel_y: self.rel_y + y * self.rel_height,
            rel_width: width * self.rel_width,
            rel_height: height * self.rel_height,
        }
    }
}

#[allow(unused_must_use)]
pub fn generate_element_buffer(size: usize) -> (Vec<u8>, gl::Type) {
    let mut ty = gl::UNSIGNED_SHORT;
    let mut data = if (size / 6) * 4 * 3 >= u16::max_value() as usize {
        ty = gl::UNSIGNED_INT;
        Vec::with_capacity(size * 4)
    } else {
        Vec::with_capacity(size * 2)
    };
    for i in 0..size / 6 {
        for val in &[0, 1, 2, 2, 1, 3] {
            if ty == gl::UNSIGNED_INT {
                data.write_u32::<NativeEndian>((i as u32) * 4 + val);
            } else {
                data.write_u16::<NativeEndian>((i as u16) * 4 + (*val as u16));
            }
        }
    }

    (data, ty)
}
