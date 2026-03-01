use core::num::NonZero;
use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use bind_group::BindGroup;
use glam::{Mat4, Quat, Vec3, Vec4};
use material::{Material, MaterialParameter};
use material_shadercomp::{
    CAMERA_PARAMS_BIND_GROUP_INDEX, INSTANCE_PARAMS_BIND_GROUP_INDEX,
    MATERIAL_PARAMS_BIND_GROUP_INDEX,
};
use mesh::{IndexBuffer, Mesh, MeshTopology, VertexBuffer};
use serde::{Deserialize, Serialize};
use tobj::{LoadError, LoadOptions};
use types::{GVec3, ShaderBufferParameterType, ShaderOpaqueParameterType};
use wgpu::{BackendOptions, Backends, Color, ColorWrites, InstanceFlags, ShaderStages};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

mod bind_group;
mod material;
mod mesh;
mod types;

static TEAPOT: &[u8] = include_bytes!("teapot.obj");
static BUNNY: &[u8] = include_bytes!("bunny.obj");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Shader {
    #[serde(skip)]
    id: usize,
    name: String,
    vertex_attributes: Vec<ShaderVertexAttribute>,
    keywords: HashMap<String, ShaderKeyword>,
    user_params: Vec<ShaderParameter>,
    source: ShaderSource,
}

impl Shader {
    pub fn load_source(&mut self) {
        if let ShaderSource::File { path } = &self.source {
            let content = std::fs::read_to_string(path).unwrap();
            self.source = ShaderSource::Inline { content };
        }
    }

