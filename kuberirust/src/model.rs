use anyhow::*;
use std::ops::Range;
use std::path::Path;
use wgpu::util::DeviceExt;

use crate::texture;

use rand::Rng;

use std::collections::HashMap;

use cgmath::Vector3;
use cgmath::Vector2;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex {
    position: cgmath::Vector3<f32>,
    tex_coords: cgmath::Vector2<f32>,
    //normal: cgmath::Vector3<f32>,
    //tangent: cgmath::Vector3<f32>,
    //bitangent: cgmath::Vector3<f32>,
}

unsafe impl bytemuck::Zeroable for ModelVertex {}
unsafe impl bytemuck::Pod for ModelVertex {}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
                // Tangent and bitangent
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    //pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        //normal_texture: texture::Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                /*
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },*/
            ],
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            //normal_texture,
            bind_group,
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    //pub name: String,
    pub blocktype: BlockType,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indexes: u32,
    //pub material: usize,
    pub instances_buffer: wgpu::Buffer,
    pub uniform_bind_group_instances: wgpu::BindGroup,
    pub num_instances: u32,
}

pub struct World{
    pub chunks: HashMap<[u8;3], Chunk>,
}


#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum QuadType {
    GRASS_TOP,
    GRASS_SIDE,
    DIRT,
    STONE,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BlockType {
    GRASS,
    DIRT,
    STONE,
}

pub struct Block {
    pub blocktype : BlockType,
}

pub struct Chunk {
    pub blocks: HashMap<[u8;3], Block>,
}

const CHUNKSIZE: u8 = 16;

#[derive(PartialEq, Eq, Hash)]
pub enum UV {
    MIN,
    MAX,
}

#[derive(PartialEq, Eq, Hash)]
pub struct UVQuadKey{
    quadtype: QuadType,
    uv: UV
}

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 10, 11, 8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
];

struct Instance {
    position: cgmath::Vector3<f32>,
    //rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: cgmath::Matrix4::from_translation(self.position)
                //* cgmath::Matrix4::from(self.rotation),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceRaw {
    model: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

pub struct Model {
    pub meshes: Vec<Mesh>,
    //pub materials: Vec<Material>,
    pub material: Option<Material>,
    pub world : World,

}

impl Model {
    fn build_random_chunk(&self)->Chunk
    {
        //Generate random chunk
        let mut chunk = Chunk{ blocks : HashMap::new(),};
        let mut rng = rand::thread_rng();
        for k in 0..CHUNKSIZE {
            for l in 0..CHUNKSIZE {
                for m in 0..CHUNKSIZE {
                    let val = rng.gen_range(0, 10);
                    if val<8
                    {
                        //Add block
                        chunk.blocks.insert( [k, l, m], Block{blocktype:BlockType::GRASS});
                    }
                }
            }
        }
        chunk
    }


