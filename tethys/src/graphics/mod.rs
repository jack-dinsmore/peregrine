pub mod model;
pub mod shader;
pub mod primitives;
pub mod object;
pub mod camera;
pub mod render_pass;

use wgpu::{CommandEncoder, SurfaceConfiguration, SurfaceTexture, TextureView};
use winit::window::Window;

pub struct Graphics<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: (u32, u32),
    window: &'a Window,
    depth_texture_view: wgpu::TextureView,
}

impl<'a> Graphics<'a> {
    // Creating some of the wgpu types requires async code
    pub(crate) async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let (_depth_texture, depth_texture_view, _depth_sampler) = Self::make_depth_texture(&device, &config);

        
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size: (size.width, size.height),
            depth_texture_view,
        }
    }

    pub(crate) fn make_depth_texture(device: &wgpu::Device, config: &SurfaceConfiguration) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let size = wgpu::Extent3d { // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );
        (texture, view, sampler)
    }

    pub(crate) fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub(crate) fn start_render(&self) -> anyhow::Result<(SurfaceTexture, TextureView)> {
        let texture = self.surface.get_current_texture()?;
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        Ok((texture, view))
    }

    pub(crate) fn make_encoder(&self) -> CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
    }

    pub(crate) fn make_render_pass<'b>(&self, view: &TextureView, encoder: &'b mut CommandEncoder, clear_color: bool) -> wgpu::RenderPass<'b> {
        let color_load_op = match clear_color {
            true => wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.01,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            false => wgpu::LoadOp::Load,
        };
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: color_load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass
    }

    pub(crate) fn queue_encoder(&self, encoder: CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }
    
    pub fn set_mouse_pos(&self, size: (u32, u32)) {
        self.window.set_cursor_position(winit::dpi::PhysicalPosition{x: size.0, y: size.1}).unwrap();
    }
}