pub mod model;
pub mod shader;
pub mod primitives;
pub mod object;
pub mod camera;

use std::sync::Arc;

use camera::Camera;
use model::{Material, Mesh};
use object::Object;
use shader::Shader;
use winit::window::Window;

use crate::App;

pub struct RenderPass<'a> {
    graphics: &'a Graphics<'a>,
    render_pass: wgpu::RenderPass<'a>,
    camera: Option<&'a Camera>,
    objects: Vec<&'a Object>,
}

impl<'a> RenderPass<'a> {
    fn new<'b: 'a>(graphics: &'a Graphics, encoder: &'b mut wgpu::CommandEncoder, view: &'a wgpu::TextureView) -> Self {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        Self {
            graphics,
            render_pass,
            camera: None,
            objects: Vec::new(),
        }
    }

    fn render_models(&mut self) {
        for object in self.objects.drain(0..self.objects.len()) {
            object.update(&self.graphics, self.camera.expect("You must set a camera"));
            self.render_pass.set_bind_group(1, &object.bind_group, &[]);
            // It is guaranteed that the model is borrowed for longer than this function, so move the lifetime of data up to 'a
            let model_data: &'a (Vec<Mesh>, Vec<Material>) = unsafe {
                &*Arc::as_ptr(&object.model.model_data)
            };
            for mesh in &model_data.0 {//TODO rearrange order
                self.render_pass.set_bind_group(2, &model_data.1[mesh.material_id].bind_group, &[]);
                self.render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                self.render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                self.render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }
        }
    }

    pub fn set_camera(mut self, camera: &'a Camera) -> Self {
        camera.update(&self.graphics);
        self.camera = Some(camera);
        self
    }

    pub fn set_shader(mut self, shader: &'a Shader) -> Self {
        self.render_models();
        self.render_pass.set_pipeline(&shader.render_pipeline);
        self.render_pass.set_bind_group(0, &self.camera.expect("You must set a camera").bind_group, &[]);
        self
    }

    pub fn render(mut self, objects: &[&'a Object]) -> Self {
        for object in objects {
            let index = match self.objects.binary_search_by(|probe| probe.model.cmp(&object.model)){
                Ok(i) => i,
                Err(i) => i,
            };
            self.objects.insert(index, object);
        }
        self
    }
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        self.render_models();
    }
}


pub struct Graphics<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
}

impl<'a> Graphics<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> Self {
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
        
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, app: &impl App) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        app.render(RenderPass::new(self, &mut encoder, &view));
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }
}