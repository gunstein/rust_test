use anyhow::*;
use std::ops::Range;
use std::path::Path;
use wgpu::util::DeviceExt;

use crate::texture;

use rand::Rng;

use std::collections::HashMap;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex {
    position: cgmath::Vector3<f32>,
    tex_coords: cgmath::Vector2<f32>,
    normal: cgmath::Vector3<f32>,
    tangent: cgmath::Vector3<f32>,
    bitangent: cgmath::Vector3<f32>,
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
    pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        normal_texture: texture::Texture,
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}


pub struct Mesh {
    pub name: String,
    pub blocktype: BlockType,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
    pub instances: Vec<Instance>,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,

    pub world : World,

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

#[derive(PartialEq, Eq)]
pub enum BlockType {
    GRASS,
    DIRT,
    STONE,
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

impl Model {
    pub fn build_random_chunk()->Chunk
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


    fn create_vertices(blocktype:BlockType) -> Vec<Vertex>{//(Vec<Vertex>, Vec<u16>) {
        fn build_vertex(position:[i8;3], quadtype:QuadType, u:UV, v:UV)->Vertex
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
                            let gvtestpos:[f32;3]=[position[0] as f32, position[1] as f32, position[2] as f32];
                            //Vertex{position:position, tex_coords:[Clone::clone(u_pos.unwrap()), Clone::clone(v_pos.unwrap())]}
                            Vertex{position:gvtestpos, tex_coords:[Clone::clone(u_pos.unwrap()), 1.0-Clone::clone(v_pos.unwrap())]}
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
        
        let mut vertex_data: Vec<Vertex>= Vec::new();
        
    
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
    
    
        /*
        let index_data: &[u16] = &[
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];
    
        (vertex_data, index_data.to_vec())
        */
        vertex_data
    }
    
    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        path: P,
    ) -> Result<Self> {

        let VERTICES_GRASS = create_vertices(BlockType::GRASS);
        let VERTICES_STONE = create_vertices(BlockType::STONE);
        let VERTICES_DIRT = create_vertices(BlockType::DIRT);

        /*
        let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), true)?;

        // We're assuming that the texture files are stored with the obj file
        let containing_folder = path.as_ref().parent().context("Directory has no parent")?;

        let mut materials = Vec::new();
        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;
            let diffuse_texture =
                texture::Texture::load(device, queue, containing_folder.join(diffuse_path), false)?;

            let normal_path = mat.normal_texture;
            let normal_texture =
                texture::Texture::load(device, queue, containing_folder.join(normal_path), true)?;

            materials.push(Material::new(
                device,
                &mat.name,
                diffuse_texture,
                normal_texture,
                layout,
            ));
        }
        */
        //load material
        let mut materials = Vec::new();
        let diffuse_bytes = include_bytes!("blockatlas.jpg");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "blockatlas.jpg").unwrap();
        materials.push(Material::new(
            device,
            &mat.name,
            diffuse_texture,
            normal_texture,
            layout,
        ));

        //build world
        let mut world = World{chunks:HashMap::new()};
        //First chunk,
        //trenger flere sef
        world.chunks.insert( [0, 0, 0], build_random_chunk);

        //Go through world and build meshes. One mesh for each blocktype
        let mut mesh_grass = Mesh{blocktype : BlockType::GRASS, };
        let mut mesh_dirt = Mesh{blocktype : BlockType::DIRT};
        let mut mesh_stone = Mesh{blocktype : BlockType::STONE};

        //Siden vi bruker instancing er vertexene allerede bygget
        //mesh_grass.vertex_buffer=VERTICES_GRASS Se hvordan dette er gjort i instances-programmet (f.eks)
        mesh_grass.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES_GRASS),
            usage: wgpu::BufferUsage::VERTEX,
        });

        mesh_dirt.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES_DIRT),
            usage: wgpu::BufferUsage::VERTEX,
        });
        
        mesh_dirt.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES_STONE),
            usage: wgpu::BufferUsage::VERTEX,
        });

        //mesh*.index_buffer=CUBE_INDICES
        mesh_grass.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });
        
        mesh_dirt.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        mesh_stone.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        //mesh*.instances må genereres basert på world
        for (keychunk, chunk) in &all_chunks {
            for (blockkey, block) in &curren_chunk.blocks {
                //println!("{}: \"{}\"", book, review);
                //Masse kode her
                //transler til rett plass. Må ta hensyn til flere chunks.
            }
        }

        /*
        let mut meshes = Vec::new();
        for m in obj_models {
            let mut vertices = Vec::new();
            for i in 0..m.mesh.positions.len() / 3 {
                vertices.push(ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ]
                    .into(),
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]].into(),
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ]
                    .into(),
                    // We'll calculate these later
                    tangent: [0.0; 3].into(),
                    bitangent: [0.0; 3].into(),
                });
            }

            let indices = &m.mesh.indices;

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0 = v0.position;
                let pos1 = v1.position;
                let pos2 = v2.position;

                let uv0 = v0.tex_coords;
                let uv1 = v1.tex_coords;
                let uv2 = v2.tex_coords;

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Luckily, the place I found this equation provided
                // the solution!
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent = tangent;
                vertices[c[1] as usize].tangent = tangent;
                vertices[c[2] as usize].tangent = tangent;

                vertices[c[0] as usize].bitangent = bitangent;
                vertices[c[1] as usize].bitangent = bitangent;
                vertices[c[2] as usize].bitangent = bitangent;
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", path.as_ref())),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", path.as_ref())),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsage::INDEX,
            });

            meshes.push(Mesh {
                name: m.name,
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            });
            
        }*/

        Ok(Self { meshes, materials })
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
        instances: Range<u32>,
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
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_model_instanced_with_material(
        &mut self,
        model: &'b Model,
        material: &'b Material,
        instances: Range<u32>,
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
        self.draw_mesh_instanced(mesh, material, 0..1, uniforms, light);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_model_instanced(model, 0..1, uniforms, light);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), uniforms, light);
        }
    }

    fn draw_model_instanced_with_material(
        &mut self,
        model: &'b Model,
        material: &'b Material,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_mesh_instanced(mesh, material, instances.clone(), uniforms, light);
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
        self.draw_indexed(0..mesh.num_elements, 0, instances);
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
