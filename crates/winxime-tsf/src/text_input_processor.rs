use windows::Win32::UI::TextServices::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::core::*;
use std::sync::Arc;

const VK_X_A: u16 = 0x41;
const VK_X_Z: u16 = 0x5A;
const VK_X_0: u16 = 0x30;
const VK_X_9: u16 = 0x39;

// ── Shared Rime engine ──────────────────────────────────────────

pub struct RimeEngineHandle {
    engine: Arc<std::sync::Mutex<winxime_rime::RimeEngine>>,
}

impl RimeEngineHandle {
    pub fn new(engine: winxime_rime::RimeEngine) -> Self {
        Self {
            engine: Arc::new(std::sync::Mutex::new(engine)),
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, winxime_rime::RimeEngine> {
        self.engine.lock().unwrap()
    }
}

impl Clone for RimeEngineHandle {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
        }
    }
}

// ── Captured Rime output for edit session ───────────────────────

#[allow(dead_code)]
struct RimeOutput {
    commit: Option<String>,
    composing: bool,
    preedit: Option<String>,
    candidate_count: usize,
    page_no: usize,
    page_size: usize,
}

// ── Main edit session: commit + composition in one callback ─────

#[implement(ITfEditSession)]
struct XimeEditSession {
    output: RimeOutput,
    thread_mgr: Option<ITfThreadMgr>,
    composition: Arc<std::sync::Mutex<Option<ITfComposition>>>,
}

impl ITfEditSession_Impl for XimeEditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let doc_mgr = match self.thread_mgr.as_ref() {
            Some(t) => unsafe { t.GetFocus() }?,
            None => return Ok(()),
        };
        let context = unsafe { doc_mgr.GetBase() }?;

        // 1. Commit text if present
        if let Some(ref commit) = self.output.commit {
            if !commit.is_empty() {
                // End existing composition first
                self.end_composition(ec);

                let text_store: ITextStoreACP = context.cast()?;
                let mut fetched: u32 = 0;
                let mut selection = vec![TS_SELECTION_ACP::default(); 1];
                unsafe { text_store.GetSelection(0, &mut selection, &mut fetched)?; }

                if fetched > 0 {
                    let sel = selection[0];
                    let wide: Vec<u16> = commit.encode_utf16().collect();
                    unsafe {
                        text_store.SetText(0, sel.acpStart, sel.acpEnd, &wide)?;
                    }
                    let end = sel.acpStart + wide.len() as i32;
                    unsafe {
                        text_store.SetSelection(&[TS_SELECTION_ACP {
                            acpStart: end,
                            acpEnd: end,
                            style: TS_SELECTIONSTYLE {
                                ase: TsActiveSelEnd::default(),
                                fInterimChar: BOOL(0),
                            },
                        }])?;
                    }
                }
                return Ok(());
            }
        }

        // 2. Check for composition state change
        if self.output.composing {
            let preedit = self.output.preedit.as_deref().unwrap_or("");
            let comp = self.composition.lock().unwrap().take();

            if comp.is_some() {
                // Update existing composition text
                self.update_composition_text(&context, ec, preedit);
            } else {
                // Start new composition
                self.start_composition(&context, ec, preedit);
            }
        } else {
            // Not composing: end any existing composition
            self.end_composition(ec);
        }

        Ok(())
    }
}

impl XimeEditSession_Impl {
    fn start_composition(&self, context: &ITfContext, ec: u32, preedit: &str) {
        let text_store: ITextStoreACP = match context.cast() {
            Ok(s) => s,
            Err(_) => return,
        };

        // Get selection position
        let mut fetched: u32 = 0;
        let mut selection = vec![TS_SELECTION_ACP::default(); 1];
        if unsafe { text_store.GetSelection(0, &mut selection, &mut fetched) }.is_err() || fetched == 0 {
            return;
        }
        let sel = selection[0];

        // Get ITfContextComposition from context
        let ctx_comp: ITfContextComposition = match context.cast() {
            Ok(c) => c,
            Err(_) => return,
        };

        // Create a range for the composition
        let range_acp: ITfRangeACP = match text_store.cast() {
            Ok(r) => r,
            Err(_) => return,
        };

        // Get ITfRange from ITfRangeACP
        let range: ITfRange = match range_acp.cast() {
            Ok(r) => r,
            Err(_) => return,
        };

        // Start composition at cursor position
        let none_sink: Option<&ITfCompositionSink> = None;
        match unsafe { ctx_comp.StartComposition(ec, &range, none_sink) } {
            Ok(comp) => {
                // Set preedit text on the composition range
                let wide: Vec<u16> = preedit.encode_utf16().collect();
                unsafe { text_store.SetText(0, sel.acpStart, sel.acpStart, &wide).ok(); }
                *self.composition.lock().unwrap() = Some(comp);
            }
            Err(_) => {}
        }
    }

