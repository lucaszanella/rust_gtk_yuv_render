extern crate rand; // 0.6.5
use gl::types::*;
use rand::Rng;
use std::cell::RefCell;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;

use super::renderer;
use super::renderer_error::RendererError;
use super::shader::Shader;
use super::vertex_array_object::VertexArrayObject;
use super::vertex_buffer_object::VertexBufferObject;

pub const VIDEO_VERTEX_SHADER: &'static str = "#version 300 es
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

void main()
{
    gl_Position = vec4(aPos, 1.0);
    TexCoord = vec2(aTexCoord.x, aTexCoord.y);
}
";

pub const PLANAR_FRAGMENT_SHADER: &'static str = "#version 300 es

#ifdef GL_ES
// Set default precision to medium
precision mediump int;
precision mediump float;
#endif

uniform sampler2D tex_y;
uniform sampler2D tex_u;
uniform sampler2D tex_v;
uniform float alpha;

in vec2 TexCoord;
out vec4 FragColor;

void main()
{
    vec3 yuv;
    vec4 rgba;
    yuv.r = texture(tex_y, TexCoord).r - 0.0625;
    yuv.g = texture(tex_u, TexCoord).r - 0.5;
    yuv.b = texture(tex_v, TexCoord).r - 0.5;

    rgba.r = yuv.r + 1.596 * yuv.b;
    rgba.g = yuv.r - 0.813 * yuv.b - 0.391 * yuv.g;
    rgba.b = yuv.r + 2.018 * yuv.g;
    
    rgba.a = alpha;
    FragColor = rgba;
    //FragColor = vec4(0.0, 0.5, 0.5, 1.0);
}";

#[warn(dead_code)]
pub const TEXTURE0: u32 = gl::TEXTURE0;
#[warn(dead_code)]
pub const TEXTURE1: u32 = gl::TEXTURE1;
#[warn(dead_code)]
pub const TEXTURE2: u32 = gl::TEXTURE2;
#[warn(dead_code)]

fn texture_number(index: u8) -> u32 {
    match index {
        0 => TEXTURE0,
        1 => TEXTURE1,
        2 => TEXTURE2,
        _ => panic!("unsupported index texture"),
    }
}

struct GLRenderer {
    program: Option<Shader>,
    texture_id: [u32; GLRenderer::TEXTURE_NUMBER as usize],
    texture_sampler: [i32; GLRenderer::TEXTURE_NUMBER as usize],
    vertex_array_object: Option<VertexArrayObject>,
    vertex_buffer_object: Option<VertexBufferObject>,
    alpha: i32,
    vertex_in_location: i32,
    texture_in_location: i32,
}

impl GLRenderer {
    //Y,U,V textures
    const TEXTURE_NUMBER: u8 = 3;
    //Square made of 2 triangles
    const VERTICES_TEXTURES: [f32; 20] = [
        -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0,
        0.0, 1.0, 0.0,
    ];
    pub fn new() -> Self {
        let r = GLRenderer {
            program: None,
            texture_id: [0; GLRenderer::TEXTURE_NUMBER as usize],
            texture_sampler: [0; GLRenderer::TEXTURE_NUMBER as usize],
            vertex_array_object: None,
            vertex_buffer_object: None,
            alpha: 0,
            vertex_in_location: 0,
            texture_in_location: 0,
        };
        r
    }

    pub fn init_vertex_stuff(&mut self) {
        self.program = Some(Shader::new());
        self.program
            .as_mut()
            .unwrap()
            .compile(VIDEO_VERTEX_SHADER, PLANAR_FRAGMENT_SHADER)
            .unwrap();
    }

    pub fn draw(&mut self, yuv: &[&[u8]; 3], widths: &[i32; 3], heights: &[i32; 3]) {
        let current_fragment_program: Option<&Shader> = self.program.as_ref();
        unsafe {
            self.vertex_in_location = gl::GetAttribLocation(
                current_fragment_program.as_ref().unwrap().program(),
                CString::new("aPos").unwrap().as_ptr(),
            );
            println!("vertex_in_location: {}", self.vertex_in_location);
            self.texture_in_location = gl::GetAttribLocation(
                current_fragment_program.as_ref().unwrap().program(),
                CString::new("aTexCoord").unwrap().as_ptr(),
            );
            println!("texture_in_location: {}", self.texture_in_location);
        }

        self.vertex_array_object = Some(VertexArrayObject::new());
        self.vertex_buffer_object = Some(VertexBufferObject::new());
        unsafe {
            self.vertex_array_object
                .as_ref()
                .unwrap()
                .activate()
                .unwrap();
            self.vertex_buffer_object
                .as_ref()
                .unwrap()
                .activate()
                .unwrap();

            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&GLRenderer::VERTICES_TEXTURES) as isize,
                GLRenderer::VERTICES_TEXTURES.as_ptr() as *const libc::c_void,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                self.vertex_in_location as u32,
                3,
                gl::FLOAT,
                gl::FALSE,
                (5 * ::std::mem::size_of::<f32>()) as i32,
                0 as *const libc::c_void,
            );
            gl::EnableVertexAttribArray(self.vertex_in_location as u32);

