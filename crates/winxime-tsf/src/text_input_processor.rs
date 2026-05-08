use crate::log::{init_log, log};
use std::sync::Arc;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::TextServices::*;
use windows_core::{*, Interface};
use winxime_ipc::{
    IpcClient, IpcCommand, IpcRequest, IpcRequestData, IpcResponse, KeyEventData, Position,
};

const VK_X_A: u16 = 0x41;
const VK_X_Z: u16 = 0x5A;
const VK_X_0: u16 = 0x30;
const VK_X_9: u16 = 0x39;

const GUID_LBI_INPUTMODE: GUID = GUID::from_u128(0x5D5D8287_5B53_4DAA_B44C_52EB4794A3E7);

struct IpcState {
    client: Option<IpcClient>,
    session_id: u32,
}

pub struct IpcClientHandle {
    state: Arc<std::sync::Mutex<IpcState>>,
}

impl IpcClientHandle {
    pub fn debug_ptr(&self) -> *const () {
        Arc::as_ptr(&self.state) as *const ()
    }

    pub fn new() -> std::result::Result<Self, winxime_ipc::IpcError> {
        let client = IpcClient::connect()?;
        Ok(Self {
            state: Arc::new(std::sync::Mutex::new(IpcState {
                client: Some(client),
                session_id: 0,
            })),
        })
    }

    pub fn empty() -> Self {
        Self {
            state: Arc::new(std::sync::Mutex::new(IpcState {
                client: None,
                session_id: 0,
            })),
        }
    }

    pub fn connect(&self) -> std::result::Result<(), winxime_ipc::IpcError> {
        log("IpcClientHandle::connect() called");
        let mut guard = self.state.lock().unwrap();
        if guard.client.is_some() {
            log("  -> already connected");
            return Ok(());
        }
        log("  -> calling IpcClient::connect()");
        let client = IpcClient::connect()?;
        log("  -> IpcClient::connect() succeeded");
        guard.client = Some(client);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        let r = self.state.lock().unwrap().client.is_some();
        r
    }

    pub fn start_session(&self) -> (u32, Option<IpcResponse>) {
        let mut guard = self.state.lock().unwrap();
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::StartSession,
                session_id: 0,
                data: IpcRequestData::None,
            };
            if let Ok(response) = client.send_request(&request) {
                guard.session_id = response.session_id;
                return (guard.session_id, Some(response));
            }
        }
        (guard.session_id, None)
    }

    pub fn process_key(&self, keycode: i32, modifiers: i32) -> Option<IpcResponse> {
        let mut guard = self.state.lock().unwrap();
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::ProcessKeyEvent,
                session_id,
                data: IpcRequestData::KeyEvent(KeyEventData { keycode, modifiers }),
            };
            client.send_request(&request).ok()
        } else {
            None
        }
    }

    pub fn select_candidate(&self, index: usize) -> Option<IpcResponse> {
        let mut guard = self.state.lock().unwrap();
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::SelectCandidate,
                session_id,
                data: IpcRequestData::SelectIndex(index),
            };
            client.send_request(&request).ok()
        } else {
            None
        }
    }

    pub fn change_page(&self, backward: bool) -> Option<IpcResponse> {
        let mut guard = self.state.lock().unwrap();
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::ChangePage,
                session_id,
                data: IpcRequestData::ChangePage(backward),
            };
            client.send_request(&request).ok()
        } else {
            None
        }
    }

    pub fn update_position(&self, x: i32, y: i32) {
        let mut guard = self.state.lock().unwrap();
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::UpdateInputPosition,
                session_id,
                data: IpcRequestData::Position(Position { x, y }),
            };
            let _ = client.send_oneway(&request);
        }
    }

    pub fn toggle_ascii_mode(&self) -> Option<IpcResponse> {
        let mut guard = self.state.lock().unwrap();
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::ToggleAsciiMode,
                session_id,
                data: IpcRequestData::None,
            };
            client.send_request(&request).ok()
        } else {
            None
        }
    }

    pub fn focus_in(&self) {
        let mut guard = self.state.lock().unwrap();
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::FocusIn,
                session_id,
                data: IpcRequestData::None,
            };
            let _ = client.send_oneway(&request);
        }
    }

    pub fn focus_out(&self) {
        let mut guard = self.state.lock().unwrap();
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::FocusOut,
                session_id,
                data: IpcRequestData::None,
            };
            let _ = client.send_oneway(&request);
        }
    }

    pub fn show_tray_icon(&self) {
        let mut guard = self.state.lock().unwrap();
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::ShowTrayIcon,
                session_id: 0,
                data: IpcRequestData::None,
            };
            let _ = client.send_oneway(&request);
        }
    }

    pub fn hide_tray_icon(&self) {
        let mut guard = self.state.lock().unwrap();
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::HideTrayIcon,
                session_id: 0,
                data: IpcRequestData::None,
            };
            let _ = client.send_oneway(&request);
        }
    }
}

