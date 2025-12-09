use std::sync::Arc;

use tao::window::Window;

#[allow(dead_code)]
pub struct BgSurface {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: tao::dpi::PhysicalSize<u32>,
    surface_format: wgpu::TextureFormat,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl BgSurface {
    pub async fn new(window: Arc<Window>) -> Self {
        // instance 变量是 GPU 实例
        // Backends::all 对应 Vulkan、Metal、DX12、WebGL 等所有后端图形驱动
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                // 不太清楚是否一定需要BGRA8UNORM_STORAGE,先加上，后续测试
                required_features: wgpu::Features::empty(),
                // WebGL 后端并不支持 wgpu 的所有功能，
                // 所以如果要以 web 为构建目标，就必须禁用一些功能。
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                //禁用实验性功能
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
                // 追踪 API 调用路径
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();
        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .find(|f| **f == wgpu::TextureFormat::Bgra8Unorm)
            .or_else(|| caps.formats.first())
            .copied()
            .unwrap();
        let mut size = window.inner_size();
        // 宽高不能为0否则会报错
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        let config = wgpu::SurfaceConfiguration {
            // usage 字段描述了 SurfaceTexture 如何被使用。RENDER_ATTACHMENT 指定将被用来渲染到屏幕的纹理
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            //显示设备的刷新率做为渲染的帧速率，这本质上就是垂直同步（VSync），所有平台都得支持这种呈现模式（PresentMode）
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fullscreen Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Frame Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Frame Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fullscreen Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fullscreen Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            surface_format,
            sampler,
            bind_group_layout,
            pipeline,
        }
    }

    pub fn render(
        &self,
        frame_data: &[u8],
        frame_width: u32,
        frame_height: u32,
        bytes_per_row: u32,
    ) -> Result<(), wgpu::SurfaceError> {
        //等待 surface 提供一个新的 SurfaceTexture。我们将它存储在 output 变量中以便后续使用。
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let frame_texture =
            self.device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Frame Texture"),
                    size: wgpu::Extent3d {
                        width: frame_width.max(1),
                        height: frame_height.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &frame_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            frame_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(frame_height.max(1)),
            },
            wgpu::Extent3d {
                width: frame_width.max(1),
                height: frame_height.max(1),
                depth_or_array_layers: 1,
            },
        );
        let frame_view = frame_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Frame Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&frame_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
        // 命令编码器（CommandEncoder）来记录实际的命令发送给 GPU。
        // 大多数现代图形框架希望命令在被发送到 GPU 之前存储在一个命令缓冲区中。
        // 命令编码器创建了一个命令缓冲区，然后我们可以将其发送给 GPU。
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}