            gl::VertexAttribPointer(
                self.texture_in_location as u32,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * ::std::mem::size_of::<f32>()) as i32,
                3 as *const libc::c_void,
            );
            gl::EnableVertexAttribArray(self.texture_in_location as u32);

            self.texture_sampler[0] = gl::GetUniformLocation(
                current_fragment_program.as_ref().unwrap().program(),
                CString::new("tex_y").unwrap().as_ptr(),
            );
            self.texture_sampler[1] = gl::GetUniformLocation(
                current_fragment_program.as_ref().unwrap().program(),
                CString::new("tex_u").unwrap().as_ptr(),
            );
            self.texture_sampler[2] = gl::GetUniformLocation(
                current_fragment_program.as_ref().unwrap().program(),
                CString::new("tex_v").unwrap().as_ptr(),
            );

            println!("tex_y: {}", self.texture_sampler[0]);
            println!("tex_u: {}", self.texture_sampler[1]);
            println!("tex_v: {}", self.texture_sampler[2]);

            self.alpha = gl::GetUniformLocation(
                current_fragment_program.as_ref().unwrap().program(),
                CString::new("alpha").unwrap().as_ptr(),
            );
            println!("alpha: {}", self.alpha);

            gl::Uniform1f(self.alpha, 1.0);
        }

        //TODO: delete these textures
        unsafe {
            gl::GenTextures(
                GLRenderer::TEXTURE_NUMBER as i32,
                self.texture_id.as_mut_ptr(),
            );
        }
        for i in 0..GLRenderer::TEXTURE_NUMBER {
            println!("texture_id[i]= {}", self.texture_id[i as usize]);
            println!(
                "widths[i] = {}, height[i] = {}",
                widths[i as usize], heights[i as usize]
            );
            
            unsafe {
                let texture_activation_number = texture_number(i as u8);
                gl::ActiveTexture(texture_activation_number);
                gl::BindTexture(gl::TEXTURE_2D, self.texture_id[i as usize]);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    widths[i as usize],
                    heights[i as usize],
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    yuv[i as usize].as_ptr() as *const u8 as *const libc::c_void,
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_WRAP_S,
                    gl::CLAMP_TO_EDGE as GLint,
                );
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_WRAP_T,
                    gl::CLAMP_TO_EDGE as GLint,
                );
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_WRAP_R,
                    gl::CLAMP_TO_EDGE as GLint,
                );
                gl::Uniform1i(self.texture_sampler[i as usize], i as GLint);
            }
            
        }
        unsafe {
            self.vertex_array_object
                .as_ref()
                .unwrap()
                .activate()
                .unwrap();
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }
}

pub struct SmartVideoRenderer {
    scene: RefCell<GLRenderer>,
}

impl SmartVideoRenderer {
    pub fn new() -> Self {
        SmartVideoRenderer {
            scene: RefCell::new(GLRenderer::new()),
        }
    }
}

impl renderer::Renderer for SmartVideoRenderer {
    fn initialize(&self) -> Result<(), RendererError> {
        let renderer = unsafe {
            let p = gl::GetString(gl::RENDERER);
            CStr::from_ptr(p as *const i8)
        };
        println!("Renderer: {}", renderer.to_string_lossy());

        let version = unsafe {
            let p = gl::GetString(gl::VERSION);
            CStr::from_ptr(p as *const i8)
        };
        println!("OpenGL version supported: {}", version.to_string_lossy());
        self.scene.borrow_mut().init_vertex_stuff();
        Ok(())
    }

    fn finalize(&self) {}

    fn render(&self) {
        println!("render called");
        let width = 1280;
        let height = 720;
        let mut y: std::vec::Vec<u8> = std::vec::Vec::new();
        let mut u: std::vec::Vec<u8> = std::vec::Vec::new();
        let mut v: std::vec::Vec<u8> = std::vec::Vec::new();
        y.resize((width * height) as usize, 0);
        u.resize((width * height / 4) as usize, 0);
        v.resize((width * height / 4) as usize, 0);

        //Either reads from an image or from random data
        let from_image = false;
        
        if from_image {
            let mut f = File::open("/home/dev/orwell/lab/orwell_gtk/assets/vaporwave.yuv")
                .expect("Unable to open file");

            f.read_exact(y.as_mut_slice()).unwrap();
            f.read_exact(u.as_mut_slice()).unwrap();
            f.read_exact(v.as_mut_slice()).unwrap();
        } else {
            for i in 0..y.len() {
                let num = rand::thread_rng().gen_range(0, 255);
                y[i] = num as u8;
            }
            for i in 0..u.len() {
                let num = rand::thread_rng().gen_range(0, 255);
                u[i] = num as u8;
            }
            for i in 0..v.len() {
                let num = rand::thread_rng().gen_range(0, 255);
                v[i] = num as u8;
            }
        }
        
        self.scene.borrow_mut().draw(
            &[y.as_slice(), u.as_slice(), v.as_slice()],
            &[width, width / 2, width / 2],
            &[height, height / 2, height / 2],
        );
    }
}