impl Clone for IpcClientHandle {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct RimeOutput {
    commit: Option<String>,
    preedit: String,
    _candidates: Vec<String>,
    composing: bool,
}

impl RimeOutput {
    fn from_response(response: &IpcResponse) -> Self {
        let ctx = response.context.as_ref();
        let commit = ctx.and_then(|c| c.commit.clone());
        log(&format!(
            "RimeOutput::from_response: context={}, commit={:?}, preedit='{}'",
            ctx.is_some(),
            commit,
            ctx.map(|c| c.preedit.str.clone()).unwrap_or_default()
        ));
        Self {
            commit,
            preedit: ctx.map(|c| c.preedit.str.clone()).unwrap_or_default(),
            _candidates: ctx
                .map(|c| c.candidates.candies.iter().map(|t| t.str.clone()).collect())
                .unwrap_or_default(),
            composing: response
                .status
                .as_ref()
                .map(|s| s.composing)
                .unwrap_or(false),
        }
    }
}

#[implement(ITfEditSession)]
struct XimeEditSession {
    output: RimeOutput,
    thread_mgr: Option<ITfThreadMgr>,
    composition: Arc<std::sync::Mutex<Option<ITfComposition>>>,
    composition_sink: ITfCompositionSink,
    ipc: IpcClientHandle,
}

#[implement(ITfCompositionSink)]
struct CompositionSink {
    composition: Arc<std::sync::Mutex<Option<ITfComposition>>>,
}

impl ITfCompositionSink_Impl for CompositionSink_Impl {
    fn OnCompositionTerminated(&self, _ecwrite: u32, _pcomposition: Ref<'_, ITfComposition>) -> Result<()> {
        log("OnCompositionTerminated: composition terminated");
        *self.composition.lock().unwrap() = None;
        Ok(())
    }
}

impl ITfEditSession_Impl for XimeEditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        crate::log::log(&format!(
            "DoEditSession: ec={}, commit={}, composing={}, preedit='{}'",
            ec,
            self.output.commit.is_some(),
            self.output.composing,
            self.output.preedit
        ));

        let doc_mgr = match self.thread_mgr.as_ref() {
            Some(t) => unsafe { t.GetFocus() }?,
            None => {
                crate::log::log("DoEditSession: no thread_mgr");
                return Ok(());
            }
        };
        let context = unsafe { doc_mgr.GetBase() }?;
        crate::log::log("DoEditSession: got context");

        // Handle commit and composition in one transaction (like chewing-tsf)
        let commit_text = self.output.commit.clone().unwrap_or_default();
        let preedit_text = self.output.preedit.clone();

