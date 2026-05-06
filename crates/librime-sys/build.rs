use std::env;
use std::path::{PathBuf, Path};
use std::process::Command;
use std::io::Write;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        return;
    }
    
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    
    let librime_dir = workspace_dir.join("librime");
    let dist_dir = librime_dir.join("dist");
    let dist_lib_dir = dist_dir.join("lib");
    let rime_dll = dist_lib_dir.join("rime.dll");
    
    if !rime_dll.exists() {
        build_librime(&librime_dir, workspace_dir);
    }
    
    if rime_dll.exists() {
        println!("cargo:rustc-link-search=native={}", dist_lib_dir.display());
        println!("cargo:rustc-link-lib=dylib=rime");
    } else {
        panic!("librime build failed");
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}

fn build_librime(librime_dir: &PathBuf, workspace_dir: &Path) {
    println!("cargo:warning=Building librime from source...");
    
    let vswhere = PathBuf::from("C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe");
    if !vswhere.exists() {
        panic!("Visual Studio not found");
    }
    
    let vs_install: String = match Command::new(&vswhere)
        .args(["-latest", "-property", "installationPath"])
        .output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => panic!("vswhere failed"),
    };
    
    println!("cargo:warning=VS: {}", vs_install);
    
    // Copy env.bat.template to env.bat with x64/VS2022 settings
    let _env_template = librime_dir.join("env.bat.template");
    let env_bat = librime_dir.join("env.bat");
    // Always overwrite env.bat to ensure correct settings
    {
        let mut file = std::fs::File::create(&env_bat).unwrap();
        writeln!(file, "@echo off").unwrap();
        writeln!(file, "set RIME_ROOT={}", librime_dir.display()).unwrap();
        writeln!(file, "if not defined BOOST_ROOT set BOOST_ROOT={}\\deps\\boost-1.89.0", librime_dir.display()).unwrap();
        writeln!(file, "set ARCH=x64").unwrap();
        writeln!(file, "set BJAM_TOOLSET=msvc-14.3").unwrap();
        writeln!(file, "set CMAKE_GENERATOR=\"Visual Studio 17 2022\"").unwrap();
        writeln!(file, "set PLATFORM_TOOLSET=v143").unwrap();
    }
    
    // Create temp build script
    let temp_bat = workspace_dir.join("temp-build-librime.bat");
    let mut file = std::fs::File::create(&temp_bat).unwrap();
    
    writeln!(file, "@echo off").unwrap();
    writeln!(file, "call \"{}\\VC\\Auxiliary\\Build\\vcvars64.bat\"", vs_install).unwrap();
    writeln!(file, "cd /d \"{}\"", librime_dir.display()).unwrap();
    writeln!(file, "if not exist \"{0}\\deps\\boost-1.89.0\\boost\" call install-boost.bat", librime_dir.display()).unwrap();
    // install-boost.bat uses setlocal so BOOST_ROOT doesn't persist; set it explicitly
    writeln!(file, "if not defined BOOST_ROOT set BOOST_ROOT={}\\deps\\boost-1.89.0", librime_dir.display()).unwrap();
    writeln!(file, "build.bat deps").unwrap();
    writeln!(file, "build.bat librime").unwrap();
    
    file.flush().unwrap();
    
    println!("cargo:warning=Running build script...");
    
    let status = Command::new(&temp_bat)
        .current_dir(workspace_dir)
        .status()
        .expect("build failed");
    
    std::fs::remove_file(&temp_bat).ok();
    
    if !status.success() {
        panic!("librime build failed");
    }
    
    println!("cargo:warning=librime build complete");
}