    fn create_vertices(&self, blocktype:BlockType) -> Vec<ModelVertex>{//(Vec<Vertex>, Vec<u16>) {
        fn build_vertex(position:[i8;3], quadtype:QuadType, u:UV, v:UV)->ModelVertex
        {
            let mut umap: HashMap<UVQuadKey, f32> = HashMap::new();
            umap.insert(UVQuadKey{quadtype:QuadType::GRASS_TOP, uv:UV::MIN}, 0.125); umap.insert(UVQuadKey{quadtype:QuadType::GRASS_TOP, uv:UV::MAX}, 0.1875);
            umap.insert(UVQuadKey{quadtype:QuadType::GRASS_SIDE, uv:UV::MIN}, 0.1875); umap.insert(UVQuadKey{quadtype:QuadType::GRASS_SIDE, uv:UV::MAX}, 0.25);
            umap.insert(UVQuadKey{quadtype:QuadType::DIRT, uv:UV::MIN}, 0.125); umap.insert(UVQuadKey{quadtype:QuadType::DIRT, uv:UV::MAX}, 0.1875);
            umap.insert(UVQuadKey{quadtype:QuadType::STONE, uv:UV::MIN}, 0.0); umap.insert(UVQuadKey{quadtype:QuadType::STONE, uv:UV::MAX}, 0.0625);
        
            let mut vmap: HashMap<UVQuadKey, f32> = HashMap::new();
            vmap.insert(UVQuadKey{quadtype:QuadType::GRASS_TOP, uv:UV::MIN}, 0.375); vmap.insert(UVQuadKey{quadtype:QuadType::GRASS_TOP, uv:UV::MAX}, 0.4375);
            vmap.insert(UVQuadKey{quadtype:QuadType::GRASS_SIDE, uv:UV::MIN}, 0.9375); vmap.insert(UVQuadKey{quadtype:QuadType::GRASS_SIDE, uv:UV::MAX}, 1.0);
            vmap.insert(UVQuadKey{quadtype:QuadType::DIRT, uv:UV::MIN}, 0.9375); vmap.insert(UVQuadKey{quadtype:QuadType::DIRT, uv:UV::MAX}, 1.0);
            vmap.insert(UVQuadKey{quadtype:QuadType::STONE, uv:UV::MIN}, 0.875); vmap.insert(UVQuadKey{quadtype:QuadType::STONE, uv:UV::MAX}, 0.9375);
          
            let u_pos = umap.get(&UVQuadKey{quadtype:quadtype, uv:u});
            match u_pos {
                Some(i) => {
                    let v_pos = vmap.get(&UVQuadKey{quadtype:quadtype, uv:v});
                    match v_pos {
                        Some(j) => {
                            //let gvtestpos:[f32;3]=[position[0] as f32, position[1] as f32, position[2] as f32];
                            let pos = Vector3::new(position[0] as f32, position[1] as f32, position[2] as f32);
                            let tex = Vector2::new(Clone::clone(u_pos.unwrap()), 1.0-Clone::clone(v_pos.unwrap()));
                            //Vertex{position:position, tex_coords:[Clone::clone(u_pos.unwrap()), Clone::clone(v_pos.unwrap())]}
                            ModelVertex{position:pos, tex_coords:tex}
                        },
                        None => panic!("two"),
                    }
                },
                None => panic!("one"),
            }
        }
    
    
        let mut quadtype:QuadType=QuadType::STONE;
        if blocktype == BlockType::DIRT
        {
            quadtype = QuadType::DIRT;
        }
        
        let mut vertex_data: Vec<ModelVertex>= Vec::new();
        
    
        // top (0, 0, 1)
        let mut temp_quadtype:QuadType=quadtype;   
        if blocktype==BlockType::GRASS
        {
            temp_quadtype = QuadType::GRASS_TOP;
        }
    
        vertex_data.push(build_vertex([0, 0, 1], temp_quadtype, UV::MIN, UV::MIN));
        vertex_data.push(build_vertex([1, 0, 1], temp_quadtype, UV::MAX, UV::MIN));
        vertex_data.push(build_vertex([1, 1, 1], temp_quadtype, UV::MAX, UV::MAX));
        vertex_data.push(build_vertex([0, 1, 1], temp_quadtype, UV::MIN, UV::MAX));
    
        // bottom (0, 0, -1) 
        temp_quadtype=quadtype;   
        if blocktype==BlockType::GRASS
        {
            temp_quadtype = QuadType::DIRT;
        }
    
        vertex_data.push(build_vertex([0, 1, 0], temp_quadtype, UV::MAX, UV::MIN));
        vertex_data.push(build_vertex([1, 1, 0], temp_quadtype, UV::MIN, UV::MIN));
        vertex_data.push(build_vertex([1, 0, 0], temp_quadtype, UV::MIN, UV::MAX));
        vertex_data.push(build_vertex([0, 0, 0], temp_quadtype, UV::MAX, UV::MAX));
    
        // right (1, 0, 0)
        temp_quadtype=quadtype;   
        if blocktype==BlockType::GRASS
        {
            temp_quadtype = QuadType::GRASS_SIDE;
        }
        vertex_data.push(build_vertex([1, 0, 0], temp_quadtype, UV::MIN, UV::MIN));
        vertex_data.push(build_vertex([1, 1, 0], temp_quadtype, UV::MAX, UV::MIN));
        vertex_data.push(build_vertex([1, 1, 1], temp_quadtype, UV::MAX, UV::MAX));
        vertex_data.push(build_vertex([1, 0, 1], temp_quadtype, UV::MIN, UV::MAX));
    
        // left (-1, 0, 0)
        temp_quadtype=quadtype;   
        if blocktype==BlockType::GRASS
        {
            temp_quadtype = QuadType::GRASS_SIDE;
        }
    
        vertex_data.push(build_vertex([0, 0, 1], temp_quadtype, UV::MIN, UV::MAX));
        vertex_data.push(build_vertex([0, 1, 1], temp_quadtype, UV::MAX, UV::MAX));
        vertex_data.push(build_vertex([0, 1, 0], temp_quadtype, UV::MAX, UV::MIN));
        vertex_data.push(build_vertex([0, 0, 0], temp_quadtype, UV::MIN, UV::MIN));
    
        // front (0, 1, 0)
        temp_quadtype=quadtype;   
        if blocktype==BlockType::GRASS
        {
            temp_quadtype = QuadType::GRASS_SIDE;
        }
    
        vertex_data.push(build_vertex([1, 1, 0], temp_quadtype, UV::MAX, UV::MIN));
        vertex_data.push(build_vertex([0, 1, 0], temp_quadtype, UV::MIN, UV::MIN));
        vertex_data.push(build_vertex([0, 1, 1], temp_quadtype, UV::MIN, UV::MAX));
        vertex_data.push(build_vertex([1, 1, 1], temp_quadtype, UV::MAX, UV::MAX));
    
        // back (0, -1, 0)
        temp_quadtype=quadtype;   
        if blocktype==BlockType::GRASS
        {
            temp_quadtype = QuadType::GRASS_SIDE;
        }
    
        vertex_data.push(build_vertex([1, 0, 1], temp_quadtype, UV::MAX, UV::MAX));
        vertex_data.push(build_vertex([0, 0, 1], temp_quadtype, UV::MIN, UV::MAX));
        vertex_data.push(build_vertex([0, 0, 0], temp_quadtype, UV::MIN, UV::MIN));
        vertex_data.push(build_vertex([1, 0, 0], temp_quadtype, UV::MAX, UV::MIN));
    
        vertex_data
    }
    
