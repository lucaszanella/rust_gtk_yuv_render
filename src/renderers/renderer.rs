use super::renderer_error::RendererError;

pub trait Renderer {

    fn initialize(&self) -> Result<(), RendererError>
    {
        Ok(())
    }
    fn finalize(&self)
    {

    }
    fn resize(&self, w: u32, h: u32) {
        println!("resize: {}x{}", w, h);
    }
    fn render(&self);
}