        if self.output.composing {
            crate::log::log("DoEditSession: composing mode");
            let comp = self.composition.lock().unwrap().clone();

            if let Some(ref composition) = comp {
                crate::log::log("DoEditSession: update existing composition");
                // Update composition: set full text (commit + preedit), then shift to separate
                unsafe {
                    if let Ok(range) = composition.GetRange() {
                        let full_text = format!("{}{}", commit_text, preedit_text);
                        let wide: Vec<u16> = full_text.encode_utf16().collect();
                        crate::log::log(&format!("DoEditSession: setting full text '{}' ({} chars)", full_text, wide.len()));
                        
                        if range.SetText(ec, 0, &wide).is_ok() {
                            crate::log::log("DoEditSession: SetText succeeded");
                            
                            if !commit_text.is_empty() {
                                let commit_len = commit_text.chars().count() as i32;
                                let mut moved = 0;
                                crate::log::log(&format!("DoEditSession: shifting start by {} chars", commit_len));
                                range.ShiftStart(ec, commit_len, &mut moved, std::ptr::null_mut()).ok();
                                composition.ShiftStart(ec, &range).ok();
                                crate::log::log("DoEditSession: shift start done");
                            }
                            
                            if let Ok(cursor_range) = range.Clone() {
                                let preedit_len = preedit_text.chars().count() as i32;
                                let mut moved = 0;
                                cursor_range.Collapse(ec, TF_ANCHOR_START).ok();
                                cursor_range.ShiftEnd(ec, preedit_len, &mut moved, std::ptr::null_mut()).ok();
                                cursor_range.ShiftStart(ec, preedit_len, &mut moved, std::ptr::null_mut()).ok();
                                
                                use std::mem::ManuallyDrop;
                                let mut selections = [TF_SELECTION::default(); 1];
                                selections[0].range = ManuallyDrop::new(Some(cursor_range));
                                selections[0].style.ase = TF_AE_END;
                                selections[0].style.fInterimChar = FALSE;
                                context.SetSelection(ec, &selections).ok();
                                let [TF_SELECTION { range, .. }] = selections;
                                ManuallyDrop::into_inner(range);
                                
                                Self::update_caret_position_in_session(&context, ec, &self.ipc);
                            }
                        }
                    }
                }
            } else {
                crate::log::log("DoEditSession: start new composition");
                self.start_composition(&context, ec, &preedit_text);
            }
        } else if !commit_text.is_empty() {
            crate::log::log(&format!("DoEditSession: committing '{}' and ending composition", commit_text));
            
            // Replace composition text with commit text first
            let comp = self.composition.lock().unwrap().clone();
            if let Some(ref composition) = comp {
                unsafe {
                    if let Ok(range) = composition.GetRange() {
                        // Replace preedit with commit text
                        let wide: Vec<u16> = commit_text.encode_utf16().collect();
                        crate::log::log(&format!("DoEditSession: replacing composition with commit '{}'", commit_text));
                        if range.SetText(ec, 0, &wide).is_ok() {
                            // Set cursor at end of commit text
                            let commit_len = commit_text.chars().count() as i32;
                            let mut moved = 0;
                            range.Collapse(ec, TF_ANCHOR_START).ok();
                            range.ShiftEnd(ec, commit_len, &mut moved, std::ptr::null_mut()).ok();
                            range.ShiftStart(ec, commit_len, &mut moved, std::ptr::null_mut()).ok();
                            
                            use std::mem::ManuallyDrop;
                            let mut selections = [TF_SELECTION::default(); 1];
                            selections[0].range = ManuallyDrop::new(Some(range));
                            selections[0].style.ase = TF_AE_END;
                            selections[0].style.fInterimChar = FALSE;
                            context.SetSelection(ec, &selections).ok();
                            let [TF_SELECTION { range, .. }] = selections;
                            ManuallyDrop::into_inner(range);
                        }
                    }
                }
            }
            
            // Now end composition - the commit text becomes normal text
            self.end_composition(ec);
            crate::log::log("DoEditSession: commit done");
        } else {
            crate::log::log("DoEditSession: end composition (no commit)");
            self.end_composition(ec);
        }

        Ok(())
    }
}

impl XimeEditSession_Impl {
    fn update_caret_position_in_session(context: &ITfContext, ec: u32, ipc: &IpcClientHandle) {
        use std::mem::ManuallyDrop;
        use std::ops::Deref;

        unsafe {
            let mut selection = [TF_SELECTION::default(); 1];
            let mut selection_len = 0;
            if context.GetSelection(ec, TF_DEFAULT_SELECTION, &mut selection, &mut selection_len).is_ok() {
                if let Some(sel_range) = selection[0].range.deref() {
                    if let Ok(view) = context.GetActiveView() {
                        let mut rc = RECT::default();
                        let mut clipped = BOOL::default();
                        if view.GetTextExt(ec, sel_range, &mut rc, &mut clipped).is_ok() {
                            log(&format!("update_caret_position_in_session: left={}, bottom={}", rc.left, rc.bottom));
                            let _ = ipc.update_position(rc.left, rc.bottom);
                        }
                    }
                }
                let [TF_SELECTION { range, .. }] = selection;
                ManuallyDrop::into_inner(range);
            }
        }
    }