    pub fn get_source(&self) -> &str {
        if let ShaderSource::Inline { content } = &self.source {
            content.as_str()
        } else {
            panic!("Invalid source");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShaderVertexAttribute {
    #[serde(flatten)]
    ty: ShaderVertexAttributeType,
    location: u32,
    condition: Option<ShaderParameterCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum ShaderVertexAttributeType {
    Position,
    Uv { channel: u8 },
}

impl core::fmt::Display for ShaderVertexAttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Position => "Position".fmt(f),
            Self::Uv { channel } => write!(f, "UV{}", channel),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShaderKeyword {
    default: u64,
    allowed: RangeInclusive<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
enum ShaderParameter {
    Buffer {
        #[serde(rename = "type")]
        ty: ShaderBufferParameterType,

        name: String,

        condition: Option<ShaderParameterCondition>,
    },
    Opaque {
        #[serde(rename = "type")]
        ty: ShaderOpaqueParameterType,

        name: String,

        condition: Option<ShaderParameterCondition>,
    },
}

impl ShaderParameter {
    pub fn get_condition(&self) -> Option<&ShaderParameterCondition> {
        match self {
            Self::Buffer { condition, .. } => condition.as_ref(),
            Self::Opaque { condition, .. } => condition.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
enum ShaderSource {
    Inline { content: String },
    File { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct ShaderParameterCondition(pub(crate) String);

fn params_to_buf_iter<'a>(
    params: impl IntoIterator<Item = &'a ShaderParameter>,
) -> impl IntoIterator<Item = ShaderBufferParameterType> {
    params.into_iter().filter_map(|p| {
        if let ShaderParameter::Buffer { ty, .. } = p {
            Some(*ty)
        } else {
            None
        }
    })
}
fn main() {
    let mut app = App::default();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    instance: Option<wgpu::Instance>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    cam_bindgroup: Option<BindGroup>,
    instance_bindgroup: Option<BindGroup>,
    unlit_mat: Option<Material>,
    unlit_pipeline: Option<wgpu::RenderPipeline>,
    teapot_mesh: Option<Mesh>,
    bunny_mesh: Option<Mesh>,
    configured: bool,
    start: Option<Instant>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        self.window = Some(Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_fullscreen(None))
                .unwrap(),
        ));

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::VULKAN,
            flags: if cfg!(debug_assertions) {
                InstanceFlags::advanced_debugging()
            } else {
                InstanceFlags::VALIDATION_INDIRECT_CALL
            },
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .unwrap();

        let adapter_info = adapter.get_info();

        println!(
            "Using graphics device '{}' with backend '{}' and driver '{} {}'",
            adapter_info.name, adapter_info.backend, adapter_info.driver, adapter_info.driver_info
        );

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("WutEngine Main GPU"),
            ..Default::default()
        }))
        .unwrap();

        let surface = instance
            .create_surface(self.window.as_ref().cloned().unwrap())
            .unwrap();

        let camera_params = [
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "view".to_string(),
                condition: None,
            },
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "projection".to_string(),
                condition: None,
            },
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "vp".to_string(),
                condition: None,
            },
        ];

        self.start = Some(Instant::now());

        let camera_layout: wgpu::BindGroupLayout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            NonZero::new(BindGroup::total_buffer_size(params_to_buf_iter(
                                &camera_params,
                            )) as u64)
                            .unwrap(),
                        ),
                    },
                    count: None,
                }],
            });

        let camera_bind_group = BindGroup::new("Camera".to_string(), camera_layout, &camera_params);

        let instance_params = [
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "model".to_string(),
                condition: None,
            },
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "mvp".to_string(),
                condition: None,
            },
        ];

        let instance_layout: wgpu::BindGroupLayout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Instance layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            NonZero::new(BindGroup::total_buffer_size(params_to_buf_iter(
                                &instance_params,
                            )) as u64)
                            .unwrap(),
                        ),
                    },
                    count: None,
                }],
            });

        let instance_bind_group =
            BindGroup::new("Instance".to_string(), instance_layout, &instance_params);

        let mut desc: Shader = serde_json::from_str(include_str!("unlit.json")).unwrap();

        dbg!(&desc);

        desc.load_source();

        let mut keywords = HashMap::default();

        keywords.insert("HAS_COLOR_MAP".to_owned(), 0);

        let mut material = Material::new(desc, keywords, &device);

        material
            .user_bind_group
            .set_parameter(
                "base_color",
                MaterialParameter::Vec4(Vec4::new(0.3, 0.3, 0.6, 1.0)),
                &queue,
            )
            .unwrap();

        println!("Starting read");
        let mut teapot_reader = BufReader::new(TEAPOT);
        let (teapot_model, _) =
            tobj::load_obj_buf(&mut teapot_reader, &LoadOptions::default(), |_| {
                Err(LoadError::ReadError)
            })
            .unwrap();

        let teapot_mesh = create_mesh(teapot_model[0].mesh.clone(), &device);

        let mut bunny_reader = BufReader::new(BUNNY);
        let (bunny_model, _) =
            tobj::load_obj_buf(&mut bunny_reader, &LoadOptions::default(), |_| {
                Err(LoadError::ReadError)
            })
            .unwrap();

        let bunny_mesh = create_mesh(bunny_model[0].mesh.clone(), &device);
        println!("Done");

        self.instance = Some(instance);
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.cam_bindgroup = Some(camera_bind_group);
        self.instance_bindgroup = Some(instance_bind_group);
        self.unlit_mat = Some(material);
        self.teapot_mesh = Some(teapot_mesh);
        self.bunny_mesh = Some(bunny_mesh);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if event == winit::event::WindowEvent::CloseRequested {
            event_loop.exit()
        }

        if matches!(event, WindowEvent::Resized(_)) {
            self.configured = false;
            self.about_to_wait(event_loop);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let win_size = self.window.as_ref().unwrap().inner_size();

        if let Some(srfc) = &self.surface
            && !self.configured
        {
            let caps = srfc.get_capabilities(self.adapter.as_ref().unwrap());

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: *caps.formats.first().unwrap(),
                width: win_size.width,
                height: win_size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
            };

            srfc.configure(self.device.as_ref().unwrap(), &config);
            self.configured = true;
        }

        assert!(self.configured);

        let output = self
            .surface
            .as_ref()
            .unwrap()
            .get_current_texture()
            .unwrap();

        let out_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        if self.unlit_pipeline.is_none() {
            let device = self.device.as_ref().unwrap();
            let camera_bind_group = self.cam_bindgroup.as_ref().unwrap();
            let instance_bind_group = self.instance_bindgroup.as_ref().unwrap();
            let material = self.unlit_mat.as_ref().unwrap();

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Unlit pipeline layout"),
                bind_group_layouts: &sort_layouts(
                    &camera_bind_group.layout,
                    &material.user_bind_group.layout,
                    &instance_bind_group.layout,
                ),
                immediate_size: 0,
            });

            let shm = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Unlit Shader Module"),
                source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                    (*material.compiled.module).clone(),
                )),
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shm,
                    entry_point: None,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: (size_of::<f32>() * 3) as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shm,
                    entry_point: None,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: out_view.texture().format(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

            self.unlit_pipeline = Some(pipeline);
        }

        let cam_bindgroup = self.cam_bindgroup.as_mut().unwrap();
        let instance_bindgroup = self.instance_bindgroup.as_mut().unwrap();
        let time = Instant::now()
            .duration_since(self.start.unwrap())
            .as_secs_f32();

        update_bind_groups(
            win_size.into(),
            cam_bindgroup,
            instance_bindgroup,
            self.queue.as_ref().unwrap(),
            time,
        );

        let unlit_bindgroup = &mut self.unlit_mat.as_mut().unwrap().user_bind_group;

        unlit_bindgroup
            .set_parameter(
                "base_color",
                MaterialParameter::Vec4(
                    Vec3::new(
                        f32::sin(time) + 1.3,
                        f32::sin(time * 5.0) + 1.3,
                        f32::sin(time * 3.0) + 1.3,
                    )
                    .normalize()
                    .extend(1.0),
                ),
                self.queue.as_ref().unwrap(),
            )
            .unwrap();

        cam_bindgroup.update_bind_group(self.device.as_ref().unwrap());
        instance_bindgroup.update_bind_group(self.device.as_ref().unwrap());
        unlit_bindgroup.update_bind_group(self.device.as_ref().unwrap());

        let mut encoder =
            self.device
                .as_ref()
                .unwrap()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Encoder"),
                });

        // @ Camera init

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &out_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_bind_group(
            CAMERA_PARAMS_BIND_GROUP_INDEX,
            Some(self.cam_bindgroup.as_ref().unwrap().get_bind_group()),
            &[],
        );

        // @ Material change
        {
            render_pass.set_pipeline(self.unlit_pipeline.as_ref().unwrap());

            render_pass.set_bind_group(
                MATERIAL_PARAMS_BIND_GROUP_INDEX,
                Some(
                    self.unlit_mat
                        .as_ref()
                        .unwrap()
                        .user_bind_group
                        .get_bind_group(),
                ),
                &[],
            );
        }

        // @ Draw call
        {
            render_pass.set_bind_group(
                INSTANCE_PARAMS_BIND_GROUP_INDEX,
                Some(self.instance_bindgroup.as_ref().unwrap().get_bind_group()),
                &[],
            );

            let mesh = self.bunny_mesh.as_ref().unwrap();
            let attrs = &self.unlit_mat.as_ref().unwrap().compiled.vertex_attributes;

            for (attribute, &location) in attrs {
                let vertex_buffer = mesh
                    .vertex_buffers
                    .get(attribute)
                    .expect("Missing attribute on mesh");

                render_pass.set_vertex_buffer(location, vertex_buffer.buffer.slice(..));
            }

            render_pass.set_index_buffer(
                mesh.index_buffer.buffer.slice(..),
                mesh.index_buffer.format.to_wgpu(),
            );

            render_pass.draw_indexed(0..mesh.index_buffer.count as u32, 0, 0..1);
        }

        drop(render_pass);

        self.queue
            .as_ref()
            .unwrap()
            .submit(core::iter::once(encoder.finish()));

        output.present();
    }
}

