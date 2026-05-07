use anyhow::Result;
use std::process::{Command, Stdio};

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!("Commands:");
        eprintln!("  build       - Build all components");
        eprintln!("  run         - Start the server (foreground, Ctrl+C to stop)");
        eprintln!("  run-dev     - Build + start server");
        eprintln!("  install     - Register TSF DLL (elevated)");
        eprintln!("  uninstall   - Unregister TSF DLL (elevated)");
        return Ok(());
    }

    let cmd = &args[1];

    match cmd.as_str() {
        "build" => build(),
        "run" => run_server(),
        "run-dev" => run_dev(),
        "install" => install(),
        "uninstall" => uninstall(),
        _ => {
            eprintln!("Unknown command: {}", cmd);
            Ok(())
        }
    }
}

fn build() -> Result<()> {
    println!("Building all components...");
    let status = Command::new("cargo").args(["build"]).status()?;

    if !status.success() {
        anyhow::bail!("Build failed");
    }
    println!("Build completed successfully");

    let dll_path = std::path::Path::new("target/debug/winxime_tsf.dll");
    if dll_path.exists() {
        println!("  TSF DLL: target\\debug\\winxime_tsf.dll");
        println!("  Server:  target\\debug\\winxime-server.exe");
    }

    Ok(())
}

fn server_exe_path() -> std::path::PathBuf {
    std::path::Path::new("target/debug/winxime-server.exe").to_path_buf()
}

fn run_server() -> Result<()> {
    let exe = server_exe_path();
    if !exe.exists() {
        anyhow::bail!("Server not built. Run 'cargo xtask build' first.");
    }
    let status = Command::new(&exe).status()?;

    if !status.success() {
        anyhow::bail!("Server exited with error");
    }
    Ok(())
}

fn kill_server() {
    let output = Command::new("taskkill")
        .args(["/F", "/IM", "winxime-server.exe"])
        .output();

    match output {
        Ok(out) => {
            let msg = String::from_utf8_lossy(&out.stdout);
            if msg.contains("SUCCESS") {
                println!("Stopped existing server.");
            }
        }
        Err(_) => {}
    }
}

fn run_dev() -> Result<()> {
    println!("========================================");
    println!("Winxime Development Workflow");
    println!("========================================");
    println!("");

    kill_server();

    println!("[1/3] Building...");
    let status = Command::new("cargo").args(["build"]).status()?;
    if !status.success() {
        anyhow::bail!("Build failed");
    }
    println!("");

    let exe = server_exe_path();
    if !exe.exists() {
        anyhow::bail!("winxime-server.exe not found after build");
    }

    println!("[2/3] Starting server...");
    let mut child = Command::new(&exe)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    match child.try_wait()? {
        Some(status) if !status.success() => {
            anyhow::bail!("Server failed to start");
        }
        _ => {}
    }
    println!("");

    println!("[3/3] To test the input method:");
    println!("  Register TSF DLL (elevated PowerShell):");
    println!("    regsvr32 target\\debug\\winxime_tsf.dll");
    println!("  Then switch to Xime input method in any app");
    println!("");
    println!("Press Ctrl+C to stop the server.");
    println!("========================================");

    child.wait()?;
    Ok(())
}

fn install() -> Result<()> {
    let dll_path = std::path::Path::new("target/debug/winxime_tsf.dll");
    if !dll_path.exists() {
        anyhow::bail!("winxime_tsf.dll not found. Run 'cargo xtask build' first.");
    }

    let abs_path = std::fs::canonicalize(dll_path)?;
    let abs_path_str = abs_path.to_string_lossy().to_string();

    println!("Registering {} ...", abs_path_str);

    let status = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Start-Process -Verb RunAs -Wait -FilePath 'regsvr32.exe' -ArgumentList '{}'",
                abs_path_str
            ),
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Registration failed or was cancelled");
    }

    println!("TSF DLL registered successfully");
    Ok(())
}

fn uninstall() -> Result<()> {
    let dll_path = std::path::Path::new("target/debug/winxime_tsf.dll");
    if !dll_path.exists() {
        anyhow::bail!("winxime_tsf.dll not found");
    }

    let abs_path = std::fs::canonicalize(dll_path)?;
    let abs_path_str = abs_path.to_string_lossy().to_string();

    println!("Unregistering {} ...", abs_path_str);

    let status = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Start-Process -Verb RunAs -Wait -FilePath 'regsvr32.exe' -ArgumentList '/u {}'",
                abs_path_str
            ),
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Unregistration failed or was cancelled");
    }

    println!("TSF DLL unregistered successfully");
    Ok(())
}
