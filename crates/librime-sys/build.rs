use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

    let possible_dll_locations = [
        dist_lib_dir.join("rime.dll"),
        dist_dir.join("bin").join("rime.dll"),
        librime_dir.join("build").join("Release").join("rime.dll"),
        librime_dir.join("build").join("bin").join("Release").join("rime.dll"),
        librime_dir.join("build").join("lib").join("Release").join("rime.dll"),
    ];

    let found_dll = possible_dll_locations.iter().find(|p| p.exists());

    if let Some(found_path) = found_dll {
        if found_path != &rime_dll {
            println!("cargo:warning=Found rime.dll at {}, copying to expected location", found_path.display());
            std::fs::copy(found_path, &rime_dll).ok();
        }
        
        println!("cargo:rustc-link-search=native={}", dist_lib_dir.display());
        println!("cargo:rustc-link-lib=dylib=rime");
        
        let profiles = ["debug", "release"];
        for profile in &profiles {
            let target_dir = workspace_dir.join("target").join(profile);
            let target_rime_dll = target_dir.join("rime.dll");
            if target_dir.exists() {
                std::fs::copy(&rime_dll, &target_rime_dll).ok();
                println!("cargo:warning=Copied rime.dll to target/{}", profile);
            }
        }
    } else {
        println!("cargo:warning=Searched for rime.dll in:");
        for loc in &possible_dll_locations {
            println!("cargo:warning=  {} (exists: {})", loc.display(), loc.exists());
        }
        let build_dir = librime_dir.join("build");
        if build_dir.exists() {
            println!("cargo:warning=Build directory exists, listing contents:");
            if let Ok(entries) = std::fs::read_dir(&build_dir) {
                for entry in entries.flatten() {
                    println!("cargo:warning=  {}", entry.path().display());
                }
            }
        } else {
            println!("cargo:warning=Build directory does not exist: {}", build_dir.display());
        }
        panic!(
            "librime build failed: rime.dll not found at {} or any alternative location",
            rime_dll.display()
        );
    }

    copy_rime_data(&workspace_dir, &librime_dir);

    println!("cargo:rerun-if-changed=build.rs");
}

fn copy_rime_data(_workspace_dir: &Path, _librime_dir: &Path) {
    // opencc is not needed - rime works without it
}

fn build_librime(librime_dir: &PathBuf, workspace_dir: &Path) {
    println!(
        "cargo:warning=Building librime from source (this may take 10+ minutes on first build)..."
    );

    let vswhere =
        PathBuf::from("C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe");
    if !vswhere.exists() {
        panic!("Visual Studio Installer not found at {}", vswhere.display());
    }

    let vs_install: String = match Command::new(&vswhere)
        .args(["-latest", "-property", "installationPath"])
        .output()
    {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(e) => panic!("vswhere failed: {}", e),
    };

    if vs_install.is_empty() {
        panic!("Visual Studio not found via vswhere");
    }

    println!("cargo:warning=Using Visual Studio at: {}", vs_install);

    let env_bat = librime_dir.join("env.bat");
    {
        let mut file = std::fs::File::create(&env_bat).unwrap();
        writeln!(file, "@echo off").unwrap();
        writeln!(file, "set RIME_ROOT={}", librime_dir.display()).unwrap();
        writeln!(
            file,
            "if not defined BOOST_ROOT set BOOST_ROOT={}\\deps\\boost-1.89.0",
            librime_dir.display()
        )
        .unwrap();
        writeln!(file, "set ARCH=x64").unwrap();
        writeln!(file, "set BJAM_TOOLSET=msvc-14.3").unwrap();
        writeln!(file, "set CMAKE_GENERATOR=\"Visual Studio 17 2022\"").unwrap();
        writeln!(file, "set PLATFORM_TOOLSET=v143").unwrap();
    }

    let temp_bat = workspace_dir.join("temp-build-librime.bat");
    {
        let mut file = std::fs::File::create(&temp_bat).unwrap();
        writeln!(file, "@echo off").unwrap();
        writeln!(
            file,
            "call \"{}\\VC\\Auxiliary\\Build\\vcvars64.bat\"",
            vs_install
        )
        .unwrap();
        writeln!(file, "cd /d \"{}\"", librime_dir.display()).unwrap();
        writeln!(
            file,
            "if not exist \"{0}\\deps\\boost-1.89.0\\boost\" call install-boost.bat",
            librime_dir.display()
        )
        .unwrap();
        writeln!(
            file,
            "if not defined BOOST_ROOT set BOOST_ROOT={}\\deps\\boost-1.89.0",
            librime_dir.display()
        )
        .unwrap();
        writeln!(file, "echo ===== STARTING DEPS BUILD =====").unwrap();
        writeln!(file, "build.bat deps").unwrap();
        writeln!(file, "if errorlevel 1 echo DEPS BUILD FAILED with error %errorlevel% && exit /b %errorlevel%").unwrap();
        writeln!(file, "echo ===== DEPS BUILD COMPLETE =====").unwrap();
        writeln!(file, "echo ===== STARTING LIBRIME BUILD =====").unwrap();
        writeln!(file, "echo build_dir=%build_dir%").unwrap();
        writeln!(file, "echo build_config=%build_config%").unwrap();
        writeln!(file, "build.bat librime shared").unwrap();
        writeln!(file, "if errorlevel 1 echo LIBRIME BUILD FAILED with error %errorlevel% && exit /b %errorlevel%").unwrap();
        writeln!(file, "echo ===== LIBRIME BUILD COMPLETE =====").unwrap();
        writeln!(file, "echo Checking if rime.dll was created...").unwrap();
        writeln!(file, "if exist \"{}\\dist\\lib\\rime.dll\" echo rime.dll found in dist/lib", librime_dir.display()).unwrap();
        writeln!(file, "if exist \"{}\\build\\Release\\rime.dll\" echo rime.dll found in build/Release", librime_dir.display()).unwrap();
        writeln!(file, "dir \"{}\\dist\\lib\\*.dll\" 2>nul || echo No DLLs in dist/lib", librime_dir.display()).unwrap();
    }

    println!("cargo:warning=Starting librime compilation...");

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let progress_thread = thread::spawn(move || {
        let mut count = 0;
        while running_clone.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(30));
            if running_clone.load(Ordering::Relaxed) {
                count += 1;
                println!(
                    "cargo:warning=librime build in progress... ({} minutes elapsed)",
                    count / 2
                );
            }
        }
    });

    let status = Command::new(&temp_bat)
        .current_dir(workspace_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    running.store(false, Ordering::Relaxed);
    progress_thread.join().ok();

    std::fs::remove_file(&temp_bat).ok();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=librime compilation finished successfully");
        }
        Ok(s) => {
            panic!("librime build failed with exit code: {:?}", s.code());
        }
        Err(e) => {
            panic!("librime build failed to execute: {}", e);
        }
    }
}