fn create_mesh(obj: tobj::Mesh, device: &wgpu::Device) -> Mesh {
    let mut vertex_buffers = HashMap::new();

    let positions_flat_slice = obj.positions.as_slice();

    assert!(positions_flat_slice.len().is_multiple_of(3));

    let positions_vec_slice = bytemuck::cast_slice::<_, GVec3<f32>>(positions_flat_slice);

    assert_eq!(
        core::mem::size_of_val(positions_flat_slice),
        core::mem::size_of_val(positions_vec_slice)
    );

    let pos_buffer = VertexBuffer::new(
        positions_vec_slice,
        ShaderVertexAttributeType::Position,
        device,
        false,
    )
    .unwrap();

    vertex_buffers.insert(ShaderVertexAttributeType::Position, pos_buffer);

    let index_slice = obj.indices.as_slice();

    assert!(
        index_slice
            .len()
            .is_multiple_of(MeshTopology::Triangle.indices_per_primitive())
    );

    let index_buffer = IndexBuffer::new(index_slice, MeshTopology::Triangle, device).unwrap();

    Mesh {
        vertex_buffers,
        index_buffer,
    }
}

#[inline]
fn sort_layouts<'a>(
    cam: &'a wgpu::BindGroupLayout,
    mat: &'a wgpu::BindGroupLayout,
    instance: &'a wgpu::BindGroupLayout,
) -> [&'a wgpu::BindGroupLayout; 3] {
    core::array::from_fn(|i| match i as u32 {
        CAMERA_PARAMS_BIND_GROUP_INDEX => cam,
        MATERIAL_PARAMS_BIND_GROUP_INDEX => mat,
        INSTANCE_PARAMS_BIND_GROUP_INDEX => instance,
        _ => unreachable!(),
    })
}

