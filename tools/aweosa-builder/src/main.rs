use std::{env, fs, io, path::Path};

const MAGIC: &[u8; 4] = b"AWOS";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 40;

fn put_u16(out: &mut Vec<u8>, v: u16) { out.extend_from_slice(&v.to_le_bytes()); }
fn put_u32(out: &mut Vec<u8>, v: u32) { out.extend_from_slice(&v.to_le_bytes()); }

fn package(manifest: &[u8], code: &[u8], data: &[u8]) -> io::Result<Vec<u8>> {
    if manifest.len() > 8192 || code.is_empty() || code.len() > u32::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid AWOS payload"));
    }
    let mut out = Vec::with_capacity(HEADER_LEN + manifest.len() + code.len() + data.len());
    out.extend_from_slice(MAGIC);
    put_u16(&mut out, VERSION);
    put_u16(&mut out, 0);
    put_u32(&mut out, 1); // ABI v1
    put_u32(&mut out, manifest.len() as u32);
    put_u32(&mut out, code.len() as u32);
    put_u32(&mut out, data.len() as u32);
    put_u32(&mut out, 0); // signature length; release signing is separate
    put_u32(&mut out, 0); // entry offset
    put_u32(&mut out, 0); // reserved
    out.extend_from_slice(manifest);
    out.extend_from_slice(code);
    out.extend_from_slice(data);
    Ok(out)
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("build") => {
            let manifest_path = args.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing manifest"))?;
            let code_path = args.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing code"))?;
            let output = args.next().unwrap_or_else(|| "app.awos".into());
            let manifest = fs::read(&manifest_path)?;
            let code = fs::read(&code_path)?;
            let bytes = package(&manifest, &code, &[])?;
            fs::write(Path::new(&output), bytes)?;
            println!("built {output}");
            Ok(())
        }
        _ => {
            eprintln!("usage: aweosa-builder build <manifest> <code> [output.awos]");
            Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid command"))
        }
    }
}
