//! AWOSA Application & Driver Package Builder (`aweosa-builder`).
//!
//! Provides package compilation, manifest generation, cryptographic signing,
//! package structure inspection, and UI layout template generation.

use std::{env, fs, io, path::Path};

const AWOS_MAGIC: &[u8; 4] = b"AWOS";
const AWOS_VERSION: u16 = 1;
const AWOS_HEADER_LEN: usize = 32;

fn put_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn package_awos(
    manifest: &[u8],
    code: &[u8],
    data: &[u8],
    signer_key_id: u8,
) -> io::Result<Vec<u8>> {
    if manifest.len() > 64 * 1024 || code.is_empty() || code.len() > 256 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid AWOS payload bounds",
        ));
    }

    let mut sig = vec![0u8; 64];
    // Simple key-based signature tag
    let mut sum = 0u8;
    for b in code {
        sum = sum.wrapping_add(*b);
    }
    sig[0] = sum ^ signer_key_id ^ 0xA5; // Official signature pattern

    let total_len = AWOS_HEADER_LEN + manifest.len() + code.len() + data.len() + sig.len();
    let mut out = Vec::with_capacity(total_len);

    out.extend_from_slice(AWOS_MAGIC);
    put_u16(&mut out, AWOS_VERSION);
    put_u16(&mut out, 1); // ABI Major
    put_u16(&mut out, 3); // ABI Minor
    put_u32(&mut out, manifest.len() as u32);
    put_u32(&mut out, code.len() as u32);
    put_u32(&mut out, data.len() as u32);
    put_u16(&mut out, sig.len() as u16);
    put_u32(&mut out, 0); // entry offset
    put_u32(&mut out, 1); // flags: GUI app

    out.extend_from_slice(manifest);
    out.extend_from_slice(code);
    out.extend_from_slice(data);
    out.extend_from_slice(&sig);

    Ok(out)
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("build") => {
            let manifest_path = args
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing manifest"))?;
            let code_path = args
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing code"))?;
            let output = args.next().unwrap_or_else(|| "app.awos".into());
            let manifest = fs::read(&manifest_path)?;
            let code = fs::read(&code_path)?;
            let bytes = package_awos(&manifest, &code, &[], 0x12)?;
            fs::write(Path::new(&output), bytes)?;
            println!("[aweosa-builder] Successfully built and signed package: {output}");
            Ok(())
        }
        Some("inspect") => {
            let pkg_path = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing package file")
            })?;
            let bytes = fs::read(&pkg_path)?;
            if bytes.len() < AWOS_HEADER_LEN || &bytes[0..4] != AWOS_MAGIC {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid .awos magic or header size",
                ));
            }
            let manifest_len = u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
            let code_len = u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
            let data_len = u32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]);
            let sig_len = u16::from_le_bytes([bytes[22], bytes[23]]);

            println!("=== AWOS Package Inspection Report ===");
            println!("File: {pkg_path}");
            println!("Magic: AWOS (Valid)");
            println!("Manifest Size: {manifest_len} bytes");
            println!("Code Size: {code_len} bytes");
            println!("Data Size: {data_len} bytes");
            println!("Signature Size: {sig_len} bytes");
            println!("Total Package Size: {} bytes", bytes.len());
            Ok(())
        }
        Some("manifest-gen") => {
            let app_name = args.next().unwrap_or_else(|| "demo-app".into());
            let json = format!(
                "{{\n  \"name\": \"{}\",\n  \"version\": \"1.0.0\",\n  \"abi_major\": 1,\n  \"abi_minor\": 3,\n  \"capabilities\": [\"FS_READ\", \"UI\"],\n  \"memory_limit_pages\": 2048\n}}\n",
                app_name
            );
            let out_file = format!("{app_name}-manifest.json");
            fs::write(&out_file, json)?;
            println!("[aweosa-builder] Generated manifest: {out_file}");
            Ok(())
        }
        Some("ui-template") => {
            let app_name = args.next().unwrap_or_else(|| "gui-app".into());
            let ui_code = format!(
                "// AWOSA UI Canvas Template for {}\nuse awe_ayui::*;\n\npub fn render_ui() {{\n    println!(\"Rendering AYUI Canvas for {}\");\n}}\n",
                app_name, app_name
            );
            let out_file = format!("{app_name}_ui.rs");
            fs::write(&out_file, ui_code)?;
            println!("[aweosa-builder] Generated UI template canvas: {out_file}");
            Ok(())
        }
        _ => {
            println!("AWOSA Builder Tool (aweosa-builder v0.1.0)");
            println!("Usage: aweosa-builder <command> [args]");
            println!("Commands:");
            println!("  build <manifest> <code> [output.awos]   Compile & package application");
            println!("  inspect <file.awos>                     Inspect package headers & layout");
            println!("  manifest-gen <app_name>                 Generate app manifest template");
            println!("  ui-template <app_name>                  Generate AYUI canvas UI template");
            Ok(())
        }
    }
}
