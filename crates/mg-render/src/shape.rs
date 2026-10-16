use bytemuck::{Pod, Zeroable};
use mg_tokens::shape::MaterialShape;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState, Buffer,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device, FragmentState,
    MultisampleState, Queue, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, TextureFormat, VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
};

pub const SHADER: &str = include_str!("shape.wgsl");

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

const INSTANCE_SIZE: usize = 96;
const SHAPE_BANK_VEC4_COUNT: usize = 35 * 32;
const SHAPE_BANK_BYTES: usize = SHAPE_BANK_VEC4_COUNT * 16;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ShapeInstance {
    pub rect: [f32; 4],
    pub fill_col: [f32; 4],
    pub stroke_col: [f32; 4],
    pub radii: [f32; 4],
    pub stroke_clip: [f32; 4],
    pub clip_size: [f32; 2],
    pub _pad: [f32; 2],
}

impl ShapeInstance {
    pub const fn empty() -> Self {
        Self {
            rect: [0.0; 4],
            fill_col: [0.0; 4],
            stroke_col: [0.0; 4],
            radii: [0.0; 4],
            stroke_clip: [0.0; 4],
            clip_size: [0.0; 2],
            _pad: [0.0; 2],
        }
    }
}

pub fn build_shape_bank_data() -> Vec<[f32; 4]> {
    let mut bank = Vec::with_capacity(SHAPE_BANK_VEC4_COUNT);
    for shape in MaterialShape::ALL {
        let contour = shape.contour_64();
        for pair in contour.as_chunks::<2>().0 {
            bank.push([pair[0][0], pair[0][1], pair[1][0], pair[1][1]]);
        }
    }
    debug_assert_eq!(bank.len(), SHAPE_BANK_VEC4_COUNT);
    bank
}

pub struct ShapePipeline {
    pipeline: RenderPipeline,
    screen_buf: Buffer,
    #[allow(dead_code)]
    shape_bank_buf: Buffer,
    screen_group: BindGroup,
    instance_buf: Buffer,
    instance_cap: usize,
    pub instances: Vec<ShapeInstance>,
}

impl ShapePipeline {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(SHADER.into()),
        });
        let attrs = [
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 1,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 2,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 3,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 64,
                shader_location: 4,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 80,
                shader_location: 5,
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 88,
                shader_location: 6,
            },
        ];
        let layout = VertexBufferLayout {
            array_stride: INSTANCE_SIZE as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &attrs,
        };
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[Some(layout)],
                compilation_options: Default::default(),
            },
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    // Fragment output is premultiplied linear color.
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let screen_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 16,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shape_bank_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            size: SHAPE_BANK_BYTES as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bank_data = build_shape_bank_data();
        queue.write_buffer(&shape_bank_buf, 0, bytemuck::cast_slice(&bank_data));

        let screen_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &screen_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &shape_bank_buf,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });
        let instance_cap = 512usize;
        let instance_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (instance_cap * INSTANCE_SIZE) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            screen_buf,
            shape_bank_buf,
            screen_group,
            instance_buf,
            instance_cap,
            instances: Vec::new(),
        }
    }

    pub fn set_screen(&mut self, queue: &Queue, w: f32, h: f32) {
        let data = [w, h, 0.0f32, 0.0f32];
        queue.write_buffer(&self.screen_buf, 0, bytemuck::bytes_of(&data));
    }

    pub fn upload(&mut self, device: &Device, queue: &Queue) -> u32 {
        let count = self.instances.len();
        if count == 0 {
            return 0;
        }
        if count > self.instance_cap {
            self.instance_cap = count.next_power_of_two();
            self.instance_buf = device.create_buffer(&BufferDescriptor {
                label: None,
                size: (self.instance_cap * INSTANCE_SIZE) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        let bytes: &[u8] = bytemuck::cast_slice(&self.instances);
        queue.write_buffer(&self.instance_buf, 0, bytes);
        count as u32
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.screen_group, &[]);
        pass.set_vertex_buffer(0, self.instance_buf.slice(..));
        pass.draw(0..6, 0..self.instances.len() as u32);
    }

    pub fn clip_last(&mut self, clip: [f32; 4]) {
        if let Some(last) = self.instances.last_mut() {
            last.stroke_clip[0] = clip[0];
            last.stroke_clip[1] = clip[1];
            last.clip_size = [clip[2], clip[3]];
        }
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_stride_is_96_bytes() {
        assert_eq!(std::mem::size_of::<ShapeInstance>(), 96);
        assert_eq!(std::mem::size_of::<ShapeInstance>(), INSTANCE_SIZE);
    }

    #[test]
    fn instance_is_zeroable_and_pod() {
        let a = ShapeInstance::empty();
        let b = a;
        assert_eq!(a.rect, b.rect);
        assert_eq!(a.clip_size, [0.0, 0.0]);
        let bytes: &[u8] = bytemuck::bytes_of(&a);
        assert_eq!(bytes.len(), 96);
    }

    #[test]
    fn shape_bank_has_expected_size() {
        let bank = build_shape_bank_data();
        assert_eq!(bank.len(), SHAPE_BANK_VEC4_COUNT);
        assert_eq!(bank.len() * 16, SHAPE_BANK_BYTES);
    }
}