    fn start_composition(&self, context: &ITfContext, ec: u32, preedit: &str) {
        log(&format!("start_composition: preedit='{}'", preedit));
        use windows::Win32::UI::TextServices::{ITfInsertAtSelection, TF_IAS_QUERYONLY};
        
        let insert_at_selection: ITfInsertAtSelection = match context.cast() {
            Ok(s) => s,
            Err(e) => {
                log(&format!("start_composition: cast ITfInsertAtSelection failed: {:?}", e));
                return;
            }
        };

        let ctx_comp: ITfContextComposition = match context.cast() {
            Ok(c) => c,
            Err(e) => {
                log(&format!("start_composition: cast ITfContextComposition failed: {:?}", e));
                return;
            }
        };

        unsafe {
            let range = match insert_at_selection.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[]) {
                Ok(r) => r,
                Err(e) => {
                    log(&format!("start_composition: InsertTextAtSelection failed: {:?}", e));
                    return;
                }
            };
            log("start_composition: got empty range");

            match ctx_comp.StartComposition(ec, &range, Some(&self.composition_sink)) {
                Ok(comp) => {
                    log("start_composition: StartComposition succeeded");
                    let comp_range = match comp.GetRange() {
                        Ok(r) => r,
                        Err(e) => {
                            log(&format!("start_composition: GetRange failed: {:?}", e));
                            return;
                        }
                    };
                    
                    let wide: Vec<u16> = preedit.encode_utf16().collect();
                    log(&format!("start_composition: setting text {} chars", wide.len()));
                    match comp_range.SetText(ec, 0, &wide) {
                        Ok(_) => {
                            log("start_composition: SetText succeeded");
                            let preedit_len = preedit.chars().count() as i32;
                            let mut moved = 0;
                            comp_range.Collapse(ec, TF_ANCHOR_START).ok();
                            comp_range.ShiftEnd(ec, preedit_len, &mut moved, std::ptr::null_mut()).ok();
                            comp_range.ShiftStart(ec, preedit_len, &mut moved, std::ptr::null_mut()).ok();
                            
                            use std::mem::ManuallyDrop;
                            let mut selections = [TF_SELECTION::default(); 1];
                            selections[0].range = ManuallyDrop::new(Some(comp_range));
                            selections[0].style.ase = TF_AE_END;
                            selections[0].style.fInterimChar = FALSE;
                            context.SetSelection(ec, &selections).ok();
                            let [TF_SELECTION { range, .. }] = selections;
                            ManuallyDrop::into_inner(range);
                            
                            *self.composition.lock().unwrap() = Some(comp);
                            
                            Self::update_caret_position_in_session(context, ec, &self.ipc);
                        }
                        Err(e) => {
                            log(&format!("start_composition: SetText failed: {:?}", e));
                        }
                    }
                }
                Err(e) => {
                    log(&format!("start_composition: StartComposition failed: {:?}", e));
                }
            }
        }
    }

    fn update_composition_text(&self, _context: &ITfContext, ec: u32, preedit: &str) {
        log(&format!("update_composition_text: preedit='{}'", preedit));
        let comp = self.composition.lock().unwrap();
        if let Some(ref composition) = *comp {
            unsafe {
                if let Ok(range) = composition.GetRange() {
                    let wide: Vec<u16> = preedit.encode_utf16().collect();
                    match range.SetText(ec, 0, &wide) {
                        Ok(_) => log("update_composition_text: SetText succeeded"),
                        Err(e) => log(&format!("update_composition_text: SetText failed: {:?}", e)),
                    }
                }
            }
        }
    }

    fn end_composition(&self, ec: u32) {
        if let Some(comp) = self.composition.lock().unwrap().take() {
            unsafe {
                comp.EndComposition(ec).ok();
            }
        }
    }
}

#[implement(ITfKeyEventSink)]
pub struct KeyEventSink {
    ipc: IpcClientHandle,
    composing: std::sync::atomic::AtomicBool,
    thread_mgr: std::sync::Mutex<Option<ITfThreadMgr>>,
    client_id: std::sync::atomic::AtomicU32,
    composition: Arc<std::sync::Mutex<Option<ITfComposition>>>,
    ascii_mode: crate::language_bar::SharedAsciiMode,
    lang_bar_sink_ref: crate::language_bar::LangBarSinkRef,
}

impl KeyEventSink {
    pub fn new(ipc: IpcClientHandle, ascii_mode: crate::language_bar::SharedAsciiMode, sink_ref: crate::language_bar::LangBarSinkRef) -> Self {
        Self {
            ipc,
            composing: std::sync::atomic::AtomicBool::new(false),
            thread_mgr: std::sync::Mutex::new(None),
            client_id: std::sync::atomic::AtomicU32::new(0),
            composition: Arc::new(std::sync::Mutex::new(None)),
            ascii_mode,
            lang_bar_sink_ref: sink_ref,
        }
    }

