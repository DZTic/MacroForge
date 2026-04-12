use rdev::{Button, Event, EventType, Key as RdevKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use rayon::prelude::*;

#[cfg(windows)]
use winapi::um::winuser::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, GetForegroundWindow, GetMessageW,
    GetRawInputData, MSG, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RegisterClassW,
    RegisterRawInputDevices, RIDEV_INPUTSINK, RID_INPUT, SendInput, WM_INPUT,
    WNDCLASSW, INPUT, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT, SM_CXVIRTUALSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    GetWindowTextW, IsWindowVisible
};
#[cfg(windows)]
use winapi::um::libloaderapi::GetModuleHandleW;
#[cfg(windows)]




/// Call once at startup — polls foreground window every 200ms and stores
/// the last window that is NOT one of our own Tauri windows.
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
fn send_mouse_move(x: i32, y: i32) {
    use winapi::um::winuser::{GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN};
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
fn send_mouse_button(button: u8, down: bool, x: i32, y: i32) {
    use winapi::um::winuser::{GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN};
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
            // On inclut MOVE et ABSOLUTE même pour le clic pour verrouiller la position
            dwFlags: flag | MOUSEEVENTF_ABSOLUTE | 0x4000, 
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(windows)]
fn send_mouse_relative(dx: i32, dy: i32) {
    use winapi::um::winuser::{INPUT, INPUT_MOUSE, MOUSEINPUT, MOUSEEVENTF_MOVE, SendInput};
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
fn send_key(vk: u16, key_up: bool, is_extended: bool) {
    #[cfg(windows)]
    unsafe {
        use winapi::um::winuser::{MapVirtualKeyW, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC_EX, INPUT, INPUT_KEYBOARD, KEYBDINPUT, SendInput};
        let scan = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) as u16;
        let mut flags = 0;
        if key_up { flags |= KEYEVENTF_KEYUP; }
        if is_extended { flags |= KEYEVENTF_EXTENDEDKEY; }
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
fn capture_screen_gdi(x: i32, y: i32, width: i32, height: i32) -> Option<Vec<u8>> {
    use winapi::um::winuser::{GetDC, ReleaseDC};
    use winapi::um::wingdi::{CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, GetDIBits, DeleteObject, DeleteDC, SRCCOPY, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
    use std::ptr::null_mut;

    unsafe {
        let hdc_screen = GetDC(null_mut());
        if hdc_screen.is_null() { return None; }
        
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        let old_obj = SelectObject(hdc_mem, hbm as *mut _);

        // Capture ultra-rapide
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
        GetDIBits(hdc_mem, hbm, 0, height as u32, pixels.as_mut_ptr() as *mut _, &mut bmi as *mut _ as *mut _, DIB_RGB_COLORS);

        // Nettoyage
        SelectObject(hdc_mem, old_obj);
        DeleteObject(hbm as *mut _);
        DeleteDC(hdc_mem);
        ReleaseDC(null_mut(), hdc_screen);

        Some(pixels)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
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


pub fn set_app_handle(handle: AppHandle) {
    *APP_HANDLE.lock().unwrap() = Some(handle);
}

#[cfg(windows)]
fn spawn_raw_input_listener() {
    let flag = RAW_INPUT_FLAG.clone();
    *flag.lock().unwrap() = true;

    thread::spawn(move || {
        unsafe {
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
                    ) == size {
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
                                
                                // On enregistre les mouvements par blocs pour ne pas saturer la liste d'actions
                                // Mais on garde une fréquence élevée (env. 125Hz) pour la fluidité
                                let now = Instant::now();
                                let should_record = if let Some(last_move) = state.last_move_record_time {
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
                                        let elapsed_f64 = now.duration_since(start).as_secs_f64() * 1000.0;
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
                                // Si le bouton droit n'est pas enfoncé, on vide quand même les deltas accumulés
                                // pour éviter un saut brusque au prochain clic droit
                                state.pending_dx = 0;
                                state.pending_dy = 0;
                            }
                        }
                    }
                }
                winapi::um::winuser::TranslateMessage(&msg);
                winapi::um::winuser::DispatchMessageW(&msg);
            }
        }
    });
}

pub fn emit_recording_state(is_recording: bool) {
    if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
        let _ = handle.emit("recording-state-changed", is_recording);
    }
}

/// Payload envoyé à l'overlay pour chaque action exécutée
#[derive(Clone, Serialize)]
pub struct PlaybackActionPayload {
    pub index: usize,
    pub total: usize,
    pub action_type: String,
    pub x: f64,
    pub y: f64,
    pub detail: String,
}

/// Émet un event vers l'overlay de debug
fn emit_playback_action(payload: PlaybackActionPayload) {
    if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
        let _ = handle.emit("playback-action", payload);
    }
}

/// Affiche ou masque la fenêtre overlay.
fn set_overlay_visible(visible: bool) {
    if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
        if let Some(win) = handle.get_webview_window("overlay") {
            if visible {
                let _ = win.set_ignore_cursor_events(true); // CRUCIAL : l'overlay ne bloque plus les clics
                let _ = win.show();
                let _ = win.set_always_on_top(true);

                // Exclure la fenêtre de toutes les captures d'écran GDI
                #[cfg(windows)]
                {
                    use winapi::um::winuser::SetWindowDisplayAffinity;
                    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
                    if let Ok(hwnd) = win.hwnd() {
                        unsafe {
                            SetWindowDisplayAffinity(
                                hwnd.0 as winapi::shared::windef::HWND,
                                WDA_EXCLUDEFROMCAPTURE,
                            );
                        }
                    }
                }
            } else {
                let _ = win.hide();
            }
        }
    }
}

/// Returns true if two RGB pixels are within `tolerance` on each channel
#[inline]
fn pixels_match(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, tolerance: u8) -> bool {
    (r1 as i16 - r2 as i16).unsigned_abs() <= tolerance as u16
        && (g1 as i16 - g2 as i16).unsigned_abs() <= tolerance as u16
        && (b1 as i16 - b2 as i16).unsigned_abs() <= tolerance as u16
}

/// Convertit une rdev::Key en (nom lisible, Virtual Key Windows, is_extended)
fn rdev_key_to_name_and_scan(key: &RdevKey) -> (String, u16, bool) {
    let name = format!("{:?}", key);
    let mut is_extended = false;
    // On utilise les Virtual Codes (VK) de Windows pour être indépendant de la disposition
    let vk: u16 = match key {
        RdevKey::Return => 0x0D,
        RdevKey::Space => 0x20,
        RdevKey::Backspace => 0x08,
        RdevKey::Tab => 0x09,
        RdevKey::Escape => 0x1B,
        RdevKey::Delete => { is_extended = true; 0x2E },
        RdevKey::Home => { is_extended = true; 0x24 },
        RdevKey::End => { is_extended = true; 0x23 },
        RdevKey::PageUp => { is_extended = true; 0x21 },
        RdevKey::PageDown => { is_extended = true; 0x22 },
        RdevKey::UpArrow => { is_extended = true; 0x26 },
        RdevKey::DownArrow => { is_extended = true; 0x28 },
        RdevKey::LeftArrow => { is_extended = true; 0x25 },
        RdevKey::RightArrow => { is_extended = true; 0x27 },
        RdevKey::ShiftLeft => 0xA0,
        RdevKey::ShiftRight => 0xA1,
        RdevKey::ControlLeft => 0xA2,
        RdevKey::ControlRight => { is_extended = true; 0xA3 },
        RdevKey::Alt => 0x12,
        RdevKey::AltGr => { is_extended = true; 0xA5 },
        RdevKey::MetaLeft => { is_extended = true; 0x5B },
        RdevKey::MetaRight => { is_extended = true; 0x5C },
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
        // Les ponctuations peuvent varier, mais les VK Windows sont assez stables
        RdevKey::Comma => 0xBC,
        RdevKey::Dot => 0xBE,
        RdevKey::Minus => 0xBD,
        RdevKey::Equal => 0xBB,
        RdevKey::SemiColon => 0xBA,
        RdevKey::Quote => 0xDE,
        RdevKey::BackSlash => 0xDC,
        RdevKey::Slash => 0xBF,
        RdevKey::BackQuote => 0xC0,
        // Pour les autres, on essaye de mapper le code brut d'rdev vers un VK
        RdevKey::Unknown(sc) => {
            #[cfg(windows)]
            unsafe {
                use winapi::um::winuser::{MapVirtualKeyW, MAPVK_VSC_TO_VK_EX};
                MapVirtualKeyW(*sc as u32, MAPVK_VSC_TO_VK_EX) as u16
            }
            #[cfg(not(windows))]
            { *sc as u16 }
        },
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
        
        // Initialise la position de départ à la position actuelle du curseur
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
        
        // Flush des derniers deltas
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

    // Sécurité : ne pas lancer si aucune action n'est chargée
    if state.actions.is_empty() {
        println!("Lecture annulée : aucune action dans la macro.");
        return;
    }

    state.is_playing = true;
    let actions_to_play = state.actions.clone();
    let stop_flag = Arc::clone(&state.stop_playback_flag);
    *stop_flag.lock().unwrap() = false;

    drop(state);

    if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
        let _ = handle.emit("playback-state-changed", true);
    }

    thread::spawn(move || {
        let playback_start = Instant::now();
        let ts = || format!("[+{:.2}s]", playback_start.elapsed().as_secs_f64());
        let total_actions = actions_to_play.len();

        println!("{} === PLAYBACK DÉMARRÉ ({} actions) ===", ts(), total_actions);
        set_overlay_visible(true);

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

            // Réinitialisé à chaque itération pour éviter toute pollution inter-boucle
            let mut action_index = 0usize;

            // === NOUVEAU SYSTÈME DE TIMING ABSOLU ===
            // Pour éviter la dérive temporelle, on ne dort pas séquentiellement.
            // On calcule le moment précis où chaque action DOIT se produire depuis le début.
            let mut timeline_origin = Instant::now();
            let mut total_recorded_delay = 0u64;

            for action in &actions_to_play {
                action_index += 1;

                if *stop_flag.lock().unwrap() {
                    println!("{} [STOP] stop_flag détecté avant action #{} — arrêt.", ts(), action_index);
                    break 'main_loop;
                }
                
                // Crédit de temps enregistré pour cette action
                total_recorded_delay += action.delay_ms;

                // --- VÉRIFICATION DE L'IMAGE D'ARRÊT (TOUTES LES 3 SECONDES + BLACKOUT) ---
                if let Some(ref path) = stop_image_config {
                    let now = Instant::now();
                    let in_blackout = stop_blackout_until.map(|t| now < t).unwrap_or(false);

                    if !in_blackout && last_stop_check.elapsed() >= Duration::from_secs(3) {
                        last_stop_check = now;
                        if check_image_present(path) {
                            if MACRO_STATE.lock().unwrap().loop_playback {
                                println!("{} [STOP IMAGE] Détectée ! Redémarrage (Blackout 15s activé).", ts());
                                stop_blackout_until = Some(now + Duration::from_secs(15));
                                continue 'main_loop;
                            } else {
                                println!("{} [STOP IMAGE] Détectée ! Arrêt définitif.", ts());
                                break 'main_loop;
                            }
                        }
                    }
                }

                // Attente de haute précision jusqu'au moment cible
                let target_time = timeline_origin + Duration::from_millis(total_recorded_delay);
                loop {
                    let now = Instant::now();
                    if now >= target_time { break; }
                    
                    let diff = target_time.duration_since(now).as_millis();
                    if diff > 10 {
                        thread::sleep(Duration::from_millis(1));
                    } else if diff > 1 {
                        thread::yield_now();
                    } else {
                        std::hint::spin_loop();
                    }
                    
                    if *stop_flag.lock().unwrap() { break 'main_loop; }

                    // --- VÉRIFICATION DE L'IMAGE D'ARRÊT (DANS LA BOUCLE D'ATTENTE) ---
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
                            println!("{} [#{}/{}] KeyPress '{}' delay={}ms", ts(), action_index, total_actions, name, action.delay_ms);
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "KeyPress".into(),
                                x: 0.0, y: 0.0,
                                detail: format!("{} +{}ms", name, action.delay_ms),
                            });
                            send_key(vk, false, is_ext);
                        }
                        ActionType::KeyRelease(ref name, vk, is_ext) => {
                            println!("{} [#{}/{}] KeyRelease '{}' delay={}ms", ts(), action_index, actions_to_play.len(), name, action.delay_ms);
                            send_key(vk, true, is_ext);
                        }
                        ActionType::MouseMoveRelative(dx, dy) => {
                            send_mouse_relative(dx, dy);
                        }
                        ActionType::MouseMove(x, y) => {
                            send_mouse_move(x as i32, y as i32);
                        }
                        ActionType::MousePress(u, _x, _y) => {
                            // On ne force plus le mouvement absolu ici pour éviter les "sauts" de caméra en FPS
                            send_mouse_button(u, true, 0, 0);
                        }
                        ActionType::MouseRelease(u, _x, _y) => {
                            send_mouse_button(u, false, 0, 0);
                        }
                        ActionType::Scroll(_x, y) => {
                            unsafe {
                                use winapi::um::winuser::{mouse_event, MOUSEEVENTF_WHEEL};
                                let delta = (y * 120.0) as i32;
                                mouse_event(MOUSEEVENTF_WHEEL, 0, 0, delta as u32, 0);
                            }
                        }
                        ActionType::WaitImage(ref path, timeout) => {
                            println!("{} [#{}/{}] WaitImage '{}' timeout={}ms", ts(), action_index, actions_to_play.len(), path, timeout);
                            emit_playback_action(PlaybackActionPayload {
                                index: action_index,
                                total: total_actions,
                                action_type: "WaitImage".into(),
                                x: 0.0, y: 0.0,
                                detail: "Recherche image...".into(),
                            });


                            // Chargement depuis le cache ou le disque
                            let template_arc = {
                                let mut cache = IMAGE_CACHE.lock().unwrap();
                                if let Some(img) = cache.get(path.as_str()) {
                                    println!("{} WaitImage: image chargée depuis le cache.", ts());
                                    img.clone()
                                } else if path == "embedded://extreme.png" || path == "embedded://failed.PNG" {
                                    println!("{} WaitImage: chargement de l'image intégrée {}", ts(), path);
                                    let data = if path == "embedded://extreme.png" { EXTREME_IMAGE_DATA } else { FAILED_IMAGE_DATA };
                                    match image::load_from_memory(data) {
                                        Ok(img) => {
                                            let rb = Arc::new(img.to_rgba8());
                                            cache.insert(path.clone(), rb.clone());
                                            println!("{} WaitImage: image intégrée chargée et mise en cache.", ts());
                                            rb
                                        }
                                        Err(e) => {
                                            println!("{} WaitImage: ERREUR chargement image intégrée: {} — action ignorée.", ts(), e);
                                            continue;
                                        }
                                    }
                                } else {
                                    println!("{} WaitImage: chargement depuis le disque...", ts());
                                    match image::open(&path) {
                                        Ok(img) => {
                                            let rb = Arc::new(img.to_rgba8());
                                            cache.insert(path.clone(), rb.clone());
                                            println!("{} WaitImage: image chargée et mise en cache.", ts());
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
                            println!("{} WaitImage: taille template {}x{}", ts(), tw, th);

                            let mut found = false;
                            let mut retry_count = 0u32;

                            // En mode loop : on répète la recherche jusqu'à trouver l'image
                            // En mode no-loop : on s'arrête après le timeout configuré
                            'wait_outer: loop {
                                retry_count += 1;
                                if retry_count > 1 {
                                    println!("{} WaitImage: tentative #{} ...", ts(), retry_count);
                                }
                                let start_wait = Instant::now();

                                'wait: while (start_wait.elapsed().as_millis() as u64) < timeout {
                                    if *stop_flag.lock().unwrap() {
                                        println!("{} WaitImage: stop_flag détecté pendant la recherche — arrêt.", ts());
                                        break 'main_loop;
                                    }

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

                                            let res = (0..=(mh_usize - th)).into_par_iter().find_map_any(|sy| {
                                                let monitor_row_start = sy * mw_usize * 4;
                                                for sx in 0..=(mw_usize - tw) {
                                                    let monitor_pixel_idx = monitor_row_start + sx * 4;

                                                    let (sr, sg, sb) = (screen_raw[monitor_pixel_idx + 2], screen_raw[monitor_pixel_idx + 1], screen_raw[monitor_pixel_idx + 0]);

                                                    // Test 1: Haut-Gauche
                                                    if !pixels_match(sr, sg, sb, template_raw[0], template_raw[1], template_raw[2], 25) {
                                                        continue;
                                                    }

                                                    // Test 2: Centre
                                                    let t_mid_y = th / 2;
                                                    let t_mid_x = tw / 2;
                                                    let s_mid_idx = ((sy + t_mid_y) * mw_usize + (sx + t_mid_x)) * 4;
                                                    let t_mid_idx = (t_mid_y * tw + t_mid_x) * 4;
                                                    let (smr, smg, smb) = (screen_raw[s_mid_idx + 2], screen_raw[s_mid_idx + 1], screen_raw[s_mid_idx + 0]);
                                                    if !pixels_match(smr, smg, smb, template_raw[t_mid_idx], template_raw[t_mid_idx + 1], template_raw[t_mid_idx + 2], 25) {
                                                        continue;
                                                    }

                                                    // Test 3: Bas-Droite
                                                    let t_last_y = th - 1;
                                                    let t_last_x = tw - 1;
                                                    let s_last_idx = ((sy + t_last_y) * mw_usize + (sx + t_last_x)) * 4;
                                                    let t_last_idx = (t_last_y * tw + t_last_x) * 4;
                                                    let (slr, slg, slb) = (screen_raw[s_last_idx + 2], screen_raw[s_last_idx + 1], screen_raw[s_last_idx + 0]);
                                                    if !pixels_match(slr, slg, slb, template_raw[t_last_idx], template_raw[t_last_idx + 1], template_raw[t_last_idx + 2], 25) {
                                                        continue;
                                                    }

                                                    // Échantillonnage final
                                                    let mut matched = true;
                                                    'tmatch: for ty in (0..th).step_by(2) {
                                                        let t_row_start = ty * tw * 4;
                                                        let s_row_start = (sy + ty) * mw_usize * 4;
                                                        for tx in (0..tw).step_by(2) {
                                                            let t_idx = t_row_start + tx * 4;
                                                            let s_idx = s_row_start + (sx + tx) * 4;
                                                            let (cur_r, cur_g, cur_b) = (screen_raw[s_idx + 2], screen_raw[s_idx + 1], screen_raw[s_idx + 0]);
                                                            if !pixels_match(cur_r, cur_g, cur_b, template_raw[t_idx], template_raw[t_idx + 1], template_raw[t_idx + 2], 25) {
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

                                    // --- VÉRIFICATION DE L'IMAGE D'ARRÊT (DANS WAITIMAGE) ---
                                    if let Some(ref path) = stop_image_config {
                                        let now = Instant::now();
                                        let in_blackout = stop_blackout_until.map(|t| now < t).unwrap_or(false);

                                        if !in_blackout && last_stop_check.elapsed() >= Duration::from_secs(3) {
                                            last_stop_check = now;
                                            if check_image_present(path) {
                                                if MACRO_STATE.lock().unwrap().loop_playback {
                                                    println!("{} [STOP IMAGE] Détectée dans WaitImage ! Redémarrage (Blackout 15s).", ts());
                                                    stop_blackout_until = Some(now + Duration::from_secs(15));
                                                    continue 'main_loop;
                                                } else {
                                                    break 'main_loop;
                                                }
                                            }
                                        }
                                    }
                                }

                                if found {
                                    println!("{} WaitImage: IMAGE TROUVÉE après tentative #{}", ts(), retry_count);
                                    break 'wait_outer;
                                }

                                // Timeout atteint sans trouver l'image
                                let is_looping = MACRO_STATE.lock().unwrap().loop_playback;
                                println!("{} WaitImage: timeout atteint (tentative #{}) — is_looping={}", ts(), retry_count, is_looping);
                                if is_looping {
                                    // En mode loop : on relance la recherche (le jeu a peut-être besoin
                                    // de plus de temps pour revenir à l'état initial)
                                    println!("{} WaitImage: relance de la recherche...", ts());
                                    continue 'wait_outer;
                                } else {
                                    // En mode no-loop : arrêt définitif
                                    println!("{} WaitImage: arrêt définitif (mode no-loop).", ts());
                                    break 'main_loop;
                                }
                            }

                            if found {
                                // RE-SYNCHRONISATION : L'horloge repart à zéro
                                // pour que les actions suivantes respectent leur délai relatif
                                // à partir du moment où l'image a été vue.
                                timeline_origin = Instant::now();
                                total_recorded_delay = 0;
                            }
                        }
                        ActionType::Wait(ms) => {
                            println!("{} [#{}/{}] Wait {}ms", ts(), action_index, total_actions, ms);
                            let start_wait = Instant::now();
                            while start_wait.elapsed().as_millis() < ms as u128 {
                                if *stop_flag.lock().unwrap() { break 'main_loop; }
                                
                                // --- VÉRIFICATION DE L'IMAGE D'ARRÊT (DANS WAIT) ---
                                if let Some(ref path) = stop_image_config {
                                    let now = Instant::now();
                                    let in_blackout = stop_blackout_until.map(|t| now < t).unwrap_or(false);

                                    if !in_blackout && last_stop_check.elapsed() >= Duration::from_secs(3) {
                                        last_stop_check = now;
                                        if check_image_present(path) {
                                            if MACRO_STATE.lock().unwrap().loop_playback {
                                                println!("{} [STOP IMAGE] Détectée dans Wait ! Redémarrage (Blackout 15s).", ts());
                                                stop_blackout_until = Some(now + Duration::from_secs(15));
                                                continue 'main_loop;
                                            } else {
                                                break 'main_loop;
                                            }
                                        }
                                    }
                                }
                                
                                thread::sleep(Duration::from_millis(100));
                            }
                            // RE-SYNCHRONISATION après une pause manuelle
                            timeline_origin = Instant::now();
                            total_recorded_delay = 0;
                        }
                    }
                }
            } // fin du for action

            println!("{} --- Itération #{} terminée ({} actions exécutées) ---", ts(), iteration, action_index);

            // Vérification stop_flag en fin d'itération
            if *stop_flag.lock().unwrap() {
                println!("{} [STOP] stop_flag détecté en fin d'itération #{} — arrêt.", ts(), iteration);
                break 'main_loop;
            }

            // Vérifier si on doit boucler ou s'arrêter
            let should_loop = MACRO_STATE.lock().unwrap().loop_playback;
            println!("{} Fin itération #{} — loop_playback={}", ts(), iteration, should_loop);

            if !should_loop {
                println!("{} Mode no-loop : fin de la macro.", ts());
                break 'main_loop;
            }

            // Délai de stabilisation entre deux itérations en mode loop
            println!("{} Mode loop : pause de stabilisation 250ms...", ts());
            thread::sleep(Duration::from_millis(250));
            println!("{} Mode loop : redémarrage de l'itération #{}...", ts(), iteration + 1);
        } // fin de 'main_loop

        let mut state = MACRO_STATE.lock().unwrap();
        state.is_playing = false;
        println!("{} === PLAYBACK TERMINÉ (total {} itérations) ===", ts(), iteration);
        set_overlay_visible(false);

        if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
            let _ = handle.emit("playback-state-changed", false);
        }
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

/// Helper function to check if an image is currently on screen
fn check_image_present(path: &str) -> bool {
    // Basic reuse of the existing detection logic, but returning bool
    let template_arc = {
        let mut cache = IMAGE_CACHE.lock().unwrap();
        if let Some(img) = cache.get(path) {
            img.clone()
        } else if path == "embedded://extreme.png" || path == "embedded://failed.PNG" {
            let data = if path == "embedded://extreme.png" { EXTREME_IMAGE_DATA } else { FAILED_IMAGE_DATA };
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

            if th > mh_usize || tw > mw_usize { return false; }

            let res = (0..=(mh_usize - th)).into_par_iter().find_map_any(|sy| {
                let monitor_row_start = sy * mw_usize * 4;
                for sx in 0..=(mw_usize - tw) {
                    let monitor_pixel_idx = monitor_row_start + sx * 4;
                    let (sr, sg, sb) = (screen_raw[monitor_pixel_idx + 2], screen_raw[monitor_pixel_idx + 1], screen_raw[monitor_pixel_idx + 0]);

                    // Point 1 : Coin Haut-Gauche
                    if !pixels_match(sr, sg, sb, template_raw[0], template_raw[1], template_raw[2], 25) {
                        continue;
                    }

                    // Point 2 : Centre
                    let t_mid_y = th / 2;
                    let t_mid_x = tw / 2;
                    let s_mid_idx = ((sy + t_mid_y) * mw_usize + (sx + t_mid_x)) * 4;
                    let t_mid_idx = (t_mid_y * tw + t_mid_x) * 4;
                    let (smr, smg, smb) = (screen_raw[s_mid_idx + 2], screen_raw[s_mid_idx + 1], screen_raw[s_mid_idx + 0]);
                    if !pixels_match(smr, smg, smb, template_raw[t_mid_idx], template_raw[t_mid_idx + 1], template_raw[t_mid_idx + 2], 25) {
                        continue;
                    }

                    // Point 3 : Coin Bas-Droite
                    let t_last_y = th - 1;
                    let t_last_x = tw - 1;
                    let s_last_idx = ((sy + t_last_y) * mw_usize + (sx + t_last_x)) * 4;
                    let t_last_idx = (t_last_y * tw + t_last_x) * 4;
                    let (slr, slg, slb) = (screen_raw[s_last_idx + 2], screen_raw[s_last_idx + 1], screen_raw[s_last_idx + 0]);
                    if pixels_match(slr, slg, slb, template_raw[t_last_idx], template_raw[t_last_idx + 1], template_raw[t_last_idx + 2], 25) {
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
    // Intercept F8 = start record, F9 = stop record, F4 = stop playback
    if let EventType::KeyPress(key) = &event.event_type {
        match key {
            RdevKey::F8 => {
                start_recording();
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
            } else {
                // Si la touche est déjà notée comme pressée, on ignore la répétition (auto-repeat)
                if state.key_press_times.contains_key(&vk) {
                    None
                } else {
                    state.key_press_times.insert(vk, Instant::now());
                    Some(ActionType::KeyPress(name, vk, is_ext))
                }
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
            #[cfg(not(windows))]
            {
                state.last_x = *x;
                state.last_y = *y;
                
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
            #[cfg(windows)]
            {
                // Sur Windows, on utilise le mouvement absolu rdev seulement si RMB est relâché.
                // Si RMB est enfoncé, on laisse le Raw Input gérer le (relatif).
                state.last_x = *x;
                state.last_y = *y;

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

        // FILTRE ANTI-DOUBLON (Debounce)
        // Certains pilotes ou systèmes envoient deux fois le même événement quasi-instantanément.
        if let Some(last_action) = state.actions.last() {
            let is_duplicate = match (&last_action.action_type, &action_type) {
                (ActionType::KeyPress(_, vk1, _), ActionType::KeyPress(_, vk2, _)) => vk1 == vk2,
                (ActionType::KeyRelease(_, vk1, _), ActionType::KeyRelease(_, vk2, _)) => vk1 == vk2,
                (ActionType::MousePress(b1, _, _), ActionType::MousePress(b2, _, _)) => b1 == b2,
                (ActionType::MouseRelease(b1, _, _), ActionType::MouseRelease(b2, _, _)) => b1 == b2,
                _ => false,
            };

            // Si c'est exactement la même action en moins de 5ms, on l'ignore.
            if is_duplicate && delay_ms < 5 {
                return;
            }
        }

        // Mise à jour du set des touches pressées lors du relâchement
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


