//! AWEOSA App Builder bootstrap.
//! Dependency-free host tool: converts raw code/data/manifest sections into an
//! AWOS v1 container. The kernel remains no_std; this tool is a build-time
//! utility and intentionally uses only the Rust standard library.

use std::{env, fs, process};

const MAGIC: &[u8; 4] = b"AWOS";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 36;

fn put_u16(out: &mut Vec<u8>, v: u16) { out.extend_from_slice(&v.to_le_bytes()); }
fn put_u32(out: &mut Vec<u8>, v: u32) { out.extend_from_slice(&v.to_le_bytes()); }

fn read(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| { eprintln!("AWEOSA: cannot read {path}: {e}"); process::exit(2); })
}

fn usage() -> ! {
    eprintln!("usage: awe-builder --manifest M --code C [--data D] --out APP.awos");
    process::exit(2);
}

fn main() {
    let mut manifest = None;
    let mut code = None;
    let mut data = Vec::new();
    let mut out = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next().unwrap_or_else(|| usage());
        match arg.as_str() {
            "--manifest" => manifest = Some(read(&value)),
            "--code" => code = Some(read(&value)),
            "--data" => data = read(&value),
            "--out" => out = Some(value),
            _ => usage(),
        }
    }
    let manifest = manifest.unwrap_or_else(|| usage());
    let code = code.unwrap_or_else(|| usage());
    let out = out.unwrap_or_else(|| usage());
    if code.is_empty() || manifest.len() > 8192 || code.len() > u32::MAX as usize || data.len() > u32::MAX as usize {
        eprintln!("AWEOSA: invalid section sizes");
        process::exit(2);
    }

    let mut image = Vec::with_capacity(HEADER_LEN + manifest.len() + code.len() + data.len());
    image.extend_from_slice(MAGIC);
    put_u16(&mut image, VERSION);
    put_u16(&mut image, 0); // flags
    put_u32(&mut image, 1); // ABI v1
    put_u32(&mut image, manifest.len() as u32);
    put_u32(&mut image, code.len() as u32);
    put_u32(&mut image, data.len() as u32);
    put_u32(&mut image, 0); // detached signature is added by signing pipeline
    put_u32(&mut image, 0); // entry offset: code entry
    put_u32(&mut image, 0); // reserved
    debug_assert_eq!(image.len(), HEADER_LEN);
    image.extend_from_slice(&manifest);
    image.extend_from_slice(&code);
    image.extend_from_slice(&data);
    fs::write(&out, image).unwrap_or_else(|e| { eprintln!("AWEOSA: cannot write {out}: {e}"); process::exit(3); });
}
