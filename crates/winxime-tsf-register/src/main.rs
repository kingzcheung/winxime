use std::{env, fs, process};
use windows::{
    Win32::{
        Foundation::E_FAIL,
        Globalization::*,
        System::{
            Com::*,
            Console::{ATTACH_PARENT_PROCESS, AttachConsole},
            LibraryLoader::{GetProcAddress, LoadLibraryW},
        },
        UI::{Input::KeyboardAndMouse::HKL, TextServices::*},
    },
    core::*,
};
use winxime_ipc::IpcClient;

windows::core::link!("input.dll" "system" fn InstallLayoutOrTip(psz: *const u16, dwFlags: u32));
const ILOT_INSTALL: u32 = 0x00000000;
const ILOT_UNINSTALL: u32 = 0x00000001;

const XIME_TSF_CLSID: GUID = GUID::from_u128(0x5C1E4D8A_F3B2_4A7E_9CD1_2A3B4C5D6E7F);
const XIME_PROFILE_GUID: GUID = GUID::from_u128(0x5C1E4D8A_F3B2_4A7E_9CD1_2A3B4C5D6E7F);
const XIME_TIP_DESC: PCWSTR = w!("0x0804:{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}");

const CATEGORIES: [GUID; 7] = [
    GUID_TFCAT_TIP_KEYBOARD,
    GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
    GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT,
    GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
    GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    GUID_TFCAT_TIPCAP_COMLESS,
];

fn get_system_dir() -> std::path::PathBuf {
    let system_root = env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    std::path::PathBuf::from(system_root).join("System32")
}

fn copy_dll_to_system(source: &std::path::Path) -> Result<std::path::PathBuf> {
    let system_dir = get_system_dir();
    let dest = system_dir.join("winxime_tsf.dll");
    
    if dest.exists() {
        fs::remove_file(&dest).ok();
    }
    
    fs::copy(source, &dest).map_err(|_| Error::from(E_FAIL))?;
    println!("  Copied {} -> {}", source.display(), dest.display());
    
    Ok(dest)
}

fn remove_dll_from_system() -> Result<()> {
    let system_dir = get_system_dir();
    let dll_path = system_dir.join("winxime_tsf.dll");
    
    if dll_path.exists() {
        fs::remove_file(&dll_path).map_err(|_| Error::from(E_FAIL))?;
        println!("  Removed {}", dll_path.display());
    }
    
    Ok(())
}

fn register_dll(dll_path: &str) -> Result<()> {
    unsafe {
        let dll_path_wide: Vec<u16> = dll_path.encode_utf16().chain(std::iter::once(0)).collect();
        let hmodule = LoadLibraryW(PCWSTR::from_raw(dll_path_wide.as_ptr()))?;
        
        let proc = GetProcAddress(hmodule, s!("DllRegisterServer"));
        if proc.is_none() {
            return Err(Error::from(E_FAIL));
        }
        
        let func: extern "system" fn() -> HRESULT = std::mem::transmute(proc.unwrap());
        let result = func();
        if result.is_err() {
            return Err(Error::from(result));
        }
    }
    Ok(())
}

fn unregister_dll(dll_path: &str) -> Result<()> {
    unsafe {
        let dll_path_wide: Vec<u16> = dll_path.encode_utf16().chain(std::iter::once(0)).collect();
        let hmodule = LoadLibraryW(PCWSTR::from_raw(dll_path_wide.as_ptr()))?;
        
        let proc = GetProcAddress(hmodule, s!("DllUnregisterServer"));
        if proc.is_none() {
            return Err(Error::from(E_FAIL));
        }
        
        let func: extern "system" fn() -> HRESULT = std::mem::transmute(proc.unwrap());
        let result = func();
        if result.is_err() {
            return Err(Error::from(result));
        }
    }
    Ok(())
}

