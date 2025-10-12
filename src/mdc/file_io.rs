use crate::mdc::mdc::MeshBuffers;
use std::fs::File;
use std::io::Write;

#[allow(dead_code)]
pub fn write_to_rust(mesh_buffers: &MeshBuffers, path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    writeln!(file, "#[allow(dead_code)]")?;
    writeln!(file, "pub const EXPECTED_POSITIONS: &[[f32; 3]] = &[")?;
    for pos in &mesh_buffers.positions {
        writeln!(file, "    [{:?}, {:?}, {:?}],", pos[0], pos[1], pos[2])?;
    }
    writeln!(file, "];\n")?;

    writeln!(file, "#[allow(dead_code)]")?;
    writeln!(file, "pub const EXPECTED_NORMALS: &[[f32; 3]] = &[")?;
    for normal in &mesh_buffers.normals {
        writeln!(
            file,
            "    [{:?}, {:?}, {:?}],",
            normal[0], normal[1], normal[2]
        )?;
    }
    writeln!(file, "];\n")?;

    writeln!(file, "#[allow(dead_code)]")?;
    writeln!(file, "pub const EXPECTED_COLORS: &[[f32; 4]] = &[")?;
    for color in &mesh_buffers.colors {
        writeln!(
            file,
            "    [{:?}, {:?}, {:?}, {:?}],",
            color[0], color[1], color[2], color[3]
        )?;
    }
    writeln!(file, "];\n")?;

    writeln!(file, "#[allow(dead_code)]")?;
    writeln!(file, "pub const EXPECTED_INDICES: &[u32] = &[")?;
    for (i, idx) in mesh_buffers.indices.iter().enumerate() {
        if i % 12 == 0 {
            write!(file, "    ")?;
        }
        write!(file, "{}", idx)?;
        if i < mesh_buffers.indices.len() - 1 {
            write!(file, ", ")?;
        }
        if (i + 1) % 12 == 0 {
            writeln!(file)?;
        }
    }
    if !mesh_buffers.indices.is_empty() && mesh_buffers.indices.len() % 12 != 0 {
        writeln!(file)?;
    }
    writeln!(file, "];")?;

    Ok(())
}