    pub fn new()-> Result<Self>{
        Ok(Self { meshes: Vec::new(), material:None, world: World{chunks:HashMap::new()} })
    }

    //pub fn load<P: AsRef<Path>>(
    pub fn load(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        //path: P,
    ){
        //load material
        //let res_dir = std::path::Path::new(env!("OUT_DIR")).join("res");
        //res_dir.join("cube.obj")
        //let mut materials = Vec::new();
        let diffuse_bytes = include_bytes!("blockatlas.jpg");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "blockatlas.jpg", false).unwrap();
        self.material = Some(Material::new(
            device,
            "blockatlas",
            diffuse_texture,
            //normal_texture,
            layout,
        ));

        //build world
        //let mut world = World{chunks:HashMap::new()};
        //First chunk,
        //trenger flere sef
        self.world.chunks.insert( [0, 0, 0], self.build_random_chunk());

        //Go through world and build meshes. One mesh for each blocktype
        let mut create_mesh_and_addto_model = |blocktype| {
            let vertices = self.create_vertices(blocktype);
            let  vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(CUBE_INDICES),
                usage: wgpu::BufferUsage::INDEX,
            });
            
            let create_instance = |x, y, z| {
                let position = cgmath::Vector3 {
                    x: x as f32,
                    y: y as f32,
                    z: z as f32,
                };
                Instance { position }
            };

            let mut instances:Vec<Instance>=Vec::new();
            for (chunkkey, chunk) in &self.world.chunks {
                for (blockkey, block) in &chunk.blocks {
                    //transler til rett plass. Må ta hensyn til flere chunks.
                    let x = (chunkkey[0] * CHUNKSIZE ) + blockkey[0];
                    let y = (chunkkey[1] * CHUNKSIZE ) + blockkey[1];
                    let z = (chunkkey[2] * CHUNKSIZE ) + blockkey[2];

                    instances.push(create_instance(x as f32, y as f32, z as f32));
                }
            }

            let uniform_bind_group_layout_instances =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::StorageBuffer {
                            // We don't plan on changing the size of this buffer
                            dynamic: false,
                            // The shader is not allowed to modify it's contents
                            readonly: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout_instances"),
            });
    
            let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
            let instances_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsage::STORAGE,
            });
                
            let uniform_bind_group_instances = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout_instances,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(instances_buffer.slice(..)),
                    },
                ],
                label: Some("uniform_bind_group_instances"),
            }); 
            
            self.meshes.push(Mesh{
                blocktype: blocktype, 
                vertex_buffer: vertex_buffer,
                index_buffer: index_buffer,
                num_indexes: CUBE_INDICES.len() as u32,
                instances_buffer: instances_buffer,
                uniform_bind_group_instances: uniform_bind_group_instances,
                num_instances: instances.len() as u32
            });


        };

        create_mesh_and_addto_model(BlockType::GRASS);
        create_mesh_and_addto_model(BlockType::DIRT);
        create_mesh_and_addto_model(BlockType::STONE);
    }
}

pub trait DrawModel<'a, 'b>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );

    fn draw_model(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_model_instanced_with_material(
        &mut self,
        model: &'b Model,
        //material: &'b Material,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, /*0..1,*/ uniforms, light);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        //self.draw_indexed(0..mesh.num_elements, 0, instances);
        self.draw_indexed(0..mesh.num_indexes, 0, 0..mesh.num_instances);
        
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_model_instanced(model, /*0..1,*/ uniforms, light);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        let material = model.material.as_ref().unwrap();
        for mesh in &model.meshes {
            //let material = &model.materials[mesh.material];
            
            self.draw_mesh_instanced(mesh, &material/*, instances.clone()*/, uniforms, light);
        }
    }

    fn draw_model_instanced_with_material(
        &mut self,
        model: &'b Model,
        //material: &'b Material,
        //instances: Range<u32>,Copy, Clone
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        let material = model.material.as_ref().unwrap();
        for mesh in &model.meshes {
            self.draw_mesh_instanced(mesh, &material, /*instances.clone(),*/ uniforms, light);
        }
    }
}

pub trait DrawLight<'a, 'b>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) where
        'b: 'a;

    fn draw_light_model(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, uniforms, light);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, uniforms, &[]);
        self.set_bind_group(1, light, &[]);
        self.draw_indexed(0..mesh.num_indexes, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, uniforms, light);
    }
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(mesh, instances.clone(), uniforms, light);
        }
    }
}
