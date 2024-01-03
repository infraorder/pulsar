use bevy::{
    app::{App, Plugin},
    asset::{AssetServer, Handle},
    core::FrameCount,
    core_pipeline::{
        core_2d::{self, Camera2d, CORE_2D},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{QueryItem, With},
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
        world::{FromWorld, World},
    },
    reflect::Reflect,
    render::{
        camera::{Camera, ExtractedCamera},
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            BindGroupEntries, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
            BindingType, CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d,
            FilterMode, FragmentState, MultisampleState, Operations, PipelineCache, PrimitiveState,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, Shader, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureDescriptor, TextureDimension, TextureFormat,
            TextureSampleType, TextureUsages, TextureViewDimension,
        },
        renderer::{RenderContext, RenderDevice},
        texture::{BevyDefault, CachedTexture, TextureCache},
        view::{ExtractedView, ViewTarget},
        ExtractSchedule, MainWorld, Render, RenderApp, RenderSet,
    },
};

// const PREPROCESS_FEEDBACK_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(2881092380479);

pub mod draw_2d_graph {
    pub mod node {
        pub const FEEDBACK: &str = "feedback";
    }
}

pub struct FeedbackPlugin;

impl Plugin for FeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FeedbackSettings>();

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<FeedbackPipeline>>()
            .add_systems(ExtractSchedule, extract_feedback_settings)
            .add_systems(
                Render,
                (
                    prepare_feedback_pipelines.in_set(RenderSet::Prepare),
                    prepare_feedback_history_textures.in_set(RenderSet::PrepareResources),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<FeedbackNode>>(
                CORE_2D,
                draw_2d_graph::node::FEEDBACK,
            )
            .add_render_graph_edges(
                CORE_2D,
                &[
                    core_2d::graph::node::MAIN_PASS,
                    draw_2d_graph::node::FEEDBACK,
                    core_2d::graph::node::TONEMAPPING,
                ],
            );
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<FeedbackPipeline>();
    }
}

#[derive(Bundle, Default)]
pub struct FeedbackBundle {
    pub settings: FeedbackSettings,
}

#[derive(Component, Reflect, Clone, Default)]
pub struct FeedbackSettings {
    pub reset: bool,
}

#[derive(Default)]
pub struct FeedbackNode;

impl ViewNode for FeedbackNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ViewTarget,
        &'static FeedbackHistoryTextures,
        &'static FeedbackPipelineId,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (camera, view_target, feedback_history_textures, feedback_pipeline_id): QueryItem<
            Self::ViewQuery,
        >,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (Some(pipelines), Some(pipeline_cache)) = (
            world.get_resource::<FeedbackPipeline>(),
            world.get_resource::<PipelineCache>(),
        ) else {
            return Ok(());
        };

        let (Some(feedback_pipeline),) =
            (pipeline_cache.get_render_pipeline(feedback_pipeline_id.0),)
        else {
            return Ok(());
        };

        let view_target = view_target.post_process_write();

        let feedback_bind_group = render_context.render_device().create_bind_group(
            "feedback_bind_group",
            &pipelines.feedback_bind_group_layout,
            &BindGroupEntries::sequential((
                view_target.source,
                &feedback_history_textures.read.default_view,
                &pipelines.nearest_sampler,
                &pipelines.linear_sampler,
            )),
        );

        {
            let mut feedback_pass =
                render_context.begin_tracked_render_pass(RenderPassDescriptor {
                    label: Some("feedback_pass"),
                    color_attachments: &[
                        Some(RenderPassColorAttachment {
                            view: view_target.destination,
                            resolve_target: None,
                            ops: Operations::default(),
                        }),
                        Some(RenderPassColorAttachment {
                            view: &feedback_history_textures.write.default_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        }),
                    ],
                    depth_stencil_attachment: None,
                });
            feedback_pass.set_render_pipeline(feedback_pipeline);
            feedback_pass.set_bind_group(0, &feedback_bind_group, &[]);
            if let Some(viewport) = camera.viewport.as_ref() {
                feedback_pass.set_camera_viewport(viewport);
            }
            feedback_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}

#[derive(Resource)]
struct FeedbackPipeline {
    shader: Handle<Shader>,
    feedback_bind_group_layout: BindGroupLayout,
    nearest_sampler: Sampler,
    linear_sampler: Sampler,
}

impl FromWorld for FeedbackPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let nearest_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("feedback_nearest_sampler"),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..SamplerDescriptor::default()
        });
        let linear_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("feedback_linear_sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..SamplerDescriptor::default()
        });

        let feedback_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("feedback_bind_group_layout"),
                entries: &[
                    // View target (read)
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // TAA History (read)
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Nearest sampler
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    // Linear sampler
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/feedback.wgsl");

        FeedbackPipeline {
            shader,
            feedback_bind_group_layout,
            nearest_sampler,
            linear_sampler,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct FeedbackPipelineKey {
    hdr: bool,
    reset: bool,
}

impl SpecializedRenderPipeline for FeedbackPipeline {
    type Key = FeedbackPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = vec![];

        let format = if key.hdr {
            shader_defs.push("TONEMAP".into());
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        if key.reset {
            shader_defs.push("RESET".into());
        }

        RenderPipelineDescriptor {
            label: Some("feedback_pipeline".into()),
            layout: vec![self.feedback_bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                entry_point: "feedback".into(),
                targets: vec![
                    Some(ColorTargetState {
                        format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    }),
                    Some(ColorTargetState {
                        format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    }),
                ],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct FeedbackHistoryTextures {
    write: CachedTexture,
    read: CachedTexture,
}

fn prepare_feedback_history_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    frame_count: Res<FrameCount>,
    views: Query<(Entity, &ExtractedCamera, &ExtractedView)>,
) {
    for (entity, camera, _) in &views {
        if let Some(physical_viewport_size) = camera.physical_viewport_size {
            let mut texture_descriptor = TextureDescriptor {
                label: None,
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: physical_viewport_size.x,
                    height: physical_viewport_size.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: ViewTarget::TEXTURE_FORMAT_HDR,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            };

            texture_descriptor.label = Some("feedback_history_1_texture");
            let history_1_texture = texture_cache.get(&render_device, texture_descriptor.clone());

            texture_descriptor.label = Some("feedback_history_2_texture");
            let history_2_texture = texture_cache.get(&render_device, texture_descriptor);

            let textures = if frame_count.0 % 2 == 0 {
                FeedbackHistoryTextures {
                    write: history_1_texture,
                    read: history_2_texture,
                }
            } else {
                FeedbackHistoryTextures {
                    write: history_2_texture,
                    read: history_1_texture,
                }
            };

            commands.entity(entity).insert(textures);
        }
    }
}

#[derive(Component)]
pub struct FeedbackPipelineId(CachedRenderPipelineId);

fn prepare_feedback_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<FeedbackPipeline>>,
    pipeline: Res<FeedbackPipeline>,
    views: Query<(Entity, &ExtractedView, &FeedbackSettings)>,
) {
    for (entity, view, feedback_settings) in &views {
        let mut pipeline_key = FeedbackPipelineKey {
            hdr: view.hdr,
            reset: feedback_settings.reset,
        };
        let pipeline_id = pipelines.specialize(&pipeline_cache, &pipeline, pipeline_key.clone());

        // Prepare non-reset pipeline anyways - it will be necessary next frame
        if pipeline_key.reset {
            pipeline_key.reset = false;
            pipelines.specialize(&pipeline_cache, &pipeline, pipeline_key);
        }

        commands
            .entity(entity)
            .insert(FeedbackPipelineId(pipeline_id));
    }
}

fn extract_feedback_settings(mut commands: Commands, mut main_world: ResMut<MainWorld>) {
    let mut cameras =
        main_world.query_filtered::<(Entity, &Camera, &mut FeedbackSettings), With<Camera2d>>();

    for (entity, camera, mut feedback_settings) in cameras.iter_mut(&mut main_world) {
        if camera.is_active {
            commands
                .get_or_spawn(entity)
                .insert(feedback_settings.clone());
            feedback_settings.reset = false;
        }
    }
}