    fn update_composition_text(&self, _context: &ITfContext, _ec: u32, preedit: &str) {
        // For now, end and restart composition to update text
        // A more efficient approach would use ITfCompositionView::GetRange + ITfRange::SetText
        let text_store: ITextStoreACP = match _context.cast() {
            Ok(s) => s,
            Err(_) => return,
        };

        let mut fetched: u32 = 0;
        let mut selection = vec![TS_SELECTION_ACP::default(); 1];
        if unsafe { text_store.GetSelection(0, &mut selection, &mut fetched) }.is_err() || fetched == 0 {
            return;
        }
        let sel = selection[0];

        let wide: Vec<u16> = preedit.encode_utf16().collect();
        unsafe { text_store.SetText(0, sel.acpStart, sel.acpStart, &wide).ok(); }
    }

    fn end_composition(&self, ec: u32) {
        if let Some(comp) = self.composition.lock().unwrap().take() {
            unsafe { comp.EndComposition(ec).ok(); }
        }
    }
}

// ── Key event sink ──────────────────────────────────────────────

#[implement(ITfKeyEventSink)]
pub struct KeyEventSink {
    rime: std::sync::Mutex<Option<RimeEngineHandle>>,
    composing: std::sync::atomic::AtomicBool,
    thread_mgr: std::sync::Mutex<Option<ITfThreadMgr>>,
    client_id: std::sync::atomic::AtomicU32,
    composition: Arc<std::sync::Mutex<Option<ITfComposition>>>,
}

impl KeyEventSink {
    pub fn new() -> Self {
        Self {
            rime: std::sync::Mutex::new(None),
            composing: std::sync::atomic::AtomicBool::new(false),
            thread_mgr: std::sync::Mutex::new(None),
            client_id: std::sync::atomic::AtomicU32::new(0),
            composition: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn set_rime(&self, rime: RimeEngineHandle) {
        *self.rime.lock().unwrap() = Some(rime);
    }

    pub fn set_thread_mgr(&self, mgr: Option<ITfThreadMgr>) {
        *self.thread_mgr.lock().unwrap() = mgr;
    }

    pub fn set_client_id(&self, id: u32) {
        self.client_id.store(id, std::sync::atomic::Ordering::Release);
    }

    fn is_composing(&self) -> bool {
        self.composing.load(std::sync::atomic::Ordering::Acquire)
    }

    fn set_composing(&self, val: bool) {
        self.composing.store(val, std::sync::atomic::Ordering::Release);
    }

    fn should_handle_key(&self, vk: VIRTUAL_KEY) -> bool {
        let code = vk.0;
        if (VK_X_A..=VK_X_Z).contains(&code) {
            return true;
        }
        if code == VK_SPACE.0 || code == VK_RETURN.0 || code == VK_BACK.0 || code == VK_ESCAPE.0 {
            return true;
        }
        if self.is_composing() {
            if (VK_X_0..=VK_X_9).contains(&code) {
                return true;
            }
            if code == VK_UP.0 || code == VK_DOWN.0 || code == VK_PRIOR.0 || code == VK_NEXT.0 {
                return true;
            }
        }
        false
    }

    fn capture_output(&self) -> Option<RimeOutput> {
        let guard = self.rime.lock().ok()?;
        let handle = guard.as_ref()?;
        let engine = handle.lock();

        let commit = engine.get_commit();
        let composing = engine.is_composing();
        let preedit = engine.get_composition().and_then(|c| c.preedit);

        let candidates = engine.get_candidates();
        let candidate_count = candidates.len();
        // Get page info from first candidate if available
        let page_no = engine.get_composition().map(|c| c.sel_start).unwrap_or(0);
        let page_size = candidates.len();

        Some(RimeOutput {
            commit,
            composing,
            preedit,
            candidate_count,
            page_no: page_no / 5,
            page_size,
        })
    }

    fn schedule_edit_session(&self, context: &ITfContext, output: RimeOutput) {
        let tid = self.client_id.load(std::sync::atomic::Ordering::Acquire);
        let mgr = self.thread_mgr.lock().unwrap().clone();

        let session = XimeEditSession {
            output,
            thread_mgr: mgr,
            composition: self.composition.clone(),
        };
        let session_itf: ITfEditSession = session.into();
        unsafe {
            let _ = context.RequestEditSession(tid, &session_itf, TF_ES_READWRITE);
        }
    }

    fn handle_key_event(&self, context: Option<&ITfContext>, vk: VIRTUAL_KEY) {
        let guard = match self.rime.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let handle = match guard.as_ref() {
            Some(h) => h,
            None => return,
        };
        let code = vk.0;
        let is_composing = self.is_composing();

        if is_composing && code >= 0x31 && code <= 0x39 {
            let index = (code - 0x31) as usize;
            let mut engine = handle.lock();
            engine.select_candidate(index);
            drop(engine);
            if let Some(output) = self.capture_output() {
                self.set_composing(output.composing);
                if let Some(ctx) = context {
                    self.schedule_edit_session(ctx, output);
                }
            }
            return;
        }

        if is_composing && code == VK_PRIOR.0 {
            let mut engine = handle.lock();
            engine.change_page(true);
            drop(engine);
            if let Some(output) = self.capture_output() {
                self.set_composing(output.composing);
                if let Some(ctx) = context {
                    self.schedule_edit_session(ctx, output);
                }
            }
            return;
        }
        if is_composing && code == VK_NEXT.0 {
            let mut engine = handle.lock();
            engine.change_page(false);
            drop(engine);
            if let Some(output) = self.capture_output() {
                self.set_composing(output.composing);
                if let Some(ctx) = context {
                    self.schedule_edit_session(ctx, output);
                }
            }
            return;
        }

        let xk = librime_sys::vk_to_xk(code);
        let mods = librime_sys::get_key_modifiers();
        let mut engine = handle.lock();
        let handled = engine.process_key(xk, mods);
        drop(engine);
        drop(guard);

        if handled {
            if let Some(output) = self.capture_output() {
                self.set_composing(output.composing);
                if let Some(ctx) = context {
                    self.schedule_edit_session(ctx, output);
                }
            }
        }
    }
}

impl ITfKeyEventSink_Impl for KeyEventSink_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(&self, _pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let vk = VIRTUAL_KEY(wparam.0 as u16);
        Ok(BOOL(if self.should_handle_key(vk) { 1 } else { 0 }))
    }

    fn OnKeyDown(&self, pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let vk = VIRTUAL_KEY(wparam.0 as u16);
        let handled = self.should_handle_key(vk);
        if handled {
            self.handle_key_event(pic, vk);
        }
        Ok(BOOL(if handled { 1 } else { 0 }))
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

// ── Text input processor ────────────────────────────────────────

#[implement(ITfTextInputProcessor)]
pub struct XimeTextService {
    thread_mgr: std::cell::RefCell<Option<ITfThreadMgr>>,
    client_id: std::cell::Cell<u32>,
    cookie: std::cell::Cell<u32>,
    rime: std::sync::Mutex<Option<RimeEngineHandle>>,
    key_sink: std::cell::RefCell<Option<ITfKeyEventSink>>,
    shared_data: String,
    user_data: String,
}

impl XimeTextService {
    pub fn new(shared_data: String, user_data: String) -> Self {
        Self {
            thread_mgr: std::cell::RefCell::new(None),
            client_id: std::cell::Cell::new(0),
            cookie: std::cell::Cell::new(0),
            rime: std::sync::Mutex::new(None),
            key_sink: std::cell::RefCell::new(None),
            shared_data,
            user_data,
        }
    }

    fn ensure_rime(&self) {
        let mut guard = self.rime.lock().unwrap();
        if guard.is_some() {
            return;
        }
        let shared = std::path::Path::new(&self.shared_data);
        let user = std::path::Path::new(&self.user_data);
        match winxime_rime::RimeEngine::new(shared, user, "Xime") {
            Ok(engine) => {
                *guard = Some(RimeEngineHandle::new(engine));
            }
            Err(e) => {
                eprintln!("Xime: failed to init Rime: {}", e);
            }
        }
    }
}

impl ITfTextInputProcessor_Impl for XimeTextService_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        *self.thread_mgr.borrow_mut() = ptim.cloned();
        self.client_id.set(tid);

        self.ensure_rime();

        let sink = KeyEventSink::new();
        sink.set_client_id(tid);
        sink.set_thread_mgr(ptim.cloned());
        if let Some(handle) = self.rime.lock().unwrap().as_ref() {
            sink.set_rime(handle.clone());
        }

        let key_sink_itf: ITfKeyEventSink = sink.into();

        if let Some(thread_mgr) = ptim {
            if let Ok(source) = thread_mgr.cast::<ITfSource>() {
                let cookie = unsafe {
                    source.AdviseSink(&ITfKeyEventSink::IID, &key_sink_itf)?
                };
                self.cookie.set(cookie);
                *self.key_sink.borrow_mut() = Some(key_sink_itf);
            }
        }

        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        let cookie = self.cookie.get();
        if cookie != 0 {
            if let Some(thread_mgr) = self.thread_mgr.borrow().as_ref() {
                if let Ok(source) = thread_mgr.cast::<ITfSource>() {
                    unsafe { let _ = source.UnadviseSink(cookie); }
                }
            }
        }

        self.cookie.set(0);
        *self.key_sink.borrow_mut() = None;
        *self.thread_mgr.borrow_mut() = None;
        Ok(())
    }
}