fn register(icon_path: String) -> Result<()> {
    unsafe {
        let input_processor_profile_mgr: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;

        let pw_icon_path = icon_path.encode_utf16().chain(std::iter::once(0)).collect::<Vec<_>>();

        let mut lcid = LocaleNameToLCID(w!("zh-CN"), 0);
        if matches!(lcid, 0 | 0x0C00 | 0x1000) {
            lcid = 0x804;
        }

        input_processor_profile_mgr.RegisterProfile(
            &XIME_TSF_CLSID,
            lcid as u16,
            &XIME_PROFILE_GUID,
            w!("Xime(曦码)五笔").as_wide(),
            &pw_icon_path,
            0,
            HKL::default(),
            0,
            false,
            0,
        )?;

        let category_manager: ITfCategoryMgr =
            CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        for tfcat in &CATEGORIES {
            category_manager.RegisterCategory(&XIME_TSF_CLSID, tfcat, &XIME_TSF_CLSID)?;
        }
    }

    Ok(())
}

fn unregister() -> Result<()> {
    unsafe {
        let input_processor_profile_mgr: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;

        let category_manager: ITfCategoryMgr =
            CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        for tfcat in &CATEGORIES {
            if let Err(error) =
                category_manager.UnregisterCategory(&XIME_TSF_CLSID, tfcat, &XIME_TSF_CLSID)
            {
                println!("Failed to unregister category {tfcat:?}: {error}");
            }
        }

        let mut lcid = LocaleNameToLCID(w!("zh-CN"), 0);
        if matches!(lcid, 0 | 0x0C00 | 0x1000) {
            lcid = 0x804;
        }
        input_processor_profile_mgr.UnregisterProfile(
            &XIME_TSF_CLSID,
            lcid as u16,
            &XIME_PROFILE_GUID,
            0,
        )?;
    }

    Ok(())
}

fn enable() {
    unsafe {
        InstallLayoutOrTip(XIME_TIP_DESC.as_ptr(), ILOT_INSTALL);
    }
}

fn disable() {
    unsafe {
        InstallLayoutOrTip(XIME_TIP_DESC.as_ptr(), ILOT_UNINSTALL);
    }
}

fn stop() {
    if !IpcClient::shutdown_server() {
        println!("Error: failed to stop winxime-server");
    }
}

