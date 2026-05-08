#![windows_subsystem = "windows"]

use std::{env, process};
use windows::{
    Win32::{
        Globalization::*,
        System::{
            Com::*,
            Console::{ATTACH_PARENT_PROCESS, AttachConsole},
            Diagnostics::Debug::IsDebuggerPresent,
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
        if IsDebuggerPresent().as_bool() {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;

        if env::args().len() == 1 {
            println!("Usage:");
            println!("  winxime-tsf-register -r <IconPath>    注册输入法");
            println!("  winxime-tsf-register -i               立即启用输入法");
            println!("  winxime-tsf-register -d               立即停用输入法");
            println!("  winxime-tsf-register -u               取消注册");
            println!("  winxime-tsf-register -s               停止 winxime-server");
            process::exit(1);
        }

        if let Some("-r") = env::args().nth(1).as_deref() {
            let icon_path = env::args().nth(2).expect("缺少 IconPath");
            register(icon_path)?;
        } else if let Some("-i") = env::args().nth(1).as_deref() {
            enable();
        } else if let Some("-d") = env::args().nth(1).as_deref() {
            disable();
        } else if let Some("-s") = env::args().nth(1).as_deref() {
            stop();
        } else if let Err(err) = unregister() {
            println!("警告：无法解除输入法注册，卸载可能无法正常完成。");
            println!("错误信息：{:?}", err);
        }
    }

    Ok(())
}