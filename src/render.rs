use std::{borrow::Cow, future::Future, sync::Arc};
use wgpu::{BufferBinding, CurrentSurfaceTexture, util::DeviceExt};
use winit::{
    application::ApplicationHandler, dpi::{LogicalSize, PhysicalSize}, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowButtons}
};
use crate::Export;

use std::num::NonZero;

/// Runs a future to completion. On native this blocks synchronously via pollster.
/// On wasm this spawns a local task so control returns to the browser immediately.
#[cfg(not(target_arch = "wasm32"))]
fn spawn(f: impl Future<Output = ()> + 'static) {
    pollster::block_on(f);
}

struct WgpuState {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    rbind_group: wgpu::BindGroup,
    texture_view: wgpu::TextureView,

}

pub enum TriangleAction {
    Initialized(WgpuState),
}

#[expect(clippy::large_enum_variant)]
enum AppState {
    Uninitialized,
    Loading,
    Running(WgpuState),
}

pub struct App<T: Export> {
    proxy: EventLoopProxy<TriangleAction>,
    window: Option<Arc<Window>>,
    state: AppState,
    uni: T
}



impl<T: Export> App<T> {
    pub fn new(event_loop: &EventLoop<TriangleAction>, uni: T) -> Self 
    {
        Self {
            proxy: event_loop.create_proxy(),
            window: None,
            state: AppState::Uninitialized,
            uni: uni
        }
    }
    
}

impl<T:Export> ApplicationHandler<TriangleAction> for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Uninitialized) {
            return;
        }
        self.state = AppState::Loading;

        #[cfg_attr(
            not(target_arch = "wasm32"),
            expect(unused_mut, reason = "wasm32 re-assigns to specify canvas")
        )]
        let mut attributes = Window::default_attributes();

        attributes = attributes.with_min_inner_size((PhysicalSize::new(1000, 800)));
       // attributes = attributes.with_max_inner_size((PhysicalSize::new(800, 800)));
        attributes = attributes.with_resizable(true);
        attributes = attributes.with_title("The universe in my puta(hahah)");
        attributes = attributes.with_enabled_buttons(WindowButtons::CLOSE);
        


        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Failed to create window"),
        );
        self.window = Some(window.clone());

        let display_handle = event_loop.owned_display_handle();
        let proxy = self.proxy.clone();

        spawn(async move {
            let mut size = window.inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);

            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
                    Box::new(display_handle),
                ));

            let surface = instance.create_surface(window.clone()).unwrap();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    // Request an adapter which can render to our surface
                    compatible_surface: Some(&surface),
                })
                .await
                .expect("Failed to find an appropriate adapter");

            // Create the logical device and command queue
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter,
                    // so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    trace: wgpu::Trace::Off,
                })
                .await
                .expect("Failed to create device");

            // Load the shaders from disk
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });
            let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
            });

            

            let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly, // We only want to write
                            format: wgpu::TextureFormat::Rgba8Unorm,       // Must match the texture!
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                            has_dynamic_offset: false, 
                            min_binding_size: NonZero::new(48)
                        },
                            
                        
                        count: None,
                        },
                    
                ],
                label: None,
            });
            let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    
                    
                ],
                label: None,
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&render_bind_group_layout)],
                immediate_size: 0,
                
            });
            let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[Some(&compute_bind_group_layout)],
                immediate_size: 0,
            });
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Compute Output Texture"),
                size: wgpu::Extent3d { width: 1000, height: 800, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm, // Standard for storage
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING, 
                view_formats: &[],
            });
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());


            let rbind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("render Bind Group"),
                layout: &render_bind_group_layout, // This is the layout we created in Step 2
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0, // This must match @binding(0) in WGSL
                        resource: wgpu::BindingResource::TextureView(&texture_view), // The view from Step 1
                    },
                ],
            });

            let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &compute_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

            let swapchain_capabilities = surface.get_capabilities(&adapter);
            let swapchain_format = swapchain_capabilities.formats[0];

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        
                     
                    ],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

            let config = surface
                .get_default_config(&adapter, size.width, size.height)
                .unwrap();
            surface.configure(&device, &config);

            let _ = proxy.send_event(TriangleAction::Initialized(WgpuState {
                instance,
                window,
                device,
                queue,
                surface,
                config,
                render_pipeline,
                compute_pipeline,
                bind_group_layout:compute_bind_group_layout,
                rbind_group,
                texture_view,

            }));
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: TriangleAction) {
        
        match event {
            TriangleAction::Initialized(wgpu_state) => {
                self.state = AppState::Running(wgpu_state);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }

    

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let AppState::Running(wgpu_state) = &mut self.state else {
            return;
        };
        match event {
            
            WindowEvent::Resized(new_size) => {
                // Reconfigure the surface with the new size
                wgpu_state.config.width = new_size.width.max(1);
                wgpu_state.config.height = new_size.height.max(1);
                wgpu_state
                    .surface
                    .configure(&wgpu_state.device, &wgpu_state.config);
                // On macos the window needs to be redrawn manually after resizing
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.uni.update();
                let frame = match wgpu_state.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(frame) => frame,
                    CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                        // Try again later
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                    CurrentSurfaceTexture::Suboptimal(_) | CurrentSurfaceTexture::Outdated => {
                        wgpu_state
                            .surface
                            .configure(&wgpu_state.device, &wgpu_state.config);
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                    CurrentSurfaceTexture::Validation => {
                        return;
                        //unreachable!("No error scope registered, so validation errors will panic")
                    }
                    CurrentSurfaceTexture::Lost => {
                        wgpu_state.surface = wgpu_state
                            .instance
                            .create_surface(wgpu_state.window.clone())
                            .unwrap();
                        wgpu_state
                            .surface
                            .configure(&wgpu_state.device, &wgpu_state.config);
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                };
                


                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = wgpu_state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                      let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: None,
                        timestamp_writes: None,
                    });
                    cpass.set_pipeline(&wgpu_state.compute_pipeline);


                  

                    let v= self.uni.export_stars();
                    let test_buffer = wgpu_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Simulation Parameter Buffer"),
                        contents: bytemuck::cast_slice(&v),
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    });

                    let bind_group = wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Compute Bind Group"),
                        layout: &wgpu_state.bind_group_layout, // This is the layout we created in Step 2
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0, // This must match @binding(0) in WGSL
                                resource: wgpu::BindingResource::TextureView(&wgpu_state.texture_view), // The view from Step 1
                            },
                            wgpu::BindGroupEntry {
                                binding: 1, // This must match @binding(0) in WGSL
                                resource: test_buffer.as_entire_binding(), // The view from Step 1
                            },
                        ],
                    });



                    cpass.set_bind_group(0, &bind_group, &[]);
                    cpass.dispatch_workgroups(800, 600, 1);
                }
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    rpass.set_pipeline(&wgpu_state.render_pipeline);

                 
                    rpass.set_bind_group(0, &wgpu_state.rbind_group, &[]);


                    

                    rpass.draw(0..3, 0..2);
                }



                wgpu_state.queue.submit(Some(encoder.finish()));
                if let Some(window) = &self.window {
                    window.pre_present_notify();
                }
                frame.present();
            }
            WindowEvent::Occluded(is_occluded) => {
                if !is_occluded {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
             WindowEvent::KeyboardInput { device_id:_, event, is_synthetic:_ } =>{
                //self.cube.scene_loop(1.);

                match event.physical_key{
                    
                    _=>{}
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }

            }
            _ => {
                
                
                if let Some(window) = &self.window {
                    window.request_redraw();
                }

               
            }
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
