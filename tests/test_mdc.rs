mod test_data;

#[cfg(test)]
mod tests {
    use isomesh::mdc::{MeshBuffers, mdc_mesh_generation};

    use crate::test_data::{EXPECTED_NORMALS, EXPECTED_POSITIONS};

    #[test]
    fn test_mdc_sphere() {
        let resolution = 64;
        let mut mesh_buffers = MeshBuffers::new();
        mdc_mesh_generation(0.5, &mut mesh_buffers, true, resolution, true);
        assert_eq!(mesh_buffers.positions.len(), EXPECTED_POSITIONS.len());
        for (i, expected_pos) in EXPECTED_POSITIONS.iter().enumerate() {
            let actual_pos = [
                mesh_buffers.positions[i][0],
                mesh_buffers.positions[i][1],
                mesh_buffers.positions[i][2],
            ];
            assert_eq!(actual_pos, *expected_pos);
        }
        for (i, expected_normal) in EXPECTED_NORMALS.iter().enumerate() {
            let actual_normal = [
                mesh_buffers.normals[i][0],
                mesh_buffers.normals[i][1],
                mesh_buffers.normals[i][2],
            ];
            assert_eq!(actual_normal, *expected_normal);
        }
    }
}
