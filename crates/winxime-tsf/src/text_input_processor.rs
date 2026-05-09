use crate::log::{init_log, log};
use std::sync::Arc;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::TextServices::*;
use windows_core::{*, Interface};
use winxime_ipc::{
    IpcClient, IpcCommand, IpcRequest, IpcRequestData, IpcResponse, KeyEventData, Position,
};

const TF_INVALID_COOKIE: u32 = 0xFFFFFFFF;

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
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
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

    pub fn disconnect(&self) {
        log("IpcClientHandle::disconnect() called");
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        guard.client = None;
        guard.session_id = 0;
    }

    pub fn is_connected(&self) -> bool {
        let r = self.state.lock().unwrap_or_else(|e| e.into_inner()).client.is_some();
        r
    }

    pub fn start_session(&self) -> (u32, Option<IpcResponse>) {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::StartSession,
                session_id: 0,
                data: IpcRequestData::None,
            };
            log(&format!("start_session: sending request, session_id=0"));
            match client.send_request(&request) {
                Ok(response) => {
                    log(&format!("start_session: got response, session_id={}, ascii_mode={}", 
                        response.session_id, 
                        response.status.as_ref().map(|s| s.ascii_mode).unwrap_or(false)));
                    guard.session_id = response.session_id;
                    return (guard.session_id, Some(response));
                }
                Err(e) => {
                    log(&format!("start_session: send_request FAILED: {:?}", e));
                    guard.client = None;
                    guard.session_id = 0;
                }
            }
        } else {
            log("start_session: no client");
        }
        (guard.session_id, None)
    }

    pub fn process_key(&self, keycode: i32, modifiers: i32) -> Option<IpcResponse> {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::ProcessKeyEvent,
                session_id,
                data: IpcRequestData::KeyEvent(KeyEventData { keycode, modifiers }),
            };
            match client.send_request(&request) {
                Ok(response) => {
                    log(&format!("process_key: got response, success={}", response.success));
                    return Some(response);
                }
                Err(e) => {
                    log(&format!("process_key: send_request FAILED: {:?}", e));
                    guard.client = None;
                    guard.session_id = 0;
                }
            }
        } else {
            log("process_key: no client");
        }
        None
    }

    pub fn select_candidate(&self, index: usize) -> Option<IpcResponse> {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
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
        log("IPC::focus_in() called");
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::FocusIn,
                session_id,
                data: IpcRequestData::None,
            };
            log(&format!("  -> sending FocusIn request (session_id={})", session_id));
            if client.send_oneway(&request).is_ok() {
                log("  -> FocusIn sent successfully");
            } else {
                log("  -> FocusIn send FAILED");
            }
        } else {
            log("  -> no client, cannot send FocusIn");
        }
    }

    pub fn focus_out(&self) {
        log("IPC::focus_out() called");
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let session_id = guard.session_id;
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::FocusOut,
                session_id,
                data: IpcRequestData::None,
            };
            log(&format!("  -> sending FocusOut request (session_id={})", session_id));
            if client.send_oneway(&request).is_ok() {
                log("  -> FocusOut sent successfully");
            } else {
                log("  -> FocusOut send FAILED");
            }
        } else {
            log("  -> no client, cannot send FocusOut");
        }
    }

    pub fn show_tray_icon(&self) {
        log("IPC::show_tray_icon() called");
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::ShowTrayIcon,
                session_id: 0,
                data: IpcRequestData::None,
            };
            log("  -> sending ShowTrayIcon request");
            if client.send_oneway(&request).is_ok() {
                log("  -> ShowTrayIcon sent successfully");
            } else {
                log("  -> ShowTrayIcon send FAILED");
            }
        } else {
            log("  -> no client");
        }
    }

    pub fn hide_tray_icon(&self) {
        log("IPC::hide_tray_icon() called");
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut client) = guard.client {
            let request = IpcRequest {
                command: IpcCommand::HideTrayIcon,
                session_id: 0,
                data: IpcRequestData::None,
            };
            log("  -> sending HideTrayIcon request");
            if client.send_oneway(&request).is_ok() {
                log("  -> HideTrayIcon sent successfully");
            } else {
                log("  -> HideTrayIcon send FAILED");
            }
        } else {
            log("  -> no client");
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
        *self.composition.lock().unwrap_or_else(|e| e.into_inner()) = None;
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
            let comp = self.composition.lock().unwrap_or_else(|e| e.into_inner()).clone();

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
            let comp = self.composition.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
                            
                            *self.composition.lock().unwrap_or_else(|e| e.into_inner()) = Some(comp);
                            
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
        let comp = self.composition.lock().unwrap_or_else(|e| e.into_inner());
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
        if let Some(comp) = self.composition.lock().unwrap_or_else(|e| e.into_inner()).take() {
            unsafe {
                comp.EndComposition(ec).ok();
            }
        }
    }
}

