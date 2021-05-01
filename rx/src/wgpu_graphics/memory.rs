pub struct MemoryManager {
    pub (crate)mesh_buffer: wgpu::Buffer,
    pub (crate)idx_buffer: wgpu::Buffer,
    pub (crate)instanced_buffer: wgpu::Buffer,
}

pub struct MemoryManagerConfig {
    pub mesh_buffer_size: u64,
    pub idx_buffer_size: u64,
    pub instanced_buffer_size: u64,
}

impl MemoryManager {
    fn mesh_buffer_desc( config: &MemoryManagerConfig) -> wgpu::BufferDescriptor {
        wgpu::BufferDescriptor {
            label: Some("mesh_buffer"),
            size: config.mesh_buffer_size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        }
    }
    fn index_buffer_desc(config: &MemoryManagerConfig) -> wgpu::BufferDescriptor {
        wgpu::BufferDescriptor {
            label: Some("idx_buffer"),
            size: config.idx_buffer_size,
            usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::MAP_WRITE| wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        }
    }
    fn instanced_buffer_desc(config: &MemoryManagerConfig) -> wgpu::BufferDescriptor {
        wgpu::BufferDescriptor {
            label: Some("instanced_buffer"),
            size: config.instanced_buffer_size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        }
    }


    pub fn new(device: &mut wgpu::Device, config: MemoryManagerConfig) -> Self {
        let mesh_buffer = device.create_buffer(&Self::mesh_buffer_desc(&config));
        let idx_buffer = device.create_buffer(&Self::index_buffer_desc(&config));
        let instanced_buffer = device.create_buffer(&Self::instanced_buffer_desc(&config));
        MemoryManager {
            mesh_buffer,
            idx_buffer,
            instanced_buffer,
        }
    }
}