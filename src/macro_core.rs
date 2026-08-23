use crate::events::{EngineEvent, PlaybackActionPayload};
use rayon::prelude::*;
use rdev::{Button, Event, EventType, Key as RdevKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use winapi::um::libloaderapi::GetModuleHandleW;
#[cfg(windows)]
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, GetForegroundWindow, GetMessageW, GetRawInputData,
    GetWindowTextW, IsWindowVisible, MapVirtualKeyW, RegisterClassW, RegisterRawInputDevices,
    SendInput, CW_USEDEFAULT, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC_EX, MAPVK_VSC_TO_VK_EX,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
    MSG, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RID_INPUT, SM_CXVIRTUALSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, WM_INPUT, WNDCLASSW,
};

/// Call once at startup — polls foreground window every 200ms and stores
/// the last window that is NOT one of our own MacroForge windows.
#[cfg(windows)]
pub fn start_focus_tracker() {
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_millis(200));
            unsafe {
                let hwnd = GetForegroundWindow();
                if hwnd.is_null() {
                    continue;
                }
                // Read window title to exclude MacroForge windows
                let mut buf = [0u16; 256];
                let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
                if len == 0 {
                    continue;
                }
                let title = String::from_utf16_lossy(&buf[..len as usize]);
                if title.contains("MacroForge") {
                    continue;
                }
                if IsWindowVisible(hwnd) == 0 {
                    continue;
                }
                *LAST_GAME_HWND.lock().unwrap() = hwnd as isize;
            }
        }
    });
}