    pub fn set_thread_mgr(&self, mgr: Option<ITfThreadMgr>) {
        *self.thread_mgr.lock().unwrap() = mgr;
    }

    pub fn set_client_id(&self, id: u32) {
        self.client_id
            .store(id, std::sync::atomic::Ordering::Release);
    }

    fn update_lang_bar(&self) {
        if let Some(ref sink) = *self.lang_bar_sink_ref.lock().unwrap() {
            unsafe {
                let _ = sink.OnUpdate(TF_LBI_STATUS | TF_LBI_ICON | TF_LBI_TEXT);
            }
        }
    }

    fn is_composing(&self) -> bool {
        self.composing.load(std::sync::atomic::Ordering::Acquire)
    }

    fn set_composing(&self, val: bool) {
        self.composing
            .store(val, std::sync::atomic::Ordering::Release);
    }

    fn should_handle_key(&self, vk: VIRTUAL_KEY) -> bool {
        let code = vk.0;
        if (VK_X_A..=VK_X_Z).contains(&code) {
            return true;
        }
        if code == VK_RETURN.0 || code == VK_BACK.0 || code == VK_ESCAPE.0 {
            return true;
        }
        if self.is_composing() {
            if code == VK_SPACE.0 {
                return true;
            }
            if (VK_X_0..=VK_X_9).contains(&code) {
                return true;
            }
            if code == VK_UP.0 || code == VK_DOWN.0 || code == VK_LEFT.0 || code == VK_RIGHT.0 || code == VK_PRIOR.0 || code == VK_NEXT.0 {
                return true;
            }
        }
        false
    }

    fn update_caret_position_sync(&self, context: &ITfContext) {
        log("update_caret_position_sync: getting caret rect");

        use windows::Win32::Foundation::RECT;
        use windows::Win32::UI::TextServices::{
            ITfEditSession, TF_DEFAULT_SELECTION, TF_ES_READ, TF_ES_SYNC,
        };

        #[implement(ITfEditSession)]
        struct SelectionRect {
            context: ITfContext,
            rect: std::cell::Cell<RECT>,
        }

        impl SelectionRect {
            fn new(context: ITfContext) -> Self {
                Self {
                    context,
                    rect: std::cell::Cell::default(),
                }
            }
            fn rect(&self) -> RECT {
                self.rect.get()
            }
        }

        impl ITfEditSession_Impl for SelectionRect_Impl {
            fn DoEditSession(&self, ec: u32) -> Result<()> {
                use std::mem::ManuallyDrop;
                use std::ops::Deref;

                unsafe {
                    let mut selection = [TF_SELECTION::default(); 1];
                    let mut selection_len = 0;
                    self.context.GetSelection(
                        ec,
                        TF_DEFAULT_SELECTION,
                        &mut selection,
                        &mut selection_len,
                    )?;

                    if let Some(sel_range) = selection[0].range.deref() {
                        let view = self.context.GetActiveView()?;
                        let mut rc = RECT::default();
                        let mut clipped = BOOL::default();
                        view.GetTextExt(ec, sel_range, &mut rc, &mut clipped)?;
                        log(&format!(
                            "SelectionRect: left={}, top={}, right={}, bottom={}",
                            rc.left, rc.top, rc.right, rc.bottom
                        ));
                        self.rect.set(rc);
                    }

                    let [TF_SELECTION { range, .. }] = selection;
                    ManuallyDrop::into_inner(range);
                }
                Ok(())
            }
        }

        let tid = self.client_id.load(std::sync::atomic::Ordering::Acquire);
        let session = SelectionRect::new(context.clone()).into_object();

        let _ = unsafe {
            context.RequestEditSession(tid, session.as_interface(), TF_ES_SYNC | TF_ES_READ)
        };

        let rect = session.rect();
        log(&format!(
            "update_caret_position_sync: left={}, bottom={}",
            rect.left, rect.bottom
        ));

        if self.ipc.is_connected() {
            let _ = self.ipc.update_position(rect.left, rect.bottom);
            log("update_caret_position_sync: sent to server");
        }
    }