fn update_bind_groups(
    window_size: (u32, u32),
    cam_bind_group: &mut BindGroup,
    instance_bind_group: &mut BindGroup,
    queue: &wgpu::Queue,
    time: f32,
) {
    let view_mat = Mat4::from_translation(Vec3::new(0.0, 0.0, -10.0)).inverse();
    let projection_mat = Mat4::perspective_lh(
        f32::to_radians(70.0),
        window_size.0 as f32 / window_size.1 as f32,
        0.0001,
        10000.0,
    );

    let model_mat = Mat4::from_scale_rotation_translation(
        Vec3::ONE * 5.0,
        Quat::from_euler(glam::EulerRot::XYZ, time * 1.0, time * 5.0, time * 0.5),
        Vec3::new(0.0, 0.0, 3.0),
    );

    let vp = projection_mat * view_mat;
    let mvp = vp * model_mat;

    cam_bind_group
        .set_parameter("view", MaterialParameter::Mat4(view_mat), queue)
        .unwrap();
    cam_bind_group
        .set_parameter("projection", MaterialParameter::Mat4(projection_mat), queue)
        .unwrap();
    cam_bind_group
        .set_parameter("vp", MaterialParameter::Mat4(vp), queue)
        .unwrap();

    instance_bind_group
        .set_parameter("model", MaterialParameter::Mat4(model_mat), queue)
        .unwrap();
    instance_bind_group
        .set_parameter("mvp", MaterialParameter::Mat4(mvp), queue)
        .unwrap();
}

#[derive(Debug)]
struct CompiledShader {
    pub module: Box<naga::Module>,
    pub source_id_hash: u64,
    pub keyword_hash: u64,
    pub user_bind_group_layout: wgpu::BindGroupLayout,
    pub vertex_attributes: HashMap<ShaderVertexAttributeType, u32>,
}

struct FakeHasher;

impl material_shadercomp::ShaderHasher<u64> for FakeHasher {
    fn hash_source_id(id: u64) -> u64 {
        id
    }

    fn hash_keywords<S: AsRef<str>>(keywords: &HashMap<S, u64>) -> u64 {
        let mut kw_vec: Vec<(&str, u64)> = keywords.iter().map(|(k, v)| (k.as_ref(), *v)).collect();
        kw_vec.sort();

        let mut hash: u64 = 0;

        for (k, v) in kw_vec {
            for x in k.as_bytes().iter().map(|b| *b as u64) {
                hash = hash.wrapping_add(x);
            }

            hash = hash.wrapping_add(v)
        }

        hash
    }
}
