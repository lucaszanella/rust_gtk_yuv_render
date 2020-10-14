//use super::vertex_buffer_object::VertexBufferObject;

pub struct VertexArrayObject {
    vao: Option<u32>,
    //buffers: Vec<VertexBufferObject>,
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        match self.vao {
            Some(vao) => {
                let mut x: u32 = vao;
                unsafe {
                    gl::DeleteVertexArrays(1, &x);
                }
            }
            None => {}
        }
    }
}

impl VertexArrayObject {
    pub fn empty() -> Self {
        VertexArrayObject {
            vao: None,
            //buffers: Vec::new(),
        }
    }

    pub fn new() -> Self {
        let mut v: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut v);
        }
        //println!("vao: {}", v);
        VertexArrayObject {
            vao: Some(v),
            //buffers: Vec::new(),
        }
        
    }

    pub fn inner_value(&self) -> Option<u32> {
        self.vao
    }

    pub fn activate(&self) -> Result<(), ()> {
        match self.vao {
            Some(vao) => unsafe {
                gl::BindVertexArray(vao);
                Ok(())
            },
            None => Err(()),
        }
    }

    /*
    pub unsafe fn VertexAttribPointer(
        index: types::GLuint,
        size: types::GLint,
        type_: types::GLenum,
        normalized: types::GLboolean,
        stride: types::GLsizei,
        pointer: *const __gl_imports::raw::c_void,
    */

    pub fn append(
        index: u32,
        size: i32,
        normalized: bool,
        stride: i32,
        offset: &u32,
    ) -> Result<(), ()>{
        unsafe{
            gl::VertexAttribPointer(index, size, gl::FLOAT, gl::FALSE, (5 * ::std::mem::size_of::<f32>()) as i32, offset as *const _ as *const libc::c_void);
            gl::EnableVertexAttribArray(index);
        }
        Ok(())
    }


    /*
    pub fn append(&mut self, vbo: VertexBufferObject) -> Result<(), ()> {
        
        vbo.activate();
        let attribute = self.buffers.len() as u32;
        self.buffers.push(vbo);

        match self.activate() {
            Ok(()) => unsafe {
                gl::EnableVertexAttribArray(attribute);
                gl::VertexAttribPointer(attribute, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
                Ok(())
            },
            Err(()) => Err(()),
        }
        
        Ok(())
    }
    */
    /*
    pub fn draw(&self) {
        self.activate();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
    */
}
