use windows::Win32::UI::TextServices::*;
use windows::Win32::Foundation::*;
use windows::core::*;
use std::cell::RefCell;

const VK_A: u16 = 0x41;
const VK_Z: u16 = 0x5A;
const VK_0: u16 = 0x30;
const VK_9: u16 = 0x39;
const VK_SPACE: u16 = 0x20;
const VK_RETURN: u16 = 0x0D;
const VK_BACK: u16 = 0x08;
const VK_ESCAPE: u16 = 0x1B;

fn is_letter_key(vk: u16) -> bool {
    (VK_A..=VK_Z).contains(&vk)
}

fn is_digit_key(vk: u16) -> bool {
    (VK_0..=VK_9).contains(&vk)
}

fn get_vk_char(vk: u16) -> char {
    if is_letter_key(vk) {
        ((vk - VK_A) as u8 + b'a') as char
    } else if is_digit_key(vk) {
        ((vk - VK_0) as u8 + b'0') as char
    } else {
        '\0'
    }
}

#[implement(ITfTextInputProcessor)]
pub struct TextInputProcessor {
    thread_mgr: RefCell<Option<ITfThreadMgr>>,
    key_event_sink_cookie: RefCell<u32>,
}

impl TextInputProcessor {
    pub fn new() -> Self {
        Self {
            thread_mgr: RefCell::new(None),
            key_event_sink_cookie: RefCell::new(0),
        }
    }
}

impl Default for TextInputProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ITfTextInputProcessor_Impl for TextInputProcessor_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, _tid: u32) -> Result<()> {
        *self.thread_mgr.borrow_mut() = ptim.cloned();
        
        if let Some(thread_mgr) = &*self.thread_mgr.borrow() {
            let source: ITfSource = thread_mgr.cast()?;
            unsafe {
                let sink = KeyEventSink::new();
                let sink_unknown: IUnknown = sink.into();
                let cookie = source.AdviseSink(&ITfKeyEventSink::IID, &sink_unknown)?;
                *self.key_event_sink_cookie.borrow_mut() = cookie;
            }
        }
        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        let cookie = *self.key_event_sink_cookie.borrow();
        if cookie != 0 {
            if let Some(thread_mgr) = &*self.thread_mgr.borrow() {
                let source: ITfSource = thread_mgr.cast()?;
                unsafe {
                    source.UnadviseSink(cookie)?;
                }
            }
        }
        *self.thread_mgr.borrow_mut() = None;
        Ok(())
    }
}

#[implement(ITfKeyEventSink)]
pub struct KeyEventSink {
    composing: RefCell<bool>,
    input_buffer: RefCell<String>,
}

impl KeyEventSink {
    pub fn new() -> Self {
        Self {
            composing: RefCell::new(false),
            input_buffer: RefCell::new(String::new()),
        }
    }

    fn is_composing(&self) -> bool {
        *self.composing.borrow()
    }

    fn start_composing(&self) {
        *self.composing.borrow_mut() = true;
        self.input_buffer.borrow_mut().clear();
    }

    fn end_composing(&self) {
        *self.composing.borrow_mut() = false;
        self.input_buffer.borrow_mut().clear();
    }

    fn append_char(&self, ch: char) {
        self.input_buffer.borrow_mut().push(ch);
    }

    fn remove_last_char(&self) {
        self.input_buffer.borrow_mut().pop();
    }

    fn get_buffer(&self) -> String {
        self.input_buffer.borrow().clone()
    }

    fn should_handle_key(vk: u16, composing: bool) -> bool {
        if is_letter_key(vk) {
            return true;
        }
        
        if composing {
            if is_digit_key(vk) {
                return true;
            }
            if vk == VK_SPACE || vk == VK_RETURN || vk == VK_BACK || vk == VK_ESCAPE {
                return true;
            }
        }
        
        false
    }
}

impl Default for KeyEventSink {
    fn default() -> Self {
        Self::new()
    }
}

impl ITfKeyEventSink_Impl for KeyEventSink_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(&self, _pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let vk = wparam.0 as u16;
        let composing = self.is_composing();
        let handled = KeyEventSink::should_handle_key(vk, composing);
        Ok(BOOL(if handled { 1 } else { 0 }))
    }

    fn OnKeyDown(&self, _pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let vk = wparam.0 as u16;
        
        if is_letter_key(vk) {
            if !self.is_composing() {
                self.start_composing();
            }
            let ch = get_vk_char(vk);
            self.append_char(ch);
            return Ok(BOOL(1));
        }
        
        if self.is_composing() {
            if is_digit_key(vk) {
                return Ok(BOOL(1));
            }
            
            if vk == VK_SPACE || vk == VK_RETURN {
                let _buffer = self.get_buffer();
                self.end_composing();
                return Ok(BOOL(1));
            }
            
            if vk == VK_ESCAPE {
                self.end_composing();
                return Ok(BOOL(1));
            }
            
            if vk == VK_BACK {
                self.remove_last_char();
                if self.get_buffer().is_empty() {
                    self.end_composing();
                }
                return Ok(BOOL(1));
            }
        }
        
        Ok(BOOL(0))
    }

    fn OnTestKeyUp(&self, _pic: Option<&ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL(0))
    }

    fn OnKeyUp(&self, _pic: Option<&ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL(0))
    }

    fn OnPreservedKey(&self, _pic: Option<&ITfContext>, _rguid: *const GUID) -> Result<BOOL> {
        Ok(BOOL(0))
    }
}