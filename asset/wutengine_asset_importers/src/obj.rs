//! Wavefront obj importer

use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;

use wutengine_assets::SerializedAsset;
use wutengine_assets::assets::material::SerializedMaterial;
use wutengine_assets::assets::mesh::MeshIndices;
use wutengine_assets::assets::mesh::MeshTopology;
use wutengine_assets::assets::mesh::SerializedMesh;
use wutengine_assets::nohash_hasher::IntMap;
use wutengine_assets::nohash_hasher::IntSet;
use wutengine_math::Color;
use wutengine_math::Vec2;
use wutengine_math::Vec3;

use crate::AssetImporter;
use crate::ImportedAsset;

/// Importer for OBJ 3D files, and its MTL material extension
#[derive(Debug)]
pub struct ObjAssetImporter;

/// Failure during obj import
#[derive(Debug, derive_more::Error, derive_more::Display)]
pub enum ObjImportErr {
    /// Error while loading .obj file
    #[display("Failed to load obj file due to error in obj lib: {}", _0)]
    LoadObj(tobj::LoadError),

    /// Error while loading .mtl file
    #[display("Failed to load a material due to error in obj lib: {}", _0)]
    LoadMat(tobj::LoadError),

    /// Geometry in mesh was not supported
    #[display("The geometry of an imported mesh was not supported: {}", _0)]
    UnsupportedGeometry(#[error(not(source))] String),
}

/// An imported obj mesh
#[derive(Debug, Clone)]
pub struct ImportedMesh {
    /// The name of the mesh in the file
    pub name: String,

    /// The mesh
    pub mesh: SerializedMesh,
}

/// An imported obj material
#[derive(Debug, Clone)]
pub struct ImportedMaterial {
    /// The name of the material in the file
    pub name: String,

    /// The material
    pub material: SerializedMaterial,
}

impl ObjAssetImporter {
    /// Import an obj file (and an optional material lib)
    pub fn import_obj_mtl_bytes(
        obj_bytes: &[u8],
        mtllib_files: &HashMap<PathBuf, &[u8]>,
    ) -> Result<(Vec<ImportedMesh>, Vec<ImportedMaterial>), ObjImportErr> {
        profiling::function_scope!();

        log::info!(
            "Importing .obj from bytes. {} material libs given",
            mtllib_files.len()
        );

        let mut reader = BufReader::new(obj_bytes);
        let loaded = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: false,
                ignore_points: true,
                ignore_lines: true,
            },
            |path| {
                let Some(mtl_bytes) = mtllib_files.get(path) else {
                    return Err(tobj::LoadError::OpenFileFailed);
                };

                let mut reader = BufReader::new(*mtl_bytes);

                tobj::load_mtl_buf(&mut reader)
            },
        );

        let (models, materials) = match loaded {
            Ok((models, Ok(mats))) => (models, mats),
            Ok((models, Err(tobj::LoadError::OpenFileFailed))) => (models, Vec::new()),
            Ok((_, Err(other_mtl_err))) => {
                return Err(ObjImportErr::LoadMat(other_mtl_err));
            }
            Err(err) => {
                return Err(ObjImportErr::LoadObj(err));
            }
        };

        let verts = models.iter().map(|m| m.mesh.positions.len()).sum::<usize>();

        log::info!(
            "Found {} models and {} materials, total {verts} vertices",
            models.len(),
            materials.len()
        );

        let mut meshes = Vec::with_capacity(models.len());

        for model in models {
            meshes.push(Self::import_model(model)?);
        }

        Ok((meshes, Vec::new()))
    }

    fn import_model(model: tobj::Model) -> Result<ImportedMesh, ObjImportErr> {
        profiling::function_scope!();

        let model_mesh = model.mesh;

        let topology = Self::get_topology(&model_mesh)?;
        let vertices = Self::get_vertices(&model_mesh.positions);
        let indices = Self::get_indices(model_mesh.indices);
        let normals = Self::get_normals(&model_mesh.normals);
        let uvs = Self::get_uvs(&model_mesh.texcoords);
        let colors = Self::get_colors(&model_mesh.vertex_color);

        Ok(ImportedMesh {
            name: model.name,
            mesh: SerializedMesh {
                topology,
                vertices,
                indices,
                normals,
                uvs,
                colors,
                keep_data: false,
            },
        })
    }

    fn get_topology(model_mesh: &tobj::Mesh) -> Result<MeshTopology, ObjImportErr> {
        profiling::function_scope!();

        if model_mesh.face_arities.is_empty() {
            return Ok(MeshTopology::Triangle);
        }

        let mut arities: IntSet<u32> = IntSet::default();

        arities.extend(&model_mesh.face_arities);

        if arities.len() != 1 {
            return Err(ObjImportErr::UnsupportedGeometry(
                "Not all faces have the same number of points".to_string(),
            ));
        }

        let arity = arities.into_iter().next().expect("Should have one arity");

        match arity {
            1 => Ok(MeshTopology::Point),
            2 => Ok(MeshTopology::Line),
            3 => Ok(MeshTopology::Triangle),
            other => Err(ObjImportErr::UnsupportedGeometry(format!(
                "Unsupported number of points per face: {other}"
            ))),
        }
    }

    fn get_vertices(verts: &[f32]) -> Vec<Vec3> {
        profiling::function_scope!();

        let (vtx_chunks, tail) = verts.as_chunks::<3>();

        assert!(tail.is_empty(), "Should return proper geometry");

        vtx_chunks
            .iter()
            .map(|vtx| Vec3::new(vtx[0], vtx[1], vtx[2]))
            .collect()
    }

    fn get_indices(indices: Vec<u32>) -> MeshIndices {
        profiling::function_scope!();

        let u16_indices = indices
            .iter()
            .copied()
            .all(|idx| u16::try_from(idx).is_ok());

        if u16_indices {
            MeshIndices::U16(indices.into_iter().map(|idx| idx as u16).collect())
        } else {
            MeshIndices::U32(indices)
        }
    }

    fn get_normals(normals: &[f32]) -> Vec<Vec3> {
        profiling::function_scope!();

        if normals.is_empty() {
            return Vec::new();
        }

        let (normal_chunks, tail) = normals.as_chunks::<3>();

        assert!(tail.is_empty(), "Should return proper normal geometry");

        normal_chunks
            .iter()
            .map(|normal| Vec3::new(normal[0], normal[1], normal[2]))
            .collect()
    }

    fn get_uvs(uvs: &[f32]) -> IntMap<u8, Vec<Vec2>> {
        profiling::function_scope!();

        let mut uv_out = IntMap::default();

        if uvs.is_empty() {
            return uv_out;
        }

        let (texcoord_chunks, tail) = uvs.as_chunks::<2>();

        assert!(tail.is_empty(), "Should return proper texcoord geometry");

        uv_out.insert(
            0u8,
            texcoord_chunks
                .iter()
                .map(|uv| Vec2::new(uv[0], uv[1]))
                .collect::<Vec<_>>(),
        );

        uv_out
    }

    fn get_colors(colors: &[f32]) -> Vec<Color> {
        profiling::function_scope!();

        if colors.is_empty() {
            return Vec::new();
        }
        let (color_chunks, tail) = colors.as_chunks::<3>();

        assert!(tail.is_empty(), "Should return proper color geometry");

        color_chunks
            .iter()
            .map(|color| Color::new(color[0], color[1], color[2], 1.0))
            .collect()
    }
}

impl AssetImporter for ObjAssetImporter {
    fn supported_file_types() -> Vec<&'static str> {
        vec!["obj", "mtl"]
    }

    fn from_bytes(
        bytes: &[u8],
        file_type: &str,
        path: Option<&std::path::Path>,
    ) -> Result<Vec<crate::ImportedAsset>, Box<dyn std::error::Error>> {
        profiling::function_scope!();

        _ = path;

        if file_type == "mtl" {
            // Not yet supported
            return Err(Box::new(ObjImportErr::LoadMat(
                tobj::LoadError::OpenFileFailed,
            )));
        }

        let (meshes, materials) = match Self::import_obj_mtl_bytes(bytes, &HashMap::new()) {
            Ok(mm) => mm,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        let mut output = Vec::with_capacity(meshes.len() + materials.len());

        for mesh in meshes {
            output.push(ImportedAsset {
                asset_type_id: SerializedMesh::ID,
                name: Some(mesh.name),
                asset: Box::new(mesh.mesh),
            });
        }

        for material in materials {
            output.push(ImportedAsset {
                asset_type_id: SerializedMaterial::ID,
                name: Some(material.name),
                asset: Box::new(material.material),
            });
        }

        Ok(output)
    }
}
