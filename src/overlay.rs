use winit::{dpi::{PhysicalPosition, PhysicalSize, Position, Size}, event_loop::EventLoop, window::{Window, WindowBuilder}};

use crate::{capture::ACTIVE_RECTANGLE, img::Rectangle, rectangle_renderer::RectangleRenderer};


pub struct Overlay {
    // need to be kept alive
    _surface: wgpu::Surface,
    _swap_chain_descriptor: wgpu::SwapChainDescriptor,

    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,

    rr: RectangleRenderer,

    pub window: Window,
}

impl Overlay {
    // only call once (TODO: enforce this?)
    pub async fn new(event_loop: &EventLoop<()>, window_rectangle: &Rectangle<usize>) -> Self {
        const TITLE: &str = "Bumper Robot Overlay";

        let window = WindowBuilder::new()
            .with_title(TITLE)
            .with_transparent(true)
            .with_always_on_top(true)
            .with_decorations(false)
            .with_position(Position::Physical(PhysicalPosition { x: window_rectangle.left as i32, y: window_rectangle.top as i32 } ))
            .with_inner_size(Size::Physical(PhysicalSize { width: window_rectangle.width as u32, height: window_rectangle.height as u32 }))
            .build(event_loop)
            .expect("Unable to build overlay window");

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            height: size.height,
            width: size.width,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        let mut rr = RectangleRenderer::new(&device, swap_chain_descriptor.format);

        rr.add_permanent(&Rectangle {
            left: 0.0, top: 0.0, width: 1.0, height: 1.0 },
            &[0.5, 0.25, 0.125]);

        rr.add_permanent(&ACTIVE_RECTANGLE, &[0.8, 0.3, 0.3]);

        Self {
            device,
            queue,
            swap_chain,
            rr,
            window,
            _surface: surface,
            _swap_chain_descriptor: swap_chain_descriptor,
        }
    }

    pub fn update(&mut self) {
        let rectangle_instances = self.rr.data();

        self.queue.write_buffer(&self.rr.instance_buffer, 0, bytemuck::cast_slice(&rectangle_instances));
    }

    pub fn add(&mut self, rectangle: &Rectangle<f32>) {
        self.rr.add(rectangle, &[0.0, 1.0, 0.0]);
    }

    pub fn clear(&mut self) {
        self.rr.clear();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor{ label: Some("Command Encoder (Render)")
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                        store: true
                    }
                }],
            depth_stencil_attachment: None,
            label: Some("Render Pass Descriptor")
        });

        // TODO: move to rr
        render_pass.set_pipeline(&self.rr.render_pipeline);
        render_pass.set_vertex_buffer(0, self.rr.vertex_buffer.slice(..));

        render_pass.set_vertex_buffer(1, self.rr.instance_buffer.slice(..));

        self.rr.render(&mut render_pass);

        drop(render_pass); // need to drop the encoder ref before calling finish

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
