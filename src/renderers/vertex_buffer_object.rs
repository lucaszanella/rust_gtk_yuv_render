pub struct VertexBufferObject {
    vbo: Option<u32>,
}

impl Drop for VertexBufferObject {
    fn drop(&mut self) {
        match self.vbo {
            Some(vbo) => {
                let mut x = vbo;
                unsafe {
                    gl::DeleteBuffers(1, &x);
                }
            }
            None => {}
        }
    }
}

impl VertexBufferObject {
    pub fn new() -> Self {
        let mut vbo: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }
        VertexBufferObject { vbo: Some(vbo) }
    }

    pub fn inner_value(&self) -> Option<u32> {
        self.vbo
    }

    pub fn activate(&self) -> Result<(), ()> {
        match self.vbo {
            Some(vbo) => unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                Ok(())
            },
            None => Err(()),
        }
    }

    pub fn assign(&self, values: &[f32]) -> Result<(), ()> {
        match self.activate() {
            Ok(()) => {
                unsafe {
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        std::mem::size_of_val(values) as isize,
                        values.as_ptr() as *const std::ffi::c_void,
                        gl::STATIC_DRAW,
                    );
                }
                Ok(())
            }
            Err(()) => Err(()),
        }
    }
}
