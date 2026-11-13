use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, OwnedDisplayHandle};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::x11::{WindowAttributesExtX11, WindowType};
use winit::window::{Window, WindowId};

use crate::gui::camera::{Camera, CameraController, CameraUniform};

struct StateCAS {
    mouse_left_held: bool,
    zoom: f32,
    target: (f32, f32),
}

impl Default for StateCAS {
    fn default() -> Self {
        Self {
            mouse_left_held: false,
            zoom: 1.0,
            target: (0.0, 0.0),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    coord: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

struct AppState {
    instance: wgpu::Instance,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    device: wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
    surface_format: wgpu::TextureFormat,
    render_pipeline: wgpu::RenderPipeline,

    screen_quad_vertices: [Vertex; 4],
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,

    start_time: std::time::Instant,
    state_cas: StateCAS,
}

impl AppState {
    async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> AppState {
        let state_cas = StateCAS::default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display),
        ));

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::wgt::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities.formats[0];

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let coords_range = (
            size.width as f32 / (100.0 * state_cas.zoom * 2.0),
            size.height as f32 / (100.0 * state_cas.zoom * 2.0),
        );

        #[rustfmt::skip]
        let screen_quad_vertices = [
            Vertex { position: [ -1.0, -1.0 ], coord: [state_cas.target.0 - coords_range.0, state_cas.target.1 - coords_range.1] },
            Vertex { position: [  1.0, -1.0 ], coord: [state_cas.target.0 + coords_range.0, state_cas.target.1 - coords_range.1] },
            Vertex { position: [ -1.0,  1.0 ], coord: [state_cas.target.0 - coords_range.0, state_cas.target.1 + coords_range.1] },
            Vertex { position: [  1.0,  1.0 ], coord: [state_cas.target.0 + coords_range.0, state_cas.target.1 + coords_range.1] },
        ];

        let indices: &[u16] = &[0, 1, 2, 1, 3, 2];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&screen_quad_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        use cgmath::Angle;
        let camera = Camera {
            eye: (0.0, 0.0, 2.0).into(),
            front: (
                cgmath::Deg(45.0).sin(),
                cgmath::Deg(-15.0).sin(),
                cgmath::Deg(45.0).cos(),
            )
                .into(),
            up: cgmath::Vector3::unit_y(),
            aspect: size.height as f32 / size.width as f32,
            fovy: cgmath::Deg(45.0).into(),
            znear: 0.1,
            zfar: 100.0,
        };

        let start_time = std::time::Instant::now();

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        camera_uniform.update_time(start_time.elapsed().as_secs_f32());
        // camera_uniform.update_window_size(size.width, size.height);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: &bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render pipeline layout"),
                bind_group_layouts: &[Some(&camera_bind_group_layout)],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(Vertex::desc())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let state = AppState {
            instance,
            window,
            surface,
            queue,
            device,
            size,
            surface_format,
            render_pipeline,

            screen_quad_vertices,
            vertex_buffer,
            index_buffer,
            num_indices,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller: CameraController::new(0.2),

            start_time,
            state_cas,
        };

        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: self.size.width,
            height: self.size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: [self.surface_format.add_srgb_suffix()].into(),
        };

        self.surface.configure(&self.device, &config);
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.size = size;
        self.configure_surface();
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.camera_controller.handle_key(code, is_pressed);
        }
    }

    fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_dx, dy) => {
                self.state_cas.zoom *= dy / 10.0 + 1.0;
                self.update_screen_quad_vertices();
            }
            MouseScrollDelta::PixelDelta(_) => todo!(),
        }
    }

    fn update_screen_quad_vertices(&mut self) {
        let coords_range = (
            self.size.width as f32 / (100.0 * self.state_cas.zoom * 2.0),
            self.size.height as f32 / (100.0 * self.state_cas.zoom * 2.0),
        );

        #[rustfmt::skip]
        let screen_quad_vertices = [
            Vertex { position: [ -1.0, -1.0 ], coord: [self.state_cas.target.0 - coords_range.0, self.state_cas.target.1 - coords_range.1] },
            Vertex { position: [  1.0, -1.0 ], coord: [self.state_cas.target.0 + coords_range.0, self.state_cas.target.1 - coords_range.1] },
            Vertex { position: [ -1.0,  1.0 ], coord: [self.state_cas.target.0 - coords_range.0, self.state_cas.target.1 + coords_range.1] },
            Vertex { position: [  1.0,  1.0 ], coord: [self.state_cas.target.0 + coords_range.0, self.state_cas.target.1 + coords_range.1] },
        ];

        self.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&screen_quad_vertices),
        );
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.camera_uniform
            .update_time(self.start_time.elapsed().as_secs_f32());
        self.camera_uniform.update_zoom(self.state_cas.zoom);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => unreachable!("no error scope registered"),
        };

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // render_pass.draw(0..6, 0..1);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(render_pass);

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(surface_texture);
    }
}

#[derive(Default)]
pub struct App {
    state_app: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_type = WindowType::Dialog;
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("cassiopeia")
                        .with_x11_window_type([window_type].into()),
                )
                .unwrap(),
        );

        self.state_app = Some(pollster::block_on(AppState::new(
            event_loop.owned_display_handle(),
            window,
        )));
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let state = if let Some(state) = &mut self.state_app {
            state
        } else {
            return;
        };
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                if state.state_cas.mouse_left_held {
                    state.state_cas.target.0 -= delta.0 as f32 / (100.0 * state.state_cas.zoom);
                    state.state_cas.target.1 += delta.1 as f32 / (100.0 * state.state_cas.zoom);
                    state.update_screen_quad_vertices();
                }
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let app_state = self.state_app.as_mut().unwrap();
        let state_cas = &mut app_state.state_cas;
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                app_state.update();
                app_state.render();
                app_state.get_window().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                app_state.resize(new_size);
            }
            WindowEvent::MouseInput {
                state,
                button,
                device_id: _,
            } => {
                if button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => state_cas.mouse_left_held = true,
                        ElementState::Released => state_cas.mouse_left_held = false,
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => app_state.handle_scroll(delta),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => app_state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::Focused(gained_focus) => {
                if !gained_focus {
                    state_cas.mouse_left_held = false;
                }
            }
            _ => (),
        }
    }
}
