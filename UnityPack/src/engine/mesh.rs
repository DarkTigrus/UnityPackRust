/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use super::EngineObject;
use object::ToByteVec;
use error::{Error, Result};
use object::ObjectValue;
use extras::containers::OrderedMap;
use engine::object::Object;

pub trait IntoMesh {
    fn to_mesh(self) -> Result<Mesh>;
}

#[derive(Debug)]
pub struct Mesh {
    pub object: Object,
    pub root_bone_name_hash: u32,
    pub index_buffer: Vec<u8>,
    pub bind_pose: Vec<ObjectValue>,
    pub baked_convex_collision_mesh: Vec<u8>,
    pub mesh_compression: u8,
    pub submeshes: Vec<SubMesh>,
    pub vertex_data: VertexData,
}

#[derive(Debug)]
pub struct SubMesh {
    pub first_byte: u32,
    pub first_vertex: u32,
    pub index_count: u32,
    pub local_aabb: OrderedMap<String, ObjectValue>,
    pub topology: i32,
    pub vertex_count: u32,
}

impl SubMesh {
    fn from_map(map: &mut OrderedMap<String, ObjectValue>) -> Result<Self> {
        Ok(Self {
            topology: tryGet!(map, "topology").to_i32()?,
            index_count: tryGet!(map, "indexCount").to_u32()?,
            first_vertex: tryGet!(map, "firstVertex").to_u32()?,
            vertex_count: tryGet!(map, "vertexCount").to_u32()?,
            first_byte: tryGet!(map, "firstByte").to_u32()?,
            local_aabb: tryConsume!(map, "localAABB").into_map()?,
        })
    }
}

#[derive(Debug)]
pub struct VertexData {
    pub object: Object,
    pub channels: Vec<OrderedMap<String, ObjectValue>>,
    pub current_channels: i32,
    pub data: Vec<u8>,
    pub vertex_count: u32,
}

impl VertexData {
    fn from_map(map: &mut OrderedMap<String, ObjectValue>) -> Result<Self> {
        Ok(Self {
            object: Object::new(map)?,
            current_channels: tryGet!(map, "m_CurrentChannels").to_i32()?,
            vertex_count: tryGet!(map, "m_VertexCount").to_u32()?,
            data: tryGet!(map, "m_DataSize").to_byte_vec()?,
            channels: {
                let array = tryConsume!(map, "m_Channels").into_vec()?;
                let mut res = Vec::with_capacity(array.len());
                for obj in array {
                    let channelsmap = obj.into_map()?;
                    res.push(channelsmap);
                }
                res
            },
        })
    }
}

impl IntoMesh for EngineObject {
    fn to_mesh(mut self) -> Result<Mesh> {
        Ok(Mesh {
            object: Object::new(&self.map)?,
            root_bone_name_hash: tryGet!(self.map, "m_RootBoneNameHash").to_u32()?,
            index_buffer: tryGet!(self.map, "m_IndexBuffer").to_byte_vec()?,
            bind_pose: tryConsume!(self.map, "m_BindPose").into_vec()?,
            baked_convex_collision_mesh: tryGet!(self.map, "m_BakedConvexCollisionMesh")
                .to_byte_vec()?,
            mesh_compression: tryGet!(self.map, "m_MeshCompression").to_u8()?,
            submeshes: {
                let array = tryConsume!(self.map, "m_SubMeshes").into_vec()?;
                let mut res = Vec::with_capacity(array.len());
                for obj in array {
                    let mut submeshmap = obj.into_map()?;
                    res.push(SubMesh::from_map(&mut submeshmap)?);
                }
                res
            },
            vertex_data: {
                let mut map = tryConsume!(self.map, "m_VertexData").into_map()?;
                VertexData::from_map(&mut map)?
            },
        })
    }
}
