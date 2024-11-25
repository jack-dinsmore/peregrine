pub mod model;
pub mod shader;
pub mod primitives;
pub mod object;
pub mod camera;

use camera::Camera;
use model::Texture;
use object::ObjectHandle;
use shader::Shader;
use wgpu::SurfaceConfiguration;
use winit::window::Window;

pub struct RenderPass<'a> {
    graphics: &'a Graphics<'a>,
    render_pass: wgpu::RenderPass<'a>,
    camera: Option<&'a Camera>,
    objects: Vec<ObjectHandle<'a>>,
    global_material: bool,
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
                        r: 0.01,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &graphics.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        Self {
            graphics,
            render_pass,
            camera: None,
            objects: Vec::new(),
            global_material: false,
        }
    }

    fn render_models(&mut self) {
        for object in self.objects.drain(0..self.objects.len()) {
            let object = object.as_ref();
            object.update(&self.graphics, self.camera.expect("You must set a camera"));
            self.render_pass.set_bind_group(1, &object.bind_group, &[]);
            // It is guaranteed that the model is borrowed for longer than this function, so move the lifetime of data up to 'a
            let model_data = &object.model.inner();
            for mesh in &model_data.0 {//TODO rearrange order
                if !self.global_material {
                    self.render_pass.set_bind_group(2, &model_data.1[mesh.material_id].bind_group, &[]);
                }
                self.render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                self.render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                self.render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }
        }
    }

    pub fn set_camera(&mut self, camera: &'a Camera) {
        camera.update(&self.graphics);
        self.camera = Some(camera);
    }

    pub fn set_shader(&mut self, shader: &'a Shader) {
        self.render_models();
        self.global_material = false;
        self.render_pass.set_pipeline(&shader.render_pipeline);
        self.render_pass.set_bind_group(0, &self.camera.expect("You must set a camera").bind_group, &[]);
    }
    
    pub fn set_global_texture(&mut self, texture: &Texture) {
        self.global_material = true;
        self.render_pass.set_bind_group(2, &texture.bind_group, &[]);
    }

    pub fn render(&mut self, objects: Vec<ObjectHandle<'a>>) {
        for object in objects {
            let index = match self.objects.binary_search_by(|probe| probe.as_ref().model.identifier().cmp(&object.as_ref().model.identifier())) {
                Ok(i) => i,
                Err(i) => i,
            };
            self.objects.insert(index, object);
        }
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
    pub size: (u32, u32),
    window: &'a Window,
    depth_texture_view: wgpu::TextureView,
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

    pub fn make_depth_texture(device: &wgpu::Device, config: &SurfaceConfiguration) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
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

    pub fn window(&self) -> &Window {
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

    pub fn render(&self, render_execute: impl FnOnce(RenderPass)) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        render_execute(RenderPass::new(self, &mut encoder, &view));
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }
    
    pub fn set_mouse_pos(&self, size: (u32, u32)) {
        self.window.set_cursor_position(winit::dpi::PhysicalPosition{x: size.0, y: size.1}).unwrap();
    }
}