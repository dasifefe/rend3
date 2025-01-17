use rend3::{RenderGraphNodeBuilder, Renderer};
use wgpu::TextureFormat;

pub struct EguiRenderRoutine {
    internal: egui_wgpu_backend::RenderPass,
    screen_descriptor: egui_wgpu_backend::ScreenDescriptor,
}

impl EguiRenderRoutine {
    pub fn new(
        renderer: &Renderer,
        surface_format: TextureFormat,
        msaa_samples: u32,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Self {
        let rpass = egui_wgpu_backend::RenderPass::new(&renderer.device, surface_format, msaa_samples);

        Self {
            internal: rpass,
            screen_descriptor: egui_wgpu_backend::ScreenDescriptor {
                physical_height: height,
                physical_width: width,
                scale_factor,
            },
        }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32, new_scale_factor: f32) {
        self.screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_height: new_height,
            physical_width: new_width,
            scale_factor: new_scale_factor,
        };
    }

    pub fn add_to_graph<'node>(&'node mut self, mut builder: RenderGraphNodeBuilder<'_, 'node>, input: Input<'node>) {
        let output_handle = builder.add_surface_output();

        builder.build(move |renderer, _prefix_cmd_bufs, cmd_bufs, _ready, texture_store| {
            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("egui command encoder"),
            });

            self.internal
                .update_texture(&renderer.device, &renderer.queue, &input.context.texture());
            self.internal.update_user_textures(&renderer.device, &renderer.queue);
            self.internal.update_buffers(
                &renderer.device,
                &renderer.queue,
                input.clipped_meshes,
                &self.screen_descriptor,
            );

            let output = texture_store.get_render_target(output_handle);

            self.internal
                .execute(
                    &mut encoder,
                    output,
                    input.clipped_meshes,
                    &self.screen_descriptor,
                    None,
                )
                .unwrap();

            cmd_bufs.send(encoder.finish()).unwrap();
        });
    }
}

pub struct Input<'a> {
    pub clipped_meshes: &'a Vec<egui::ClippedMesh>,
    pub context: egui::CtxRef,
}

impl epi::TextureAllocator for EguiRenderRoutine {
    fn alloc_srgba_premultiplied(&mut self, size: (usize, usize), srgba_pixels: &[egui::Color32]) -> egui::TextureId {
        self.internal.alloc_srgba_premultiplied(size, srgba_pixels)
    }

    fn free(&mut self, id: egui::TextureId) {
        self.internal.free(id);
    }
}
