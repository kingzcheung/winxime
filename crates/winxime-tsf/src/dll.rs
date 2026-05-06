use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows::core::*;
use crate::class_factory::CLSID_XIME;

const TSF_LANGID: u16 = 0x0804;

fn clsid_str() -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        CLSID_XIME.data1, CLSID_XIME.data2, CLSID_XIME.data3,
        CLSID_XIME.data4[0], CLSID_XIME.data4[1],
        CLSID_XIME.data4[2], CLSID_XIME.data4[3],
        CLSID_XIME.data4[4], CLSID_XIME.data4[5],
        CLSID_XIME.data4[6], CLSID_XIME.data4[7],
    )
}

fn get_module_path() -> String {
    unsafe {
        let handle = GetModuleHandleW(None).unwrap_or_default();
        let mut buf = [0u16; 260];
        let len = GetModuleFileNameW(handle, &mut buf);
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
        return HRESULT(-2147221231); // CLASS_E_CLASSNOTAVAILABLE
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

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _hinst: HINSTANCE,
    _reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    BOOL(1)
}

fn do_register() -> std::io::Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let cs = clsid_str();
    let module_path = get_module_path();
    let name = "Xime 五笔输入法";

    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    let (clsid_key, _) = hkcr.create_subkey(&format!("CLSID\\{}", cs))?;
    clsid_key.set_value("", &name)?;

    let (inproc, _) = clsid_key.create_subkey("InprocServer32")?;
    inproc.set_value("", &module_path)?;
    inproc.set_value("ThreadingModel", &"Apartment")?;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let tip_path = format!("Software\\Microsoft\\CTF\\TIP\\{}", cs);
    let (tip_key, _) = hklm.create_subkey(&tip_path)?;
    tip_key.set_value("", &name)?;

    let lang_path = format!("Language Profile\\0x{:04X}", TSF_LANGID);
    let (lang_base, _) = tip_key.create_subkey(&lang_path)?;
    let (profile, _) = lang_base.create_subkey(&cs)?;
    profile.set_value("", &name)?;
    profile.set_value("IconFile", &module_path)?;

    let _ = tip_key.create_subkey("DisplayAttribute");

    let cat_path = format!("CLSID\\{}\\Implemented Categories", cs);
    let (cat_key, _) = hkcr.create_subkey(&cat_path)?;
    let _ = cat_key.create_subkey("{34745C63-B2F0-4784-8B67-5E12C8701A31}");
    let _ = cat_key.create_subkey("{34745C63-B2F0-4784-8B67-5E12C8701A32}");

    let categories = [
        ("{34745C63-B2F0-4784-8B67-5E12C8701A31}", "Xime Text Input Processor"),
        ("{34745C63-B2F0-4784-8B67-5E12C8701A32}", "Xime Keyboard"),
    ];
    for (catid, desc) in &categories {
        let (ck, _) = hkcr.create_subkey(&format!("Component Categories\\{}", catid))?;
        let _ = ck.set_value("", desc);
    }

    Ok(())
}

fn do_unregister() {
    use winreg::enums::*;
    use winreg::RegKey;

    let cs = clsid_str();

    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let _ = hkcr.delete_subkey_all(&format!("CLSID\\{}", cs));

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let tip_path = format!("Software\\Microsoft\\CTF\\TIP\\{}", cs);
    let _ = hklm.delete_subkey_all(&tip_path);
}
