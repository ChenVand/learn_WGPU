use std::{
    sync::Arc, 
    borrow::Cow, 
    mem
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use wgpu::util::DeviceExt;

struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor::default(),
                None, // Trace path
            )
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }
}

#[allow(dead_code)]
struct World {
    vertex_buf: Option<wgpu::Buffer>,
    num_vertices: usize,
    // index_buf: wgpu::Buffer,
    // index_count: usize,
    uniform_buf: Option<wgpu::Buffer>,
    storage_buf: Option<wgpu::Buffer>,
    grid_size: u32,
    bind_group: Option<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
}

impl World {
    fn new(
        // config: &wgpu::SurfaceConfiguration,
        surface_format: &wgpu::TextureFormat,
        // _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let grid_size: u32 = 32;

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid uniforms"),
            contents: bytemuck::cast_slice(&[grid_size as f32, grid_size as f32]),
            usage: wgpu::BufferUsages::UNIFORM, // | wgpu::BufferUsages::COPY_DST,
        });

        let vertices: &[f32] = &[
        //   X,    Y,
            -0.8, -0.8, // Triangle 1 (Blue)
            0.8, -0.8,
            0.8,  0.8,
        
            -0.8, -0.8, // Triangle 2 (Red)
            0.8,  0.8,
            -0.8,  0.8,
        ];

        // let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Cell vertices"),
        //     contents: bytemuck::cast_slice(&vertices),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });
        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cell vertices"),
            size: mem::size_of_val(vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buf, 0, bytemuck::cast_slice(vertices));

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: 2 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        // An array representing the active state of each cell.
        let mut cell_state_array = vec![0; (grid_size * grid_size) as usize];
        for i in (0..cell_state_array.len()).step_by(3) {
            cell_state_array[i] = 1;
        } 
        let cell_state_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cell state"),
            contents: bytemuck::cast_slice(&cell_state_array[..]),
            usage: wgpu::BufferUsages::STORAGE, // | wgpu::BufferUsages::COPY_DST,
        });

        // let cell_shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let cell_shader_module = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Cell shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            }
        );

        let cell_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Cell pipeline"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &cell_shader_module,
                    entry_point: Some("vertex_main"), //can be None because only 1
                    buffers: &[vertex_buffer_layout],
                    compilation_options: Default::default(),    
                },
                fragment: Some(wgpu::FragmentState {
                    module: &cell_shader_module,
                    entry_point: Some("fragment_main"), //can be None because only 1
                    // targets: &[Some(surface_format.clone().into())],
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format.clone(),
                        blend: None, // or another blend configuration
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cell renderer bind group"),
            layout: &cell_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: cell_state_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            vertex_buf: Some(vertex_buf),
            num_vertices: vertices.len() / 2,
            grid_size: grid_size,
            uniform_buf: Some(uniform_buf), //This is only a handle to the actual buffer
            storage_buf: Some(cell_state_buffer),
            bind_group: Some(bind_group),
            pipeline: cell_pipeline,
        }
    }

    fn render(&self, state: &mut State) {
        // Create texture view
        let surface_texture = state
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(state.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = state.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    load: wgpu::LoadOp::Clear(wgpu::Color {r: 0., g: 0., b: 0.3, a: 1.}),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If you wanted to call any drawing commands, they would go here.
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buf.as_ref().unwrap().slice(..));
        
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(
            0..self.num_vertices as u32, 
            0..(self.grid_size * self.grid_size)
        );

        // End the renderpass.
        drop(render_pass);

        // Submit the command in the queue to execute
        state.queue.submit([encoder.finish()]);
        surface_texture.present();
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
    world: Option<World>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        let state_ref = self.state.as_ref().unwrap();
        self.world = Some(World::new(
            &state_ref.surface_format, 
            &state_ref.device, 
            &state_ref.queue
        ));

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        let world = self.world.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                world.render(state);

                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always folloed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    //
    // To change the log level, set the `RUST_LOG` environment variable. See the `env_logger`
    // documentation for more information.
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // When the current loop iteration finishes, immediately begin a new
    // iteration regardless of whether or not new events are available to
    // process. Preferred for applications that want to render as fast as
    // possible, like games.
    event_loop.set_control_flow(ControlFlow::Poll);

    // When the current loop iteration finishes, suspend the thread until
    // another event arrives. Helps keeping CPU utilization low if nothing
    // is happening, which is preferred if the application might be idling in
    // the background.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}