fn main() -> Result<()> {
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        let exe_path = env::current_exe().expect("无法获取exe路径");
        let exe_dir = exe_path.parent().expect("无法获取exe目录");
        let dll_path = exe_dir.join("winxime_tsf.dll");
        let dll_path_str = dll_path.to_string_lossy().to_string();

        if env::args().len() == 1 {
            println!("Usage:");
            println!("  winxime-tsf-register -install         完整安装(注册DLL+注册Profile+启用)");
            println!("  winxime-tsf-register -uninstall       完整卸载");
            println!("  winxime-tsf-register -r <IconPath>    注册TSF Profile");
            println!("  winxime-tsf-register -dll             注册DLL");
            println!("  winxime-tsf-register -i               启用输入法");
            println!("  winxime-tsf-register -d               停用输入法");
            println!("  winxime-tsf-register -s               停止 winxime-server");
            process::exit(1);
        }

        let arg1 = env::args().nth(1).expect("缺少参数");

        match arg1.as_str() {
            "-install" => {
                println!("Step 1: Copying DLL to system directory...");
                let source_dll = exe_dir.join("winxime_tsf.dll");
                let system_dll = match copy_dll_to_system(&source_dll) {
                    Ok(path) => path,
                    Err(e) => {
                        println!("  Copy FAILED: {:?}", e);
                        println!("  Note: This requires administrator privileges");
                        process::exit(1);
                    }
                };

                println!("Step 2: Registering DLL...");
                match register_dll(&system_dll.to_string_lossy()) {
                    Ok(_) => println!("  DLL registered"),
                    Err(e) => {
                        println!("  DLL registration FAILED: {:?}", e);
                        process::exit(1);
                    }
                }

                let icon_path = exe_dir.join("icon.ico").to_string_lossy().to_string();
                println!("Step 3: Registering TSF Profile...");
                match register(icon_path) {
                    Ok(_) => println!("  Profile registered"),
                    Err(e) => {
                        println!("  Profile registration FAILED: {:?}", e);
                        process::exit(1);
                    }
                }

                println!("Step 4: Enabling input method...");
                enable();
                println!("  Enabled");

                println!("Installation complete!");
            }
            "-uninstall" => {
                println!("Step 1: Stopping server...");
                stop();

                println!("Step 2: Unregistering TSF Profile...");
                match unregister() {
                    Ok(_) => println!("  Profile unregistered"),
                    Err(e) => println!("  Profile unregister failed: {:?}", e),
                }

                let system_dir = get_system_dir();
                let system_dll = system_dir.join("winxime_tsf.dll");
                println!("Step 3: Unregistering DLL...");
                match unregister_dll(&system_dll.to_string_lossy()) {
                    Ok(_) => println!("  DLL unregistered"),
                    Err(e) => println!("  DLL unregister failed: {:?}", e),
                }

                println!("Step 4: Removing DLL from system directory...");
                match remove_dll_from_system() {
                    Ok(_) => println!("  DLL removed"),
                    Err(e) => println!("  Remove failed: {:?}", e),
                }

                println!("Uninstallation complete!");
            }
            "-register-only" => {
                // For WiX: DLL is already copied to System32 by MSI
                // Icon path is passed as argument, or use exe directory
                let system_dir = get_system_dir();
                let system_dll = system_dir.join("winxime_tsf.dll");
                
                println!("Step 1: Registering DLL...");
                match register_dll(&system_dll.to_string_lossy()) {
                    Ok(_) => println!("  DLL registered"),
                    Err(e) => {
                        println!("  DLL registration FAILED: {:?}", e);
                        process::exit(1);
                    }
                }

                // Use passed icon path, or fallback to exe directory
                let icon_path = env::args().nth(2)
                    .map(|p| p.trim_matches('"').to_string())
                    .unwrap_or_else(|| exe_dir.join("icon.ico").to_string_lossy().to_string());
                println!("Step 2: Registering TSF Profile with icon: {}", icon_path);
                match register(icon_path) {
                    Ok(_) => println!("  Profile registered"),
                    Err(e) => {
                        println!("  Profile registration FAILED: {:?}", e);
                        process::exit(1);
                    }
                }

                println!("Step 3: Enabling input method...");
                enable();
                println!("  Enabled");

                println!("Registration complete!");
            }
            "-unregister-only" => {
                // For WiX: only unregister, don't delete DLL (MSI handles that)
                println!("Step 1: Unregistering TSF Profile...");
                match unregister() {
                    Ok(_) => println!("  Profile unregistered"),
                    Err(e) => println!("  Profile unregister failed: {:?}", e),
                }

                let system_dir = get_system_dir();
                let system_dll = system_dir.join("winxime_tsf.dll");
                println!("Step 2: Unregistering DLL...");
                match unregister_dll(&system_dll.to_string_lossy()) {
                    Ok(_) => println!("  DLL unregistered"),
                    Err(e) => println!("  DLL unregister failed: {:?}", e),
                }

                println!("Unregistration complete!");
            }
            "-r" => {
                let icon_path = env::args().nth(2).expect("缺少 IconPath");
                println!("Registering profile with icon: {}", icon_path);
                match register(icon_path) {
                    Ok(_) => println!("Registration successful"),
                    Err(e) => {
                        println!("Registration FAILED: {:?}", e);
                        process::exit(1);
                    }
                }
            }
            "-dll" => {
                println!("Registering DLL: {}", dll_path_str);
                match register_dll(&dll_path_str) {
                    Ok(_) => println!("DLL registered successfully"),
                    Err(e) => {
                        println!("DLL registration FAILED: {:?}", e);
                        process::exit(1);
                    }
                }
            }
            "-i" => {
                println!("Enabling input method...");
                enable();
                println!("Enable command sent");
            }
            "-d" => {
                println!("Disabling input method...");
                disable();
                println!("Disable command sent");
            }
            "-s" => {
                println!("Stopping server...");
                stop();
            }
            "-u" => {
                println!("Unregistering profile...");
                match unregister() {
                    Ok(_) => println!("Unregistration successful"),
                    Err(err) => {
                        println!("Unregister FAILED: {:?}", err);
                        process::exit(1);
                    }
                }
            }
            _ => {
                println!("Unknown parameter: {}", arg1);
                process::exit(1);
            }
        }
    }

    Ok(())
}