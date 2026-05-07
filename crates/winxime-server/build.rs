use std::env;
use std::path::PathBuf;
use std::fs;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    
    let profile = env::var("PROFILE").unwrap();
    let target_dir = workspace_dir.join("target").join(&profile);
    
    let rime_dll_src = workspace_dir
        .join("librime")
        .join("build")
        .join("bin")
        .join("Release")
        .join("rime.dll");
    
    let rime_dll_dst = target_dir.join("rime.dll");
    
    if rime_dll_src.exists() {
        if !rime_dll_dst.exists() || 
           fs::metadata(&rime_dll_src).unwrap().modified().unwrap() > 
           fs::metadata(&rime_dll_dst).unwrap().modified().unwrap() {
            fs::copy(&rime_dll_src, &rime_dll_dst).expect("Failed to copy rime.dll");
            println!("cargo:warning=Copied rime.dll to {}", target_dir.display());
        }
    }
    
    println!("cargo:rerun-if-changed={}", rime_dll_src.display());
}