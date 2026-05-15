use windows::Win32::System::Com::*;
use windows_core::*;

pub const CLSID_XIME: GUID = GUID {
    data1: 0x5C1E4D8A,
    data2: 0xF3B2,
    data3: 0x4A7E,
    data4: [0x9C, 0xD1, 0x2A, 0x3B, 0x4C, 0x5D, 0x6E, 0x7F],
};

#[implement(IClassFactory)]
pub struct ClassFactory;

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> Result<()> {
        if punkouter.as_ref().is_some() {
            return Err(Error::from(HRESULT(-2147221232)));
        }

        crate::dll::increment_instance_count();
        
        let service = crate::XimeTextService::new();
        let unknown: IUnknown = service.into();
        unsafe {
            let hr = Interface::query(&unknown, riid, ppvobject);
            if hr.is_ok() {
                Ok(())
            } else {
                crate::dll::decrement_instance_count();
                Err(Error::from(hr))
            }
        }
    }

    fn LockServer(&self, flock: BOOL) -> Result<()> {
        if flock.as_bool() {
            crate::dll::increment_instance_count();
        } else {
            crate::dll::decrement_instance_count();
        }
        Ok(())
    }
}
