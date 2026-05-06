use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::*;
use crate::class_factory::ClassFactory;

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    _rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let factory: IUnknown = ClassFactory::new().into();
    Interface::query(&factory, riid, ppv).into()
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    HRESULT(0)
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    HRESULT(0)
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    HRESULT(0)
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _hinst: HINSTANCE,
    reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    if reason == 1 {
        let _ = GetModuleHandleW(None);
    }
    BOOL(1)
}