#[cfg(windows)]
pub fn send_mouse_move(x: i32, y: i32) {
    use winapi::um::winuser::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };
    unsafe {
        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN) as f64;
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN) as f64;
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN) as f64;
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN) as f64;

        // Formule de normalisation de précision Windows
        let nx = (((x as f64 - vx) * 65536.0) / vw) as i32;
        let ny = (((y as f64 - vy) * 65536.0) / vh) as i32;

        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: std::mem::zeroed(),
        };
        *input.u.mi_mut() = MOUSEINPUT {
            dx: nx,
            dy: ny,
            mouseData: 0,
            dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | 0x4000,
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(windows)]
pub fn send_mouse_button(button: u8, down: bool, x: i32, y: i32) {
    use winapi::um::winuser::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };
    unsafe {
        let flag = match (button, down) {
            (1, true) => MOUSEEVENTF_LEFTDOWN,
            (1, false) => MOUSEEVENTF_LEFTUP,
            (2, true) => MOUSEEVENTF_RIGHTDOWN,
            (2, false) => MOUSEEVENTF_RIGHTUP,
            (_, true) => MOUSEEVENTF_MIDDLEDOWN,
            (_, false) => MOUSEEVENTF_MIDDLEUP,
        };

        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN) as f64;
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN) as f64;
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN) as f64;
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN) as f64;

        let nx = (((x as f64 - vx) * 65536.0) / vw) as i32;
        let ny = (((y as f64 - vy) * 65536.0) / vh) as i32;

        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: std::mem::zeroed(),
        };
        *input.u.mi_mut() = MOUSEINPUT {
            dx: nx,
            dy: ny,
            mouseData: 0,
            dwFlags: flag | MOUSEEVENTF_ABSOLUTE | 0x4000,
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(windows)]
pub fn send_mouse_relative(dx: i32, dy: i32) {
    use winapi::um::winuser::{SendInput, INPUT, INPUT_MOUSE, MOUSEEVENTF_MOVE, MOUSEINPUT};
    unsafe {
        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: std::mem::zeroed(),
        };
        *input.u.mi_mut() = MOUSEINPUT {
            dx,
            dy,
            mouseData: 0,
            dwFlags: MOUSEEVENTF_MOVE,
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(windows)]
pub fn send_key(vk: u16, key_up: bool, is_extended: bool) {
    unsafe {
        let scan = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) as u16;
        let mut flags = 0;
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        if is_extended {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        };
        *input.u.ki_mut() = KEYBDINPUT {
            wVk: vk,
            wScan: scan,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Capture rapide d'un rectangle de l'écran via GDI (Windows uniquement)
/// Retourne les pixels en format BGRA
#[cfg(windows)]
pub fn capture_screen_gdi(x: i32, y: i32, width: i32, height: i32) -> Option<Vec<u8>> {
    use std::ptr::null_mut;
    use winapi::um::wingdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        SelectObject, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
    };
    use winapi::um::winuser::{GetDC, ReleaseDC};

    unsafe {
        let hdc_screen = GetDC(null_mut());
        if hdc_screen.is_null() {
            return None;
        }

        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        let old_obj = SelectObject(hdc_mem, hbm as *mut _);

        BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, x, y, SRCCOPY);

        let mut bmi = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // Top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        let mut pixels = vec![0u8; (width * height * 4) as usize];
        GetDIBits(
            hdc_mem,
            hbm,
            0,
            height as u32,
            pixels.as_mut_ptr() as *mut _,
            &mut bmi as *mut _ as *mut _,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc_mem, old_obj);
        DeleteObject(hbm as *mut _);
        DeleteDC(hdc_mem);
        ReleaseDC(null_mut(), hdc_screen);

        Some(pixels)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    KeyPress(String, u16, bool), // (nom lisible, virtual key, is_extended)
    KeyRelease(String, u16, bool),
    MouseMove(f64, f64),
    MousePress(u8, f64, f64),
    MouseRelease(u8, f64, f64),
    Scroll(f64, f64),
    MouseMoveRelative(i32, i32),
    WaitImage(String, u64),
    Wait(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroAction {
    pub action_type: ActionType,
    pub delay_ms: u64,
}

pub struct MacroState {
    pub is_recording: bool,
    pub is_playing: bool,
    pub actions: Vec<MacroAction>,
    pub last_event_time: Option<Instant>,
    pub recording_start_time: Option<Instant>,
    pub expected_time_cursor: f64,
    pub stop_playback_flag: Arc<Mutex<bool>>,
    pub last_x: f64,
    pub last_y: f64,
    pub last_move_record_time: Option<Instant>,
    pub is_mouse_down: bool,
    pub is_right_mouse_down: bool,
    pub key_press_times: HashMap<u16, Instant>,
    pub loop_playback: bool,
    pub pending_dx: i32,
    pub pending_dy: i32,
    pub stop_image_path: Option<String>,
    pub stop_image_timeout: u64,
}

impl Default for MacroState {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroState {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            is_playing: false,
            actions: Vec::new(),
            last_event_time: None,
            recording_start_time: None,
            expected_time_cursor: 0.0,
            stop_playback_flag: Arc::new(Mutex::new(false)),
            last_x: 0.0,
            last_y: 0.0,
            last_move_record_time: None,
            is_mouse_down: false,
            is_right_mouse_down: false,
            key_press_times: HashMap::new(),
            loop_playback: false,
            pending_dx: 0,
            pending_dy: 0,
            stop_image_path: None,
            stop_image_timeout: 5000,
        }
    }
}

const EXTREME_IMAGE_DATA: &[u8] = include_bytes!("../extreme.png");
const FAILED_IMAGE_DATA: &[u8] = include_bytes!("../failed.PNG");

lazy_static::lazy_static! {
    pub static ref MACRO_STATE: Mutex<MacroState> = Mutex::new(MacroState::new());
    pub static ref EVENT_SENDER: Mutex<Option<Sender<EngineEvent>>> = Mutex::new(None);
    pub static ref IMAGE_CACHE: Mutex<HashMap<String, Arc<image::RgbaImage>>> = Mutex::new(HashMap::new());
}

#[cfg(windows)]
lazy_static::lazy_static! {
    pub static ref LAST_GAME_HWND: Mutex<isize> = Mutex::new(0);
    pub static ref RAW_INPUT_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[cfg(windows)]
const HID_USAGE_PAGE_GENERIC: u16 = 0x01;
#[cfg(windows)]
const HID_USAGE_GENERIC_MOUSE: u16 = 0x02;

pub fn set_event_sender(sender: Sender<EngineEvent>) {
    *EVENT_SENDER.lock().unwrap() = Some(sender);
}

fn notify_event(event: EngineEvent) {
    if let Some(ref sender) = *EVENT_SENDER.lock().unwrap() {
        let _ = sender.send(event);
    }
}

#[cfg(windows)]
fn spawn_raw_input_listener() {
    let flag = RAW_INPUT_FLAG.clone();
    *flag.lock().unwrap() = true;

    thread::spawn(move || unsafe {
        let h_instance = GetModuleHandleW(std::ptr::null());
        let class_name: Vec<u16> = "RawInputWindow\0".encode_utf16().collect();

        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(DefWindowProcW),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null_mut(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            std::ptr::null(),
            0,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            h_instance,
            std::ptr::null_mut(),
        );

        let rid = RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };

        if RegisterRawInputDevices(&rid, 1, std::mem::size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            println!("ERREUR: Impossible d'enregistrer les Raw Input Devices");
            return;
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, hwnd, 0, 0) != 0 {
            if !*flag.lock().unwrap() {
                break;
            }

            if msg.message == WM_INPUT {
                let mut size: u32 = 0;
                GetRawInputData(
                    msg.lParam as *mut _,
                    RID_INPUT,
                    std::ptr::null_mut(),
                    &mut size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                );

                let mut buffer = vec![0u8; size as usize];
                if GetRawInputData(
                    msg.lParam as *mut _,
                    RID_INPUT,
                    buffer.as_mut_ptr() as *mut _,
                    &mut size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                ) == size
                {
                    let raw = &*(buffer.as_ptr() as *const RAWINPUT);
                    let mouse = raw.data.mouse();
                    let dx = mouse.lLastX;
                    let dy = mouse.lLastY;

                    if dx != 0 || dy != 0 {
                        let mut state = MACRO_STATE.lock().unwrap();
                        if state.is_recording && state.is_right_mouse_down {
                            state.last_x += dx as f64;
                            state.last_y += dy as f64;

                            state.pending_dx += dx;
                            state.pending_dy += dy;

                            let now = Instant::now();
                            let should_record = if let Some(last_move) = state.last_move_record_time
                            {
                                now.duration_since(last_move).as_millis() >= 8
                            } else {
                                true
                            };

                            if should_record {
                                let snap_dx = state.pending_dx;
                                let snap_dy = state.pending_dy;
                                state.pending_dx = 0;
                                state.pending_dy = 0;

                                state.last_move_record_time = Some(now);
                                let delay_ms = if let Some(start) = state.recording_start_time {
                                    let elapsed_f64 =
                                        now.duration_since(start).as_secs_f64() * 1000.0;
                                    let diff = elapsed_f64 - state.expected_time_cursor;
                                    let d = diff.round() as u64;
                                    state.expected_time_cursor += d as f64;
                                    d
                                } else {
                                    0
                                };
                                state.last_event_time = Some(now);

                                state.actions.push(MacroAction {
                                    action_type: ActionType::MouseMoveRelative(snap_dx, snap_dy),
                                    delay_ms,
                                });
                            }
                        } else if state.is_recording {
                            state.pending_dx = 0;
                            state.pending_dy = 0;
                        }
                    }
                }
            }
            winapi::um::winuser::TranslateMessage(&msg);
            winapi::um::winuser::DispatchMessageW(&msg);
        }
    });
}

pub fn emit_recording_state(is_recording: bool) {
    notify_event(EngineEvent::RecordingStateChanged(is_recording));
}

fn emit_playback_action(payload: PlaybackActionPayload) {
    notify_event(EngineEvent::PlaybackAction(payload));
}

#[inline]
fn pixels_match(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, tolerance: u8) -> bool {
    (r1 as i16 - r2 as i16).unsigned_abs() <= tolerance as u16
        && (g1 as i16 - g2 as i16).unsigned_abs() <= tolerance as u16
        && (b1 as i16 - b2 as i16).unsigned_abs() <= tolerance as u16
}

fn rdev_key_to_name_and_scan(key: &RdevKey) -> (String, u16, bool) {
    let name = format!("{:?}", key);
    let mut is_extended = false;
    let vk: u16 = match key {
        RdevKey::Return => 0x0D,
        RdevKey::Space => 0x20,
        RdevKey::Backspace => 0x08,
        RdevKey::Tab => 0x09,
        RdevKey::Escape => 0x1B,
        RdevKey::Delete => {
            is_extended = true;
            0x2E
        }
        RdevKey::Home => {
            is_extended = true;
            0x24
        }
        RdevKey::End => {
            is_extended = true;
            0x23
        }
        RdevKey::PageUp => {
            is_extended = true;
            0x21
        }
        RdevKey::PageDown => {
            is_extended = true;
            0x22
        }
        RdevKey::UpArrow => {
            is_extended = true;
            0x26
        }
        RdevKey::DownArrow => {
            is_extended = true;
            0x28
        }
        RdevKey::LeftArrow => {
            is_extended = true;
            0x25
        }
        RdevKey::RightArrow => {
            is_extended = true;
            0x27
        }
        RdevKey::ShiftLeft => 0xA0,
        RdevKey::ShiftRight => 0xA1,
        RdevKey::ControlLeft => 0xA2,
        RdevKey::ControlRight => {
            is_extended = true;
            0xA3
        }
        RdevKey::Alt => 0x12,
        RdevKey::AltGr => {
            is_extended = true;
            0xA5
        }
        RdevKey::MetaLeft => {
            is_extended = true;
            0x5B
        }
        RdevKey::MetaRight => {
            is_extended = true;
            0x5C
        }
        RdevKey::CapsLock => 0x14,
        RdevKey::F1 => 0x70,
        RdevKey::F2 => 0x71,
        RdevKey::F3 => 0x72,
        RdevKey::F4 => 0x73,
        RdevKey::F5 => 0x74,
        RdevKey::F6 => 0x75,
        RdevKey::F7 => 0x76,
        RdevKey::F11 => 0x7A,
        RdevKey::F12 => 0x7B,
        RdevKey::KeyA => 0x41,
        RdevKey::KeyB => 0x42,
        RdevKey::KeyC => 0x43,
        RdevKey::KeyD => 0x44,
        RdevKey::KeyE => 0x45,
        RdevKey::KeyF => 0x46,
        RdevKey::KeyG => 0x47,
        RdevKey::KeyH => 0x48,
        RdevKey::KeyI => 0x49,
        RdevKey::KeyJ => 0x4A,
        RdevKey::KeyK => 0x4B,
        RdevKey::KeyL => 0x4C,
        RdevKey::KeyM => 0x4D,
        RdevKey::KeyN => 0x4E,
        RdevKey::KeyO => 0x4F,
        RdevKey::KeyP => 0x50,
        RdevKey::KeyQ => 0x51,
        RdevKey::KeyR => 0x52,
        RdevKey::KeyS => 0x53,
        RdevKey::KeyT => 0x54,
        RdevKey::KeyU => 0x55,
        RdevKey::KeyV => 0x56,
        RdevKey::KeyW => 0x57,
        RdevKey::KeyX => 0x58,
        RdevKey::KeyY => 0x59,
        RdevKey::KeyZ => 0x5A,
        RdevKey::Num0 => 0x30,
        RdevKey::Num1 => 0x31,
        RdevKey::Num2 => 0x32,
        RdevKey::Num3 => 0x33,
        RdevKey::Num4 => 0x34,
        RdevKey::Num5 => 0x35,
        RdevKey::Num6 => 0x36,
        RdevKey::Num7 => 0x37,
        RdevKey::Num8 => 0x38,
        RdevKey::Num9 => 0x39,
        RdevKey::Comma => 0xBC,
        RdevKey::Dot => 0xBE,
        RdevKey::Minus => 0xBD,
        RdevKey::Equal => 0xBB,
        RdevKey::SemiColon => 0xBA,
        RdevKey::Quote => 0xDE,
        RdevKey::BackSlash => 0xDC,
        RdevKey::Slash => 0xBF,
        RdevKey::BackQuote => 0xC0,
        RdevKey::Unknown(sc) => {
            #[cfg(windows)]
            unsafe {
                MapVirtualKeyW(*sc, MAPVK_VSC_TO_VK_EX) as u16
            }
            #[cfg(not(windows))]
            {
                *sc as u16
            }
        }
        _ => 0,
    };
    (name, vk, is_extended)
}

pub fn start_recording() {
    println!("Démarrage de l'enregistrement (Raw Input Mode)...");
    {
        let mut state = MACRO_STATE.lock().unwrap();
        state.is_recording = true;
        state.actions.clear();
        state.last_event_time = None;
        state.recording_start_time = Some(Instant::now());
        state.expected_time_cursor = 0.0;
        state.last_move_record_time = None;
        state.key_press_times.clear();
        state.is_mouse_down = false;
        state.is_right_mouse_down = false;
        state.pending_dx = 0;
        state.pending_dy = 0;

        #[cfg(windows)]
        unsafe {
            use winapi::um::winuser::GetCursorPos;
            let mut pt = winapi::shared::windef::POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            state.last_x = pt.x as f64;
            state.last_y = pt.y as f64;
        }
    }

    #[cfg(windows)]
    spawn_raw_input_listener();

    emit_recording_state(true);
}

pub fn stop_recording() -> usize {
    let count = {
        let mut state = MACRO_STATE.lock().unwrap();
        state.is_recording = false;

        if state.pending_dx != 0 || state.pending_dy != 0 {
            let dx = state.pending_dx;
            let dy = state.pending_dy;
            state.pending_dx = 0;
            state.pending_dy = 0;
            state.actions.push(MacroAction {
                action_type: ActionType::MouseMoveRelative(dx, dy),
                delay_ms: 0,
            });
        }

        state.last_event_time = None;
        state.recording_start_time = None;
        state.actions.len()
    };

    #[cfg(windows)]
    {
        let mut flag = RAW_INPUT_FLAG.lock().unwrap();
        *flag = false;
    }

    emit_recording_state(false);
    count
}

pub fn play_macro() {
    let mut state = MACRO_STATE.lock().unwrap();
    if state.is_playing || state.is_recording {
        return;
    }

    if state.actions.is_empty() {
        println!("Lecture annulée : aucune action dans la macro.");
        return;
    }

    state.is_playing = true;
    let actions_to_play = state.actions.clone();
    let stop_flag = Arc::clone(&state.stop_playback_flag);
    *stop_flag.lock().unwrap() = false;

    drop(state);

    notify_event(EngineEvent::PlaybackStateChanged(true));

    thread::spawn(move || {
        let playback_start = Instant::now();
        let ts = || format!("[+{:.2}s]", playback_start.elapsed().as_secs_f64());
        let total_actions = actions_to_play.len();

        println!(
            "{} === PLAYBACK DÉMARRÉ ({} actions) ===",
            ts(),
            total_actions
        );

        let mut iteration = 0u32;
        let stop_image_config: Option<String> = {
            let state = MACRO_STATE.lock().unwrap();
            state.stop_image_path.clone()
        };

        let mut last_stop_check = Instant::now();
        let mut stop_blackout_until: Option<Instant> = None;
        'main_loop: loop {
            iteration += 1;
            println!("{} --- Itération #{} démarrée ---", ts(), iteration);

            let mut action_index = 0usize;
            let mut timeline_origin = Instant::now();
            let mut total_recorded_delay = 0u64;

            for action in &actions_to_play {
                action_index += 1;

                if *stop_flag.lock().unwrap() {
                    println!(
                        "{} [STOP] stop_flag détecté avant action #{} — arrêt.",
                        ts(),
                        action_index
                    );
                    break 'main_loop;
                }

                total_recorded_delay += action.delay_ms;

                if let Some(ref path) = stop_image_config {
                    let now = Instant::now();
                    let in_blackout = stop_blackout_until.map(|t| now < t).unwrap_or(false);

                    if !in_blackout && last_stop_check.elapsed() >= Duration::from_secs(3) {
                        last_stop_check = now;
                        if check_image_present(path) {
                            if MACRO_STATE.lock().unwrap().loop_playback {
                                println!(
                                    "{} [STOP IMAGE] Détectée ! Redémarrage (Blackout 15s activé).",
                                    ts()
                                );
                                stop_blackout_until = Some(now + Duration::from_secs(15));
                                continue 'main_loop;
                            } else {
                                println!("{} [STOP IMAGE] Détectée ! Arrêt définitif.", ts());
                                break 'main_loop;
                            }
                        }
                    }
                }

                let target_time = timeline_origin + Duration::from_millis(total_recorded_delay);
                loop {
                    let now = Instant::now();
                    if now >= target_time {
                        break;
                    }

                    let diff = target_time.duration_since(now).as_millis();
                    if diff > 10 {
                        thread::sleep(Duration::from_millis(1));
                    } else if diff > 1 {
                        thread::yield_now();
                    } else {
                        std::hint::spin_loop();
                    }

                    if *stop_flag.lock().unwrap() {
                        break 'main_loop;
                    }

                    if let Some(ref path) = stop_image_config {
                        let now = Instant::now();
                        let in_blackout = stop_blackout_until.map(|t| now < t).unwrap_or(false);

                        if !in_blackout && last_stop_check.elapsed() >= Duration::from_secs(3) {
                            last_stop_check = now;
                            if check_image_present(path) {
                                if MACRO_STATE.lock().unwrap().loop_playback {
                                    println!("{} [STOP IMAGE] Détectée pendant attente ! Redémarrage (Blackout 15s).", ts());
                                    stop_blackout_until = Some(now + Duration::from_secs(15));
                                    continue 'main_loop;
                                } else {
                                    break 'main_loop;
                                }
                            }
                        }
                    }
                }

                #[cfg(windows)]
                {
                    match action.action_type.clone() {
                        ActionType::KeyPress(ref name, vk, is_ext) => {
                            println!(
                                "{} [#{}/{}] KeyPress '{}' delay={}ms",
                                ts(),
                                action_index,
                                total_actions,
                                name,
                                action.delay_ms
                            );
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "KeyPress".into(),
                                x: 0.0,
                                y: 0.0,
                                detail: format!("{} +{}ms", name, action.delay_ms),
                            });
                            send_key(vk, false, is_ext);
                        }
                        ActionType::KeyRelease(ref name, vk, is_ext) => {
                            println!(
                                "{} [#{}/{}] KeyRelease '{}' delay={}ms",
                                ts(),
                                action_index,
                                actions_to_play.len(),
                                name,
                                action.delay_ms
                            );
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "KeyRelease".into(),
                                x: 0.0,
                                y: 0.0,
                                detail: format!("{} +{}ms", name, action.delay_ms),
                            });
                            send_key(vk, true, is_ext);
                        }
                        ActionType::MouseMoveRelative(dx, dy) => {
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "MoveRel".into(),
                                x: dx as f64,
                                y: dy as f64,
                                detail: format!("Relative {}x{}", dx, dy),
                            });
                            send_mouse_relative(dx, dy);
                        }
                        ActionType::MouseMove(x, y) => {
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "Move".into(),
                                x,
                                y,
                                detail: format!("Pos {}x{}", x, y),
                            });
                            send_mouse_move(x as i32, y as i32);
                        }
                        ActionType::MousePress(u, x, y) => {
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "MousePress".into(),
                                x,
                                y,
                                detail: format!("Button {}", u),
                            });
                            send_mouse_button(u, true, 0, 0);
                        }
                        ActionType::MouseRelease(u, x, y) => {
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "MouseRelease".into(),
                                x,
                                y,
                                detail: format!("Button {}", u),
                            });
                            send_mouse_button(u, false, 0, 0);
                        }
                        ActionType::Scroll(x, y) => {
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "Scroll".into(),
                                x,
                                y,
                                detail: format!("Vector {}x{}", x, y),
                            });
                            unsafe {
                                use winapi::um::winuser::{mouse_event, MOUSEEVENTF_WHEEL};
                                let delta = (y * 120.0) as i32;
                                mouse_event(MOUSEEVENTF_WHEEL, 0, 0, delta as u32, 0);
                            }
                        }
                        ActionType::WaitImage(ref path, timeout) => {
                            println!(
                                "{} [#{}/{}] WaitImage '{}' timeout={}ms",
                                ts(),
                                action_index,
                                actions_to_play.len(),
                                path,
                                timeout
                            );
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "WaitImage".into(),
                                x: 0.0,
                                y: 0.0,
                                detail: "Recherche image...".into(),
                            });

                            let template_arc = {
                                let mut cache = IMAGE_CACHE.lock().unwrap();
                                if let Some(img) = cache.get(path.as_str()) {
                                    img.clone()
                                } else if path == "embedded://extreme.png"
                                    || path == "embedded://failed.PNG"
                                {
                                    let data = if path == "embedded://extreme.png" {
                                        EXTREME_IMAGE_DATA
                                    } else {
                                        FAILED_IMAGE_DATA
                                    };
                                    match image::load_from_memory(data) {
                                        Ok(img) => {
                                            let rb = Arc::new(img.to_rgba8());
                                            cache.insert(path.clone(), rb.clone());
                                            rb
                                        }
                                        Err(e) => {
                                            println!("{} WaitImage: ERREUR chargement image intégrée: {} — action ignorée.", ts(), e);
                                            continue;
                                        }
                                    }
                                } else {
                                    match image::open(path) {
                                        Ok(img) => {
                                            let rb = Arc::new(img.to_rgba8());
                                            cache.insert(path.clone(), rb.clone());
                                            rb
                                        }
                                        Err(e) => {
                                            println!("{} WaitImage: ERREUR ouverture image '{}': {} — action ignorée.", ts(), path, e);
                                            continue;
                                        }
                                    }
                                }
                            };

                            let (tw, th) = template_arc.dimensions();
                            let tw = tw as usize;
                            let th = th as usize;
                            let template_raw = template_arc.as_raw();

                            let mut found = false;
                            let mut _retry_count = 0u32;

                            'wait_outer: loop {
                                _retry_count += 1;
                                let start_wait = Instant::now();

                                'wait: while (start_wait.elapsed().as_millis() as u64) < timeout {
                                    if *stop_flag.lock().unwrap() {
                                        break 'main_loop;
                                    }

                                    #[cfg(windows)]
                                    {
                                        use winapi::um::winuser::GetSystemMetrics;
                                        let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
                                        let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
                                        let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
                                        let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

                                        if let Some(screen_raw) = capture_screen_gdi(vx, vy, vw, vh)
                                        {
                                            let mw_usize = vw as usize;
                                            let mh_usize = vh as usize;

                                            let res = (0..=(mh_usize - th))
                                                .into_par_iter()
                                                .find_map_any(|sy| {
                                                    let monitor_row_start = sy * mw_usize * 4;
                                                    for sx in 0..=(mw_usize - tw) {
                                                        let monitor_pixel_idx =
                                                            monitor_row_start + sx * 4;

                                                        let (sr, sg, sb) = (
                                                            screen_raw[monitor_pixel_idx + 2],
                                                            screen_raw[monitor_pixel_idx + 1],
                                                            screen_raw[monitor_pixel_idx],
                                                        );

                                                        if !pixels_match(
                                                            sr,
                                                            sg,
                                                            sb,
                                                            template_raw[0],
                                                            template_raw[1],
                                                            template_raw[2],
                                                            25,
                                                        ) {
                                                            continue;
                                                        }

                                                        let t_mid_y = th / 2;
                                                        let t_mid_x = tw / 2;
                                                        let s_mid_idx = ((sy + t_mid_y) * mw_usize
                                                            + (sx + t_mid_x))
                                                            * 4;
                                                        let t_mid_idx =
                                                            (t_mid_y * tw + t_mid_x) * 4;
                                                        let (smr, smg, smb) = (
                                                            screen_raw[s_mid_idx + 2],
                                                            screen_raw[s_mid_idx + 1],
                                                            screen_raw[s_mid_idx],
                                                        );
                                                        if !pixels_match(
                                                            smr,
                                                            smg,
                                                            smb,
                                                            template_raw[t_mid_idx],
                                                            template_raw[t_mid_idx + 1],
                                                            template_raw[t_mid_idx + 2],
                                                            25,
                                                        ) {
                                                            continue;
                                                        }

                                                        let t_last_y = th - 1;
                                                        let t_last_x = tw - 1;
                                                        let s_last_idx = ((sy + t_last_y)
                                                            * mw_usize
                                                            + (sx + t_last_x))
                                                            * 4;
                                                        let t_last_idx =
                                                            (t_last_y * tw + t_last_x) * 4;
                                                        let (slr, slg, slb) = (
                                                            screen_raw[s_last_idx + 2],
                                                            screen_raw[s_last_idx + 1],
                                                            screen_raw[s_last_idx],
                                                        );
                                                        if !pixels_match(
                                                            slr,
                                                            slg,
                                                            slb,
                                                            template_raw[t_last_idx],
                                                            template_raw[t_last_idx + 1],
                                                            template_raw[t_last_idx + 2],
                                                            25,
                                                        ) {
                                                            continue;
                                                        }

                                                        let mut matched = true;
                                                        'tmatch: for ty in (0..th).step_by(2) {
                                                            let t_row_start = ty * tw * 4;
                                                            let s_row_start =
                                                                (sy + ty) * mw_usize * 4;
                                                            for tx in (0..tw).step_by(2) {
                                                                let t_idx = t_row_start + tx * 4;
                                                                let s_idx =
                                                                    s_row_start + (sx + tx) * 4;
                                                                let (cur_r, cur_g, cur_b) = (
                                                                    screen_raw[s_idx + 2],
                                                                    screen_raw[s_idx + 1],
                                                                    screen_raw[s_idx],
                                                                );
                                                                if !pixels_match(
                                                                    cur_r,
                                                                    cur_g,
                                                                    cur_b,
                                                                    template_raw[t_idx],
                                                                    template_raw[t_idx + 1],
                                                                    template_raw[t_idx + 2],
                                                                    25,
                                                                ) {
                                                                    matched = false;
                                                                    break 'tmatch;
                                                                }
                                                            }
                                                        }
                                                        if matched {
                                                            return Some((sx, sy));
                                                        }
                                                    }
                                                    None
                                                });

                                            if res.is_some() {
                                                found = true;
                                                break 'wait;
                                            }
                                        }
                                    }

                                    thread::sleep(Duration::from_millis(33));

                                    if let Some(ref path) = stop_image_config {
                                        let now = Instant::now();
                                        let in_blackout =
                                            stop_blackout_until.map(|t| now < t).unwrap_or(false);

                                        if !in_blackout
                                            && last_stop_check.elapsed() >= Duration::from_secs(3)
                                        {
                                            last_stop_check = now;
                                            if check_image_present(path) {
                                                if MACRO_STATE.lock().unwrap().loop_playback {
                                                    stop_blackout_until =
                                                        Some(now + Duration::from_secs(15));
                                                    continue 'main_loop;
                                                } else {
                                                    break 'main_loop;
                                                }
                                            }
                                        }
                                    }
                                }

                                if found {
                                    break 'wait_outer;
                                }

                                let is_looping = MACRO_STATE.lock().unwrap().loop_playback;
                                if is_looping {
                                    continue 'wait_outer;
                                } else {
                                    break 'main_loop;
                                }
                            }

                            if found {
                                timeline_origin = Instant::now();
                                total_recorded_delay = 0;
                            }
                        }
                        ActionType::Wait(ms) => {
                            println!(
                                "{} [#{}/{}] Wait {}ms",
                                ts(),
                                action_index,
                                total_actions,
                                ms
                            );
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "Wait".into(),
                                x: 0.0,
                                y: 0.0,
                                detail: format!("Attente {}ms", ms),
                            });
                            let start_wait = Instant::now();
                            while start_wait.elapsed().as_millis() < ms as u128 {
                                if *stop_flag.lock().unwrap() {
                                    break 'main_loop;
                                }

                                if let Some(ref path) = stop_image_config {
                                    let now = Instant::now();
                                    let in_blackout =
                                        stop_blackout_until.map(|t| now < t).unwrap_or(false);

                                    if !in_blackout
                                        && last_stop_check.elapsed() >= Duration::from_secs(3)
                                    {
                                        last_stop_check = now;
                                        if check_image_present(path) {
                                            if MACRO_STATE.lock().unwrap().loop_playback {
                                                stop_blackout_until =
                                                    Some(now + Duration::from_secs(15));
                                                continue 'main_loop;
                                            } else {
                                                break 'main_loop;
                                            }
                                        }
                                    }
                                }

                                thread::sleep(Duration::from_millis(100));
                            }
                            timeline_origin = Instant::now();
                            total_recorded_delay = 0;
                        }
                    }
                }
            }

            if *stop_flag.lock().unwrap() {
                break 'main_loop;
            }

            let should_loop = MACRO_STATE.lock().unwrap().loop_playback;
            if !should_loop {
                break 'main_loop;
            }

            thread::sleep(Duration::from_millis(250));
        }

        let mut state = MACRO_STATE.lock().unwrap();
        state.is_playing = false;
        println!("{} === PLAYBACK TERMINÉ ===", ts());

        notify_event(EngineEvent::PlaybackStateChanged(false));
    });
}

pub fn get_loop_playback() -> bool {
    let state = MACRO_STATE.lock().unwrap();
    state.loop_playback
}

pub fn set_loop_playback(looping: bool) {
    let mut state = MACRO_STATE.lock().unwrap();
    state.loop_playback = looping;
}

pub fn stop_playback() {
    let state = MACRO_STATE.lock().unwrap();
    *state.stop_playback_flag.lock().unwrap() = true;
}

pub fn get_stop_image() -> (Option<String>, u64) {
    let state = MACRO_STATE.lock().unwrap();
    (state.stop_image_path.clone(), state.stop_image_timeout)
}

pub fn set_stop_image(path: Option<String>, timeout: u64) {
    let mut state = MACRO_STATE.lock().unwrap();
    state.stop_image_path = path;
    state.stop_image_timeout = timeout;
}

pub fn get_actions() -> Vec<MacroAction> {
    let state = MACRO_STATE.lock().unwrap();
    state.actions.clone()
}

pub fn set_actions(actions: Vec<MacroAction>) {
    let mut state = MACRO_STATE.lock().unwrap();
    state.actions = actions;
}

pub fn add_action(action: MacroAction) {
    let mut state = MACRO_STATE.lock().unwrap();
    state.actions.push(action);
}

pub fn insert_action(index: usize, action: MacroAction) {
    let mut state = MACRO_STATE.lock().unwrap();
    let safe_idx = index.min(state.actions.len());
    state.actions.insert(safe_idx, action);
}

