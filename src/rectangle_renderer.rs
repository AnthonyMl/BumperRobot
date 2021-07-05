use cgmath::{Matrix3, Vector2, Vector3};
use wgpu::{Device, RenderPass, TextureFormat, util::DeviceExt};

use crate::{img::Rectangle, rectangle_data::{RectangleInstance, RectangleVertex}};


pub struct RectangleRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,

    // The rectangles are stored with the permanent ones in the front followed by the transient.
    // Assumes sequential execution
    rectangles: Vec<RectangleInstance>,
    transient_count: usize,
    permanent_count: usize,
    max_instance_count: usize,
    pub instance_buffer: wgpu::Buffer,
}

const VERTICES: &[RectangleVertex] = {
    const L: f32 = 1.0;
    &[
        RectangleVertex { position: [ 0.0, 0.0 ] },
        RectangleVertex { position: [   L, 0.0 ] },
        RectangleVertex { position: [ 0.0,   L ] },
        RectangleVertex { position: [   L,   L ] },
    ]
};

impl RectangleRenderer {
    pub fn new(device: &Device, color_format: TextureFormat) -> RectangleRenderer {
        const MAX_INSTANCE_COUNT: usize = 30;

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Rectangle Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("rectangle_shader.wgsl").into())
        });

        let instance_buffer = {
            let data = [RectangleInstance::default(); MAX_INSTANCE_COUNT];
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Rectangle Instance Buffer"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST
            })
        };

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rectangle Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rectangle Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: "main", buffers: &[RectangleVertex::description(), RectangleInstance::description()]},
            fragment: Some(wgpu::FragmentState {
                module: &shader, entry_point: "main", targets: &[wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrite::all()
                }]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1, mask: !0, alpha_to_coverage_enabled: false
            }
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rectangle Vertex Buffer"),
            usage: wgpu::BufferUsage::VERTEX,
            contents: bytemuck::cast_slice(VERTICES)
        });

        RectangleRenderer {
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            max_instance_count: MAX_INSTANCE_COUNT,
            permanent_count: 0,
            transient_count: 0,
            rectangles: vec![],
        }
    }

    fn add_rectangle(&mut self, permanent: bool, rectangle: &Rectangle<f32>, color: &[f32; 3]) {
        if self.permanent_count + self.transient_count >= self.max_instance_count {
            eprintln!("Attempting to add more rectangles than supported");
            return
        }
        let screen_size = [rectangle.width, rectangle.height];

        // top left origin to middle, flip y axis
        // 0..1 -> -1..1
        let window_norm_to_device = Matrix3::new(
             2.0,  0.0, 0.0,
             0.0, -2.0, 0.0,
            -1.0,  1.0, 0.0);

        let origin = window_norm_to_device * Vector3::new(rectangle.left, rectangle.top, 1.0);

        let translation = Matrix3::from_translation(Vector2::new(origin.x, origin.y));

        let scale = Matrix3::from_nonuniform_scale(2.0 * rectangle.width, -2.0 * rectangle.height);

        let transform = translation * scale;

        if permanent {
            self.rectangles.insert(0, RectangleInstance {
                transform: *transform.as_ref(), color: *color, screen_size
            });
            self.permanent_count += 1;
        } else {
            self.rectangles.push(RectangleInstance {
                transform: *transform.as_ref(), color: *color, screen_size
            });
            self.transient_count += 1;
        }
    }

    pub fn add(&mut self, rectangle: &Rectangle<f32>, color: &[f32; 3]) {
        self.add_rectangle(false, rectangle, color);
    }

    pub fn add_permanent(&mut self, rectangle: &Rectangle<f32>, color: &[f32; 3]) {
        self.add_rectangle(true, rectangle, color);
    }

    pub fn clear(&mut self) {
        self.transient_count = 0;

        self.rectangles.truncate(self.permanent_count);
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        render_pass.draw(0..4,
            0..(self.transient_count + self.permanent_count) as u32);
    }

    pub fn data(&mut self) -> Vec<RectangleInstance> {
        self.rectangles.clone()
    }
}