    fn schedule_edit_session(&self, context: &ITfContext, output: RimeOutput) {
        log("schedule_edit_session: called");
        let tid = self.client_id.load(std::sync::atomic::Ordering::Acquire);
        let mgr = self.thread_mgr.lock().unwrap().clone();

        let composition_sink_impl = CompositionSink {
            composition: self.composition.clone(),
        };
        let composition_sink: ITfCompositionSink = composition_sink_impl.into();

        let session = XimeEditSession {
            output,
            thread_mgr: mgr,
            composition: self.composition.clone(),
            composition_sink,
            ipc: self.ipc.clone(),
        };
        let session_itf: ITfEditSession = session.into();
        log(&format!(
            "schedule_edit_session: tid={}, calling RequestEditSession",
            tid
        ));
        unsafe {
            let result = context.RequestEditSession(
                tid,
                &session_itf,
                TF_ES_ASYNCDONTCARE | TF_ES_READWRITE,
            );
            log(&format!(
                "schedule_edit_session: RequestEditSession result: {:?}",
                result
            ));
        }
    }

    fn handle_key_event(&self, context: Option<&ITfContext>, vk: VIRTUAL_KEY) -> bool {
        log(&format!("handle_key_event: vk={}", vk.0));

        if !self.ipc.is_connected() {
            log("  -> IPC not connected, attempting reconnect...");
            if self.ipc.connect().is_ok() {
                self.ipc.start_session();
                log("  -> Reconnected!");
            } else {
                log("  -> Reconnect failed");
                return false;
            }
        }

        log("  -> IPC connected");
        let code = vk.0;
        let is_composing = self.is_composing();

        if is_composing && code >= 0x31 && code <= 0x39 {
            let index = (code - 0x31) as usize;
            if let Some(response) = self.ipc.select_candidate(index) {
                let output = RimeOutput::from_response(&response);
                self.set_composing(output.composing);
                if let Some(ctx) = context {
                    self.schedule_edit_session(ctx, output);
                }
                return true;
            }
            return false;
        }

        if is_composing && code == VK_PRIOR.0 {
            if let Some(response) = self.ipc.change_page(true) {
                let output = RimeOutput::from_response(&response);
                self.set_composing(output.composing);
                if let Some(ctx) = context {
                    self.schedule_edit_session(ctx, output);
                }
                return true;
            }
            return false;
        }
        if is_composing && code == VK_NEXT.0 {
            if let Some(response) = self.ipc.change_page(false) {
                let output = RimeOutput::from_response(&response);
                self.set_composing(output.composing);
                if let Some(ctx) = context {
                    self.schedule_edit_session(ctx, output);
                }
                return true;
            }
            return false;
        }

        if let Some(ctx) = context {
            self.update_caret_position_sync(ctx);
        }

        let xk = librime_sys::vk_to_xk(code);
        let mods = librime_sys::get_key_modifiers();
        log(&format!("  -> calling process_key({}, {})", xk, mods));
        let response = self.ipc.process_key(xk, mods);
        log(&format!("  -> response: {:?}", response));
        if let Some(response) = response {
            log(&format!("  -> success={}", response.success));
            if response.success {
                let output = RimeOutput::from_response(&response);
                self.set_composing(output.composing);
                log(&format!(
                    "  -> context is {}",
                    if context.is_some() { "Some" } else { "None" }
                ));
                if let Some(ctx) = context {
                    log("  -> calling schedule_edit_session");
                    self.schedule_edit_session(ctx, output);
                    log("  -> schedule_edit_session returned");
                }
                return true;
            }
        }
        log("  -> returning false");
        false
    }
}

impl ITfKeyEventSink_Impl for KeyEventSink_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        _pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        let vk = VIRTUAL_KEY(wparam.0 as u16);
        log(&format!("OnTestKeyDown: vk={}", vk.0));
        if !self.should_handle_key(vk) {
            log("  -> not handling");
            return Ok(BOOL(0));
        }

        if !self.ipc.is_connected() {
            log("  -> IPC not connected");
            return Ok(BOOL(0));
        }

        let xk = librime_sys::vk_to_xk(vk.0);
        let mods = librime_sys::get_key_modifiers();
        log(&format!("  -> process_key({}, {})", xk, mods));
        let handled = self
            .ipc
            .process_key(xk, mods)
            .map(|r| r.success)
            .unwrap_or(false);
        log(&format!("  -> result: {}", handled));

        Ok(BOOL(if handled { 1 } else { 0 }))
    }

    fn OnKeyDown(&self, pic: Ref<'_, ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let vk = VIRTUAL_KEY(wparam.0 as u16);
        log(&format!("OnKeyDown: vk={}", vk.0));
        if !self.should_handle_key(vk) {
            log("  -> not handling");
            return Ok(BOOL(0));
        }

        let context = pic.as_ref();
        let handled = self.handle_key_event(context, vk);
        log(&format!("  -> result: {}", handled));
        Ok(BOOL(if handled { 1 } else { 0 }))
    }

    fn OnTestKeyUp(
        &self,
        _pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        let vk = VIRTUAL_KEY(wparam.0 as u16);
        if vk.0 == VK_SHIFT.0 {
            return Ok(BOOL(1));
        }
        Ok(BOOL(0))
    }

    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let vk = VIRTUAL_KEY(wparam.0 as u16);
        log(&format!("OnKeyUp: vk={}", vk.0));
        
        if vk.0 != VK_SHIFT.0 {
            return Ok(BOOL(0));
        }

        if !self.ipc.is_connected() {
            log("  -> IPC not connected");
            return Ok(BOOL(0));
        }

        log("  -> calling toggle_ascii_mode");
        let response = self.ipc.toggle_ascii_mode();
        log(&format!("  -> response: {:?}", response));
        
        if let Some(response) = response {
            if response.success {
                if let Some(status) = response.status {
                    log(&format!("  -> ascii_mode: {}", status.ascii_mode));
                    self.ascii_mode.store(status.ascii_mode, std::sync::atomic::Ordering::Release);
                    self.update_lang_bar();
                }
                return Ok(BOOL(1));
            }
        }
        
        Ok(BOOL(0))
    }

    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, _rguid: *const GUID) -> Result<BOOL> {
        Ok(BOOL(0))
    }
}

#[implement(ITfTextInputProcessor)]
pub struct XimeTextService {
    thread_mgr: std::cell::RefCell<Option<ITfThreadMgr>>,
    client_id: std::cell::Cell<u32>,
    ipc: IpcClientHandle,
    key_sink: std::cell::RefCell<Option<ITfKeyEventSink>>,
    keystroke_mgr: std::cell::RefCell<Option<ITfKeystrokeMgr>>,
    lang_bar_mgr: std::cell::RefCell<Option<ITfLangBarItemMgr>>,
    ascii_mode: crate::language_bar::SharedAsciiMode,
    lang_bar_sink_ref: crate::language_bar::LangBarSinkRef,
    lang_bar_item: std::cell::RefCell<Option<ITfLangBarItemButton>>,
}

pub const GUID_LANG_BAR_ITEM: GUID = GUID_LBI_INPUTMODE;

pub const CLSID_TEXT_SERVICE: GUID = GUID {
    data1: 0x5E1E4B52,
    data2: 0x4A6D,
    data3: 0x4F3A,
    data4: [0xB5, 0xC7, 0xD8, 0xE9, 0xF0, 0x1A, 0x2B, 0x3C],
};

impl XimeTextService {
    pub fn new() -> Self {
        let ipc = IpcClientHandle::empty();
        log(&format!(
            "XimeTextService::new() called, IPC state ptr={:p}",
            ipc.state
        ));
        Self {
            thread_mgr: std::cell::RefCell::new(None),
            client_id: std::cell::Cell::new(0),
            ipc,
            key_sink: std::cell::RefCell::new(None),
            keystroke_mgr: std::cell::RefCell::new(None),
            lang_bar_mgr: std::cell::RefCell::new(None),
            ascii_mode: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            lang_bar_sink_ref: Arc::new(std::sync::Mutex::new(None)),
            lang_bar_item: std::cell::RefCell::new(None),
        }
    }

    fn ensure_ipc(&self) -> std::result::Result<(), winxime_ipc::IpcError> {
        let connected = self.ipc.is_connected();
        log(&format!(
            "ensure_ipc: is_connected={}, ptr={:p}",
            connected,
            self.ipc.debug_ptr()
        ));
        if connected {
            return Ok(())
        }
        log("ensure_ipc: attempting connect...");
        match self.ipc.connect() {
            Ok(()) => {
                let (_, response) = self.ipc.start_session();
                if let Some(response) = response {
                    if let Some(status) = response.status {
                        log(&format!("ensure_ipc: initial ascii_mode={}", status.ascii_mode));
                        self.ascii_mode.store(status.ascii_mode, std::sync::atomic::Ordering::Release);
                    }
                }
                log("ensure_ipc: connect OK");
                Ok(())
            }
            Err(e) => {
                log(&format!("ensure_ipc: connect FAILED: {:?}", e));
                Err(e)
            }
        }
    }
}

impl XimeTextService_Impl {
    fn activate_impl(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        init_log();
        log(&format!("Activate called, tid={}", tid));

        *self.thread_mgr.borrow_mut() = ptim.cloned();
        self.client_id.set(tid);

        match self.ensure_ipc() {
            Ok(()) => {
                log("IPC connected successfully");
                self.ipc.show_tray_icon();
                if let Some(response) = self.ipc.start_session().1 {
                    if let Some(status) = response.status {
                        self.ascii_mode.store(status.ascii_mode, std::sync::atomic::Ordering::Release);
                    }
                }
            }
            Err(e) => log(&format!("IPC connection failed: {:?}", e)),
        }

        let sink_impl = KeyEventSink::new(self.ipc.clone(), self.ascii_mode.clone(), self.lang_bar_sink_ref.clone());
        sink_impl.set_client_id(tid);
        sink_impl.set_thread_mgr(ptim.cloned());
        log("KeyEventSink created with IPC handle");

        let key_sink_itf: ITfKeyEventSink = sink_impl.into();

        if let Some(thread_mgr) = ptim {
            if let Ok(kmgr) = thread_mgr.cast::<ITfKeystrokeMgr>() {
                log("Got ITfKeystrokeMgr, attempting AdviseKeyEventSink...");
                if unsafe { kmgr.AdviseKeyEventSink(tid, &key_sink_itf, true).is_ok() } {
                    log("AdviseKeyEventSink succeeded");
                    *self.keystroke_mgr.borrow_mut() = Some(kmgr);
                    *self.key_sink.borrow_mut() = Some(key_sink_itf);
                } else {
                    log("AdviseKeyEventSink failed");
                }
            } else {
                log("Failed to get ITfKeystrokeMgr");
            }

            if let Ok(lang_bar_mgr) = thread_mgr.cast::<ITfLangBarItemMgr>() {
                log("Got ITfLangBarItemMgr, creating LangBarItem...");
                let lang_bar = crate::language_bar::LangBarItemButton::new(
                    GUID_LANG_BAR_ITEM,
                    self.ipc.clone(),
                    self.ascii_mode.clone(),
                    self.lang_bar_sink_ref.clone(),
                );
                let lang_bar_itf: ITfLangBarItemButton = lang_bar.into();
                let add_result = unsafe { lang_bar_mgr.AddItem(&lang_bar_itf) };
                match add_result {
                    Ok(_) => {
                        log("LangBarItem added successfully");
                        *self.lang_bar_mgr.borrow_mut() = Some(lang_bar_mgr);
                        *self.lang_bar_item.borrow_mut() = Some(lang_bar_itf);
                    }
                    Err(e) => {
                        log(&format!("LangBarItem AddItem failed: {:?}", e));
                    }
                }
            } else {
                log("Failed to get ITfLangBarItemMgr");
            }
        } else {
            log("No thread_mgr available");
        }

        Ok(())
    }

    fn deactivate_impl(&self) -> Result<()> {
        log("Deactivate called");
        self.ipc.hide_tray_icon();
        
        if let Some(kmgr) = self.keystroke_mgr.borrow_mut().take() {
            unsafe {
                let _ = kmgr.UnadviseKeyEventSink(self.client_id.get());
            }
        }

        if let Some(lang_bar_mgr) = self.lang_bar_mgr.borrow_mut().take() {
            if let Some(lang_bar_item) = self.lang_bar_item.borrow_mut().take() {
                unsafe {
                    if let Err(e) = lang_bar_mgr.RemoveItem(&lang_bar_item) {
                        log(&format!("RemoveItem failed: {:?}", e));
                    }
                }
            }
        }

        *self.key_sink.borrow_mut() = None;
        *self.thread_mgr.borrow_mut() = None;
        self.client_id.set(0);
        Ok(())
    }
}

impl ITfTextInputProcessor_Impl for XimeTextService_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        self.activate_impl(ptim.as_ref(), tid)
    }

    fn Deactivate(&self) -> Result<()> {
        self.deactivate_impl()
    }
}