impl XimeTextService_Impl {
    fn is_composing(&self) -> bool {
        self.composing.get()
    }

    fn set_composing(&self, val: bool) {
        self.composing.set(val);
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

    fn update_lang_bar(&self) {
        if let Some(ref sink) = *self.lang_bar_sink_ref.lock().unwrap_or_else(|e| e.into_inner()) {
            unsafe {
                let _ = sink.OnUpdate(TF_LBI_STATUS | TF_LBI_ICON | TF_LBI_TEXT);
            }
        }
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

        let tid = self.client_id.get();
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
        let tid = self.client_id.get();
        let mgr = self.thread_mgr.borrow().clone();

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

impl ITfKeyEventSink_Impl for XimeTextService_Impl {
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
        let handled = self.should_handle_key(vk);
        log(&format!("  -> should_handle_key: {}", handled));
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

#[implement(ITfTextInputProcessor, ITfActiveLanguageProfileNotifySink, ITfThreadFocusSink, ITfThreadMgrEventSink, ITfKeyEventSink)]
pub struct XimeTextService {
    thread_mgr: std::cell::RefCell<Option<ITfThreadMgr>>,
    client_id: std::cell::Cell<u32>,
    ipc: IpcClientHandle,
    composing: std::cell::Cell<bool>,
    composition: Arc<std::sync::Mutex<Option<ITfComposition>>>,
    lang_bar_mgr: std::cell::RefCell<Option<ITfLangBarItemMgr>>,
    ascii_mode: crate::language_bar::SharedAsciiMode,
    lang_bar_sink_ref: crate::language_bar::LangBarSinkRef,
    lang_bar_item: std::cell::RefCell<Option<ITfLangBarItemButton>>,
    profile_sink_cookie: std::cell::Cell<u32>,
    profile_source: std::cell::RefCell<Option<ITfSource>>,
    thread_focus_sink_cookie: std::cell::Cell<u32>,
    thread_focus_source: std::cell::RefCell<Option<ITfSource>>,
    thread_mgr_event_sink_cookie: std::cell::Cell<u32>,
}

pub const GUID_LANG_BAR_ITEM: GUID = GUID_LBI_INPUTMODE;

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
            composing: std::cell::Cell::new(false),
            composition: Arc::new(std::sync::Mutex::new(None)),
            lang_bar_mgr: std::cell::RefCell::new(None),
            ascii_mode: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            lang_bar_sink_ref: Arc::new(std::sync::Mutex::new(None)),
            lang_bar_item: std::cell::RefCell::new(None),
            profile_sink_cookie: std::cell::Cell::new(TF_INVALID_COOKIE),
            profile_source: std::cell::RefCell::new(None),
            thread_focus_sink_cookie: std::cell::Cell::new(TF_INVALID_COOKIE),
            thread_focus_source: std::cell::RefCell::new(None),
            thread_mgr_event_sink_cookie: std::cell::Cell::new(TF_INVALID_COOKIE),
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

        if let Some(thread_mgr) = ptim {
            // Register ITfKeyEventSink directly (self implements ITfKeyEventSink)
            if let Ok(kmgr) = thread_mgr.cast::<ITfKeystrokeMgr>() {
                log("Got ITfKeystrokeMgr, attempting AdviseKeyEventSink...");
                use windows_core::ComObjectInterface;
                let key_sink_ref = ComObjectInterface::<ITfKeyEventSink>::as_interface_ref(self);
                let key_sink: ITfKeyEventSink = key_sink_ref.to_owned();
                unsafe {
                    if kmgr.AdviseKeyEventSink(tid, &key_sink, true).is_ok() {
                        log("AdviseKeyEventSink succeeded");
                    } else {
                        log("AdviseKeyEventSink failed");
                    }
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

            // Register profile activation sink for input method switching
            if let Ok(source) = thread_mgr.cast::<ITfSource>() {
                log("Got ITfSource from thread_mgr, registering sinks...");
                use windows_core::ComObjectInterface;
                
                // Register ITfActiveLanguageProfileNotifySink
                let profile_sink_ref = ComObjectInterface::<ITfActiveLanguageProfileNotifySink>::as_interface_ref(self);
                let profile_sink: ITfActiveLanguageProfileNotifySink = profile_sink_ref.to_owned();
                unsafe {
                    if let Ok(cookie) = source.AdviseSink(&ITfActiveLanguageProfileNotifySink::IID, &profile_sink) {
                        log(&format!("Profile sink registered, cookie={}", cookie));
                        self.profile_sink_cookie.set(cookie);
                        *self.profile_source.borrow_mut() = Some(source.clone());
                    } else {
                        log("Failed to AdviseSink for profile");
                    }
                }
                
                // Register ITfThreadFocusSink
                let thread_focus_sink_ref = ComObjectInterface::<ITfThreadFocusSink>::as_interface_ref(self);
                let thread_focus_sink: ITfThreadFocusSink = thread_focus_sink_ref.to_owned();
                unsafe {
                    if let Ok(cookie) = source.AdviseSink(&ITfThreadFocusSink::IID, &thread_focus_sink) {
                        log(&format!("Thread focus sink registered, cookie={}", cookie));
                        self.thread_focus_sink_cookie.set(cookie);
                        *self.thread_focus_source.borrow_mut() = Some(source.clone());
                    } else {
                        log("Failed to AdviseSink for thread focus");
                    }
                }
                
                // Register ITfThreadMgrEventSink (for document focus changes)
                let thread_mgr_event_sink_ref = ComObjectInterface::<ITfThreadMgrEventSink>::as_interface_ref(self);
                let thread_mgr_event_sink: ITfThreadMgrEventSink = thread_mgr_event_sink_ref.to_owned();
                unsafe {
                    if let Ok(cookie) = source.AdviseSink(&ITfThreadMgrEventSink::IID, &thread_mgr_event_sink) {
                        log(&format!("ThreadMgrEventSink registered, cookie={}", cookie));
                        self.thread_mgr_event_sink_cookie.set(cookie);
                    } else {
                        log("Failed to AdviseSink for ThreadMgrEventSink");
                    }
                }
            } else {
                log("Failed to get ITfSource from thread_mgr");
            }
        } else {
            log("No thread_mgr available");
        }

        Ok(())
    }

    fn deactivate_impl(&self) -> Result<()> {
        log("Deactivate called");
        self.ipc.hide_tray_icon();
        
        // Unregister profile sink
        if self.profile_sink_cookie.get() != TF_INVALID_COOKIE {
            if let Some(source) = self.profile_source.borrow_mut().take() {
                unsafe {
                    let _ = source.UnadviseSink(self.profile_sink_cookie.get());
                }
            }
            self.profile_sink_cookie.set(TF_INVALID_COOKIE);
        }
        
        // Unregister thread focus sink
        if self.thread_focus_sink_cookie.get() != TF_INVALID_COOKIE {
            if let Some(source) = self.thread_focus_source.borrow_mut().take() {
                unsafe {
                    let _ = source.UnadviseSink(self.thread_focus_sink_cookie.get());
                }
            }
            self.thread_focus_sink_cookie.set(TF_INVALID_COOKIE);
        }
        
        // Unregister thread mgr event sink
        if self.thread_mgr_event_sink_cookie.get() != TF_INVALID_COOKIE {
            if let Some(thread_mgr) = self.thread_mgr.borrow().as_ref() {
                if let Ok(source) = thread_mgr.cast::<ITfSource>() {
                    unsafe {
                        let _ = source.UnadviseSink(self.thread_mgr_event_sink_cookie.get());
                    }
                }
            }
            self.thread_mgr_event_sink_cookie.set(TF_INVALID_COOKIE);
        }
        
        // UnadviseKeyEventSink using thread_mgr
        if let Some(thread_mgr) = self.thread_mgr.borrow().as_ref() {
            if let Ok(kmgr) = thread_mgr.cast::<ITfKeystrokeMgr>() {
                unsafe {
                    let _ = kmgr.UnadviseKeyEventSink(self.client_id.get());
                }
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

        *self.thread_mgr.borrow_mut() = None;
        self.client_id.set(0);
        self.composing.set(false);
        *self.composition.lock().unwrap_or_else(|e| e.into_inner()) = None;
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

impl ITfActiveLanguageProfileNotifySink_Impl for XimeTextService_Impl {
    fn OnActivated(&self, clsid: *const GUID, guidprofile: *const GUID, factivated: BOOL) -> Result<()> {
        let clsid_ref = unsafe { clsid.as_ref() };
        let profile_ref = unsafe { guidprofile.as_ref() };
        
        log(&format!("ITfActiveLanguageProfileNotifySink::OnActivated: clsid={:?}, profile={:?}, activated={}", 
            clsid_ref, profile_ref, factivated.0));
        
        let our_clsid = crate::class_factory::CLSID_XIME;
        log(&format!("  -> our_clsid = {:?}", our_clsid));
        
        if let Some(clsid_val) = clsid_ref {
            log(&format!("  -> clsid_val = {:?}", clsid_val));
            log(&format!("  -> clsid_val == our_clsid? {}", clsid_val == &our_clsid));
            if clsid_val != &our_clsid {
                log("  -> Not our TIP, ignoring");
                return Ok(());
            }
            log("  -> clsid matches our TIP!");
        } else {
            log("  -> clsid_ref is None");
        }
        
        log(&format!("  -> factivated.as_bool() = {}", factivated.as_bool()));
        
        if factivated.as_bool() {
            log("  -> Our TIP is being activated!");
            
            if !self.ipc.is_connected() {
                log("  -> IPC not connected, attempting reconnect...");
                if self.ipc.connect().is_err() {
                    log("  -> Reconnect failed, skipping");
                    return Ok(());
                }
                log("  -> Reconnected!");
            }
            
            if let Some(response) = self.ipc.start_session().1 {
                if let Some(status) = response.status {
                    log(&format!("  -> ascii_mode from server: {}", status.ascii_mode));
                    self.ascii_mode.store(status.ascii_mode, std::sync::atomic::Ordering::Release);
                    
                    if let Some(ref sink) = *self.lang_bar_sink_ref.lock().unwrap_or_else(|e| e.into_inner()) {
                        unsafe {
                            let _ = sink.OnUpdate(TF_LBI_STATUS | TF_LBI_ICON | TF_LBI_TEXT);
                        }
                    }
                }
            }
            
            self.ipc.show_tray_icon();
        } else {
            log("  -> Our TIP is being deactivated");
            self.ipc.hide_tray_icon();
        }
        
        Ok(())
    }
}

impl ITfThreadFocusSink_Impl for XimeTextService_Impl {
    fn OnSetThreadFocus(&self) -> Result<()> {
        log("ITfThreadFocusSink::OnSetThreadFocus");
        
        // Ensure IPC connected
        if !self.ipc.is_connected() {
            log("  -> IPC not connected, attempting reconnect...");
            if self.ipc.connect().is_err() {
                log("  -> Reconnect failed");
                return Ok(());
            }
            log("  -> Reconnected!");
        }
        
        // Sync status with server
        if let Some(response) = self.ipc.start_session().1 {
            if let Some(status) = response.status {
                log(&format!("  -> ascii_mode from server: {}", status.ascii_mode));
                self.ascii_mode.store(status.ascii_mode, std::sync::atomic::Ordering::Release);
                
                // Update language bar
                if let Some(ref sink) = *self.lang_bar_sink_ref.lock().unwrap_or_else(|e| e.into_inner()) {
                    unsafe {
                        let _ = sink.OnUpdate(TF_LBI_STATUS | TF_LBI_ICON | TF_LBI_TEXT);
                    }
                }
            }
        }
        
        // Show tray icon
        self.ipc.show_tray_icon();
        
        Ok(())
    }
    
    fn OnKillThreadFocus(&self) -> Result<()> {
        log("ITfThreadFocusSink::OnKillThreadFocus");
        
        // Hide UI when thread loses focus
        self.ipc.hide_tray_icon();
        
        Ok(())
    }
}

impl ITfThreadMgrEventSink_Impl for XimeTextService_Impl {
    fn OnInitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnUninitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnSetFocus(
        &self,
        pdimfocus: Ref<'_, ITfDocumentMgr>,
        _pdimprevfocus: Ref<'_, ITfDocumentMgr>,
    ) -> Result<()> {
        log(&format!("ITfThreadMgrEventSink::OnSetFocus (pdimfocus.is_null={})", pdimfocus.is_null()));
        
        if pdimfocus.is_null() {
            log("  -> Focus lost (pdimfocus is null)");
            if self.ipc.is_connected() {
                self.ipc.focus_out();
            }
            self.composing.set(false);
            return Ok(());
        }
        
        log("  -> Focus gained (pdimfocus is valid)");
        
        if !self.ipc.is_connected() {
            log("  -> IPC not connected, attempting reconnect...");
            if self.ipc.connect().is_err() {
                log("  -> Reconnect failed");
                return Ok(());
            }
            log("  -> Reconnected!");
        }
        
        if let Some(response) = self.ipc.start_session().1 {
            if let Some(status) = response.status {
                log(&format!("  -> ascii_mode from server: {}", status.ascii_mode));
                self.ascii_mode.store(status.ascii_mode, std::sync::atomic::Ordering::Release);
                
                if let Some(ref sink) = *self.lang_bar_sink_ref.lock().unwrap_or_else(|e| e.into_inner()) {
                    unsafe {
                        let _ = sink.OnUpdate(TF_LBI_STATUS | TF_LBI_ICON | TF_LBI_TEXT);
                    }
                }
            }
        }
        
        self.ipc.show_tray_icon();
        self.ipc.focus_in();
        
        Ok(())
    }

    fn OnPushContext(&self, _pic: Ref<'_, ITfContext>) -> Result<()> {
        Ok(())
    }

    fn OnPopContext(&self, _pic: Ref<'_, ITfContext>) -> Result<()> {
        Ok(())
    }
}
