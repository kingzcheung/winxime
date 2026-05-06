use windows::Win32::System::Com::*;
use windows::Win32::Foundation::BOOL;
use windows::core::*;
use crate::TextInputProcessor;

pub const E_NOINTERFACE: HRESULT = HRESULT(0x80004002u32 as i32);

#[implement(IClassFactory)]
pub struct ClassFactory;

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> Result<()> {
        if punkouter.is_some() {
            return Err(Error::from(E_NOINTERFACE));
        }
        
        let processor: IUnknown = TextInputProcessor::new().into();
        let hr = unsafe { Interface::query(&processor, riid, ppvobject) };
        if hr.is_err() {
            Err(Error::from(hr))
        } else {
            Ok(())
        }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}

impl ClassFactory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClassFactory {
    fn default() -> Self {
        Self::new()
    }
}