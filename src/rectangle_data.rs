use once_cell::sync::{Lazy};
use wgpu::vertex_attr_array;


// FIXME: figure out why this crashes the analyzer

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectangleVertex {
    pub position: [f32; 2],
}

impl RectangleVertex {
    pub fn description<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectangleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectangleInstance {
    pub color: [f32; 3],
    pub transform: [[f32; 3]; 3],
    pub screen_size: [f32; 2],
}

// TODO: do something to keep this, RectangleInstance, and the shader in sync
static RECTANGLE_INSTANCE_ATTRIBUTES: Lazy<[wgpu::VertexAttribute; 5]> = Lazy::new(|| {
    vertex_attr_array![
        3 => Float32x3, // color
        4 => Float32x3, // transform[0]
        5 => Float32x3, // transform[1]
        6 => Float32x3, // transform[2]
        7 => Float32x2, // screen_size
    ]
});

impl RectangleInstance {
    pub fn description() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectangleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: RECTANGLE_INSTANCE_ATTRIBUTES.as_ref(),
        }
    }
}