pub fn update_action(index: usize, action: MacroAction) -> bool {
    let mut state = MACRO_STATE.lock().unwrap();
    if index < state.actions.len() {
        state.actions[index] = action;
        true
    } else {
        false
    }
}

pub fn delete_action(index: usize) -> Option<MacroAction> {
    let mut state = MACRO_STATE.lock().unwrap();
    if index < state.actions.len() {
        Some(state.actions.remove(index))
    } else {
        None
    }
}

pub fn duplicate_action(index: usize) -> bool {
    let mut state = MACRO_STATE.lock().unwrap();
    if index < state.actions.len() {
        let cloned = state.actions[index].clone();
        state.actions.insert(index + 1, cloned);
        true
    } else {
        false
    }
}

pub fn move_action(from_idx: usize, to_idx: usize) -> bool {
    let mut state = MACRO_STATE.lock().unwrap();
    let len = state.actions.len();
    if from_idx < len && to_idx < len && from_idx != to_idx {
        let item = state.actions.remove(from_idx);
        state.actions.insert(to_idx, item);
        true
    } else {
        false
    }
}

pub fn clear_actions() {
    let mut state = MACRO_STATE.lock().unwrap();
    state.actions.clear();
}

pub fn get_actions_count() -> usize {
    let state = MACRO_STATE.lock().unwrap();
    state.actions.len()
}

pub fn save_macro_to_file(path: &str) -> Result<(), String> {
    let state = MACRO_STATE.lock().unwrap();
    let json = serde_json::to_string_pretty(&state.actions).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_macro_from_file(path: &str) -> Result<usize, String> {
    let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let actions: Vec<MacroAction> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let count = actions.len();
    let mut state = MACRO_STATE.lock().unwrap();
    state.actions = actions;
    Ok(count)
}

fn check_image_present(path: &str) -> bool {
    let template_arc = {
        let mut cache = IMAGE_CACHE.lock().unwrap();
        if let Some(img) = cache.get(path) {
            img.clone()
        } else if path == "embedded://extreme.png" || path == "embedded://failed.PNG" {
            let data = if path == "embedded://extreme.png" {
                EXTREME_IMAGE_DATA
            } else {
                FAILED_IMAGE_DATA
            };
            match image::load_from_memory(data) {
                Ok(img) => {
                    let rb = Arc::new(img.to_rgba8());
                    cache.insert(path.to_string(), rb.clone());
                    rb
                }
                Err(_) => return false,
            }
        } else {
            match image::open(path) {
                Ok(img) => {
                    let rb = Arc::new(img.to_rgba8());
                    cache.insert(path.to_string(), rb.clone());
                    rb
                }
                Err(_) => return false,
            }
        }
    };

    let (tw, th) = template_arc.dimensions();
    let tw = tw as usize;
    let th = th as usize;
    let template_raw = template_arc.as_raw();

    #[cfg(windows)]
    {
        use winapi::um::winuser::GetSystemMetrics;
        let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
        let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
        let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

        if let Some(screen_raw) = capture_screen_gdi(vx, vy, vw, vh) {
            let mw_usize = vw as usize;
            let mh_usize = vh as usize;

            if th > mh_usize || tw > mw_usize {
                return false;
            }

            let res = (0..=(mh_usize - th)).into_par_iter().find_map_any(|sy| {
                let monitor_row_start = sy * mw_usize * 4;
                for sx in 0..=(mw_usize - tw) {
                    let monitor_pixel_idx = monitor_row_start + sx * 4;
                    let (sr, sg, sb) = (
                        screen_raw[monitor_pixel_idx + 2],
                        screen_raw[monitor_pixel_idx + 1],
                        screen_raw[monitor_pixel_idx],
                    );

                    if !pixels_match(
                        sr,
                        sg,
                        sb,
                        template_raw[0],
                        template_raw[1],
                        template_raw[2],
                        25,
                    ) {
                        continue;
                    }

                    let t_mid_y = th / 2;
                    let t_mid_x = tw / 2;
                    let s_mid_idx = ((sy + t_mid_y) * mw_usize + (sx + t_mid_x)) * 4;
                    let t_mid_idx = (t_mid_y * tw + t_mid_x) * 4;
                    let (smr, smg, smb) = (
                        screen_raw[s_mid_idx + 2],
                        screen_raw[s_mid_idx + 1],
                        screen_raw[s_mid_idx],
                    );
                    if !pixels_match(
                        smr,
                        smg,
                        smb,
                        template_raw[t_mid_idx],
                        template_raw[t_mid_idx + 1],
                        template_raw[t_mid_idx + 2],
                        25,
                    ) {
                        continue;
                    }

                    let t_last_y = th - 1;
                    let t_last_x = tw - 1;
                    let s_last_idx = ((sy + t_last_y) * mw_usize + (sx + t_last_x)) * 4;
                    let t_last_idx = (t_last_y * tw + t_last_x) * 4;
                    let (slr, slg, slb) = (
                        screen_raw[s_last_idx + 2],
                        screen_raw[s_last_idx + 1],
                        screen_raw[s_last_idx],
                    );
                    if pixels_match(
                        slr,
                        slg,
                        slb,
                        template_raw[t_last_idx],
                        template_raw[t_last_idx + 1],
                        template_raw[t_last_idx + 2],
                        25,
                    ) {
                        return Some((sx, sy));
                    }
                }
                None
            });
            return res.is_some();
        }
    }
    false
}

pub fn handle_rdev_event(event: Event) {
    if let EventType::KeyPress(key) = &event.event_type {
        match key {
            RdevKey::F8 => {
                let is_rec = {
                    let s = MACRO_STATE.lock().unwrap();
                    s.is_recording
                };
                if !is_rec {
                    start_recording();
                } else {
                    stop_recording();
                }
                return;
            }
            RdevKey::F9 => {
                stop_recording();
                return;
            }
            RdevKey::F4 => {
                stop_playback();

                let was_recording = {
                    let mut s = MACRO_STATE.lock().unwrap();
                    let rec = s.is_recording;
                    s.is_recording = false;
                    rec
                };
                if was_recording {
                    emit_recording_state(false);
                }
                return;
            }
            _ => {}
        }
    }

    let mut state = MACRO_STATE.lock().unwrap();
    if !state.is_recording {
        return;
    }

    let action_type_opt = match &event.event_type {
        EventType::KeyPress(key) => {
            let (name, vk, is_ext) = rdev_key_to_name_and_scan(key);
            if vk == 0 {
                None
            } else if let std::collections::hash_map::Entry::Vacant(e) =
                state.key_press_times.entry(vk)
            {
                e.insert(Instant::now());
                Some(ActionType::KeyPress(name, vk, is_ext))
            } else {
                None
            }
        }
        EventType::KeyRelease(key) => {
            let (name, vk, is_ext) = rdev_key_to_name_and_scan(key);
            if vk == 0 {
                None
            } else {
                Some(ActionType::KeyRelease(name, vk, is_ext))
            }
        }
        EventType::MouseMove { x, y } => {
            state.last_x = *x;
            state.last_y = *y;

            #[cfg(windows)]
            {
                if !state.is_right_mouse_down {
                    let now = Instant::now();
                    let should_record = if let Some(last_move) = state.last_move_record_time {
                        now.duration_since(last_move).as_millis() >= 16
                    } else {
                        true
                    };

                    if should_record {
                        state.last_move_record_time = Some(now);
                        Some(ActionType::MouseMove(*x, *y))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            #[cfg(not(windows))]
            {
                let now = Instant::now();
                let should_record = if let Some(last_move) = state.last_move_record_time {
                    now.duration_since(last_move).as_millis() >= 16
                } else {
                    true
                };

                if should_record {
                    state.last_move_record_time = Some(now);
                    Some(ActionType::MouseMove(*x, *y))
                } else {
                    None
                }
            }
        }
        EventType::ButtonPress(b) => {
            state.is_mouse_down = true;
            if let Button::Right = b {
                state.is_right_mouse_down = true;
            }
            let u = match b {
                Button::Left => 1,
                Button::Right => 2,
                Button::Middle => 3,
                _ => 4,
            };
            Some(ActionType::MousePress(u, state.last_x, state.last_y))
        }
        EventType::ButtonRelease(b) => {
            state.is_mouse_down = false;
            if let Button::Right = b {
                state.is_right_mouse_down = false;
            }
            let u = match b {
                Button::Left => 1,
                Button::Right => 2,
                Button::Middle => 3,
                _ => 4,
            };
            Some(ActionType::MouseRelease(u, state.last_x, state.last_y))
        }
        EventType::Wheel { delta_x, delta_y } => {
            Some(ActionType::Scroll(*delta_x as f64, *delta_y as f64))
        }
    };

    if let Some(action_type) = action_type_opt {
        let now = Instant::now();
        let delay_ms = if let Some(start) = state.recording_start_time {
            let elapsed_f64 = now.duration_since(start).as_secs_f64() * 1000.0;
            let diff = elapsed_f64 - state.expected_time_cursor;
            let d = diff.round() as u64;
            state.expected_time_cursor += d as f64;
            d
        } else {
            0
        };

        if let Some(last_action) = state.actions.last() {
            let is_duplicate = match (&last_action.action_type, &action_type) {
                (ActionType::KeyPress(_, vk1, _), ActionType::KeyPress(_, vk2, _)) => vk1 == vk2,
                (ActionType::KeyRelease(_, vk1, _), ActionType::KeyRelease(_, vk2, _)) => {
                    vk1 == vk2
                }
                (ActionType::MousePress(b1, _, _), ActionType::MousePress(b2, _, _)) => b1 == b2,
                (ActionType::MouseRelease(b1, _, _), ActionType::MouseRelease(b2, _, _)) => {
                    b1 == b2
                }
                _ => false,
            };

            if is_duplicate && delay_ms < 5 {
                return;
            }
        }

        if let ActionType::KeyRelease(_, vk, _) = &action_type {
            state.key_press_times.remove(vk);
        }

        state.last_event_time = Some(now);
        state.actions.push(MacroAction {
            action_type,
            delay_ms,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_action_manipulations() {
        clear_actions();
        assert_eq!(get_actions_count(), 0);

        let a1 = MacroAction {
            action_type: ActionType::KeyPress("A".into(), 0x41, false),
            delay_ms: 10,
        };
        let a2 = MacroAction {
            action_type: ActionType::KeyRelease("A".into(), 0x41, false),
            delay_ms: 20,
        };
        let a3 = MacroAction {
            action_type: ActionType::Wait(500),
            delay_ms: 0,
        };

        add_action(a1.clone());
        add_action(a2.clone());
        assert_eq!(get_actions_count(), 2);

        // Insertion
        insert_action(1, a3.clone());
        assert_eq!(get_actions_count(), 3);
        let actions = get_actions();
        assert_eq!(actions[1].action_type, ActionType::Wait(500));

        // Update
        let updated = MacroAction {
            action_type: ActionType::Wait(1000),
            delay_ms: 50,
        };
        assert!(update_action(1, updated));
        assert_eq!(get_actions()[1].action_type, ActionType::Wait(1000));

        // Duplicate
        assert!(duplicate_action(1));
        assert_eq!(get_actions_count(), 4);
        assert_eq!(get_actions()[2].action_type, ActionType::Wait(1000));

        // Move
        assert!(move_action(0, 2));
        assert_eq!(
            get_actions()[2].action_type,
            ActionType::KeyPress("A".into(), 0x41, false)
        );

        // Delete
        let deleted = delete_action(2);
        assert!(deleted.is_some());
        assert_eq!(get_actions_count(), 3);

        clear_actions();
        assert_eq!(get_actions_count(), 0);
    }
}
