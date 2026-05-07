use crate::class_factory::CLSID_XIME;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetProcAddress, LoadLibraryW};
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::TextServices::*;
use windows_core::*;

const TSF_LANGID: u16 = 0x0804;

fn clsid_str() -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        CLSID_XIME.data1,
        CLSID_XIME.data2,
        CLSID_XIME.data3,
        CLSID_XIME.data4[0],
        CLSID_XIME.data4[1],
        CLSID_XIME.data4[2],
        CLSID_XIME.data4[3],
        CLSID_XIME.data4[4],
        CLSID_XIME.data4[5],
        CLSID_XIME.data4[6],
        CLSID_XIME.data4[7],
    )
}

fn get_module_path() -> String {
    unsafe {
        let mut buf = [0u16; 260];
        let len = GetModuleFileNameW(Some(HMODULE(DLL_MODULE.0)), &mut buf);
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }
    if *rclsid != CLSID_XIME {
        return HRESULT(-2147221231);
    }
    let factory = crate::ClassFactory;
    let unknown: IUnknown = factory.into();
    Interface::query(&unknown, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    match do_register() {
        Ok(_) => S_OK,
        Err(e) => {
            eprintln!("DllRegisterServer failed: {}", e);
            HRESULT(-1i32)
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    do_unregister();
    S_OK
}

static mut DLL_MODULE: HINSTANCE = HINSTANCE(std::ptr::null_mut());

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst: HINSTANCE,
    _reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    DLL_MODULE = hinst;
    BOOL(1)
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn install_layout() {
    let install_str = format!("0804:{}{{{}}}", clsid_str(), clsid_str(),);
    let wide: Vec<u16> = install_str
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        if let Ok(module) = LoadLibraryW(w!("input.dll")) {
            if let Some(func) = std::mem::transmute::<
                _,
                Option<unsafe extern "system" fn(*const u16, u32) -> BOOL>,
            >(GetProcAddress(module, s!("InstallLayoutOrTip")))
            {
                let _ = func(wide.as_ptr(), 0);
            }
        }
    }
}

fn uninstall_layout() {
    let install_str = format!("0804:{}{{{}}}", clsid_str(), clsid_str(),);
    let wide: Vec<u16> = install_str
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        if let Ok(module) = LoadLibraryW(w!("input.dll")) {
            if let Some(func) = std::mem::transmute::<
                _,
                Option<unsafe extern "system" fn(*const u16, u32) -> BOOL>,
            >(GetProcAddress(module, s!("InstallLayoutOrTip")))
            {
                let _ = func(wide.as_ptr(), 1);
            }
        }
    }
}

fn do_register() -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let cs = clsid_str();
    let module_path = get_module_path();
    let name = "Xime 五笔输入法";

    eprintln!("[Xime] DllRegisterServer: CLSID={}", cs);
    eprintln!("[Xime] DllRegisterServer: module_path={}", module_path);

    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    eprintln!("[Xime] Step 1: Creating CLSID registry key...");
    let (clsid_key, _) = hkcr.create_subkey(&format!("CLSID\\{}", cs)).map_err(|e| {
        eprintln!("[Xime] ERROR: Failed to create CLSID key: {}", e);
        e
    })?;

    eprintln!("[Xime] Step 2: Setting default value...");
    clsid_key.set_value("", &name).map_err(|e| {
        eprintln!("[Xime] ERROR: Failed to set default value: {}", e);
        e
    })?;

    eprintln!("[Xime] Step 3: Creating InprocServer32...");
    let (inproc, _) = clsid_key.create_subkey("InprocServer32").map_err(|e| {
        eprintln!("[Xime] ERROR: Failed to create InprocServer32: {}", e);
        e
    })?;
    inproc.set_value("", &module_path).map_err(|e| {
        eprintln!("[Xime] ERROR: Failed to set module path: {}", e);
        e
    })?;
    inproc
        .set_value("ThreadingModel", &"Apartment")
        .map_err(|e| {
            eprintln!("[Xime] ERROR: Failed to set ThreadingModel: {}", e);
            e
        })?;

    let _ = hkcr.create_subkey(&format!(
        "CLSID\\{}\\Implemented Categories\\{{34745C63-B2F0-4784-8B67-5E12C8701A31}}",
        cs
    ));

    eprintln!("[Xime] Step 4: CoInitializeEx...");
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        eprintln!("[Xime] Step 5: CoCreateInstance ITfInputProcessorProfileMgr...");
        let mgr: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| {
                    eprintln!(
                        "[Xime] ERROR: Failed to create ITfInputProcessorProfileMgr: {:?}",
                        e
                    );
                    e
                })?;

        eprintln!("[Xime] Step 6: RegisterProfile...");
        mgr.RegisterProfile(
            &CLSID_XIME as *const GUID,
            TSF_LANGID,
            &CLSID_XIME as *const GUID,
            &wide_string(name),
            &wide_string(&module_path),
            0,
            HKL::default(),
            0,
            true,
            0,
        )
        .map_err(|e| {
            eprintln!("  -> ERROR: RegisterProfile: {:?}", e);
            e
        })?;

        eprintln!("[Xime] Step 7: EnableLanguageProfile...");
        let profiles: ITfInputProcessorProfiles = mgr.cast().map_err(|e| {
            eprintln!(
                "[Xime] ERROR: Failed to cast to ITfInputProcessorProfiles: {:?}",
                e
            );
            e
        })?;
        profiles
            .EnableLanguageProfile(
                &CLSID_XIME as *const GUID,
                TSF_LANGID,
                &CLSID_XIME as *const GUID,
                true,
            )
            .map_err(|e| {
                eprintln!("  -> ERROR: EnableLanguageProfile: {:?}", e);
                e
            })?;

        eprintln!("[Xime] Step 8: Registering categories...");
        let tsf_categories = [
            GUID_TFCAT_TIP_KEYBOARD,
            GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT,
            GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
            GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
            GUID_TFCAT_TIPCAP_COMLESS,
            GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
            GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
        ];

        if let Ok(cat_mgr) =
            CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)
        {
            for catid in &tsf_categories {
                let _ = cat_mgr.RegisterCategory(
                    &CLSID_XIME as *const GUID,
                    catid as *const GUID,
                    &CLSID_XIME as *const GUID,
                );
            }
        }

        eprintln!("[Xime] Step 9: install_layout...");
        install_layout();
    }

    eprintln!("[Xime] DllRegisterServer completed successfully!");
    Ok(())
}

fn do_unregister() {
    use winreg::enums::*;
    use winreg::RegKey;

    let cs = clsid_str();

    uninstall_layout();

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        if let Ok(mgr) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            let _ = mgr.UnregisterProfile(
                &CLSID_XIME as *const GUID,
                TSF_LANGID,
                &CLSID_XIME as *const GUID,
                0,
            );
        }

        let tsf_categories = [
            GUID_TFCAT_TIP_KEYBOARD,
            GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT,
            GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
            GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
            GUID_TFCAT_TIPCAP_COMLESS,
            GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
            GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
        ];
        if let Ok(cat_mgr) =
            CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)
        {
            for catid in &tsf_categories {
                let _ = cat_mgr.UnregisterCategory(
                    &CLSID_XIME as *const GUID,
                    catid as *const GUID,
                    &CLSID_XIME as *const GUID,
                );
            }
        }
    }

    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let _ = hkcr.delete_subkey_all(&format!("CLSID\\{}", cs));

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let _ = hklm.delete_subkey_all(&format!("Software\\Microsoft\\CTF\\TIP\\{}", cs));

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let _ = hkcu.delete_subkey_all(&format!("Software\\Microsoft\\CTF\\TIP\\{}", cs));
}
