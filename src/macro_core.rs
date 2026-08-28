use crate::events::{EngineEvent, PlaybackActionPayload};
use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use rdev::{Button, Event, EventType, Key as RdevKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use winapi::um::libloaderapi::GetModuleHandleW;
#[cfg(windows)]
use winapi::um::timeapi::{timeBeginPeriod, timeEndPeriod};
#[cfg(windows)]
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, GetForegroundWindow, GetMessageW, GetRawInputData,
    GetWindowTextW, IsWindowVisible, MapVirtualKeyW, RegisterClassW, RegisterRawInputDevices,
    SendInput, CW_USEDEFAULT, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC_EX, MAPVK_VSC_TO_VK_EX,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_MOVE, MOUSEINPUT, MSG, RAWINPUT, RAWINPUTDEVICE,
    RAWINPUTHEADER, RIDEV_INPUTSINK, RID_INPUT, WM_INPUT, WNDCLASSW,
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
                LAST_GAME_HWND.store(hwnd as isize, Ordering::Relaxed);
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
    use winapi::um::winuser::{SendInput, INPUT, INPUT_MOUSE, MOUSEINPUT};
    // S'assurer que le curseur est positionné sur la cible si des coordonnées valides sont fournies
    if x > 0 || y > 0 {
        send_mouse_move(x, y);
    }
    unsafe {
        let flag = mouse_button_dwflags(button, down);

        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: std::mem::zeroed(),
        };
        *input.u.mi_mut() = MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: 0,
            dwFlags: flag,
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(windows)]
fn mouse_button_dwflags(builder_button: u8, down: bool) -> u32 {
    use winapi::um::winuser::{
        MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
        MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    };
    match (builder_button, down) {
        (1, true) => MOUSEEVENTF_LEFTDOWN,
        (1, false) => MOUSEEVENTF_LEFTUP,
        (2, true) => MOUSEEVENTF_RIGHTDOWN,
        (2, false) => MOUSEEVENTF_RIGHTUP,
        (_, true) => MOUSEEVENTF_MIDDLEDOWN,
        (_, false) => MOUSEEVENTF_MIDDLEUP,
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
        use winapi::um::winuser::KEYEVENTF_SCANCODE;
        let scan = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) as u16;
        let mut flags = 0;
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        if is_extended {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        // Pour une compatibilité maximale (jeux DirectX / DirectInput / Win32),
        // si le scan code matériel est résolu, injecter avec KEYEVENTF_SCANCODE.
        if scan != 0 {
            flags |= KEYEVENTF_SCANCODE;
        }
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        };
        *input.u.ki_mut() = KEYBDINPUT {
            wVk: if scan == 0 { vk } else { 0 },
            wScan: scan,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(windows)]
thread_local! {
    static TLS_SCREEN_BUFFER: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Capture rapide d'un rectangle de l'écran via GDI (Windows uniquement) dans un buffer existant.
/// Évite toute réallocation si le buffer a déjà la capacité requise (width * height * 4 octets).
/// Retourne true si la capture a réussi, false sinon.
#[cfg(windows)]
pub fn capture_screen_gdi_into(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    buffer: &mut Vec<u8>,
) -> bool {
    if width <= 0 || height <= 0 {
        return false;
    }
    let required_len = (width as usize) * (height as usize) * 4;
    if buffer.len() != required_len {
        buffer.resize(required_len, 0);
    }

    use std::ptr::null_mut;
    use winapi::um::wingdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        SelectObject, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
    };
    use winapi::um::winuser::{GetDC, ReleaseDC};

    unsafe {
        let hdc_screen = GetDC(null_mut());
        if hdc_screen.is_null() {
            return false;
        }

        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_null() {
            ReleaseDC(null_mut(), hdc_screen);
            return false;
        }

        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbm.is_null() {
            DeleteDC(hdc_mem);
            ReleaseDC(null_mut(), hdc_screen);
            return false;
        }

        let old_obj = SelectObject(hdc_mem, hbm as *mut _);

        let blt_ok = BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, x, y, SRCCOPY);
        if blt_ok == 0 {
            SelectObject(hdc_mem, old_obj);
            DeleteObject(hbm as *mut _);
            DeleteDC(hdc_mem);
            ReleaseDC(null_mut(), hdc_screen);
            return false;
        }

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

        let lines_copied = GetDIBits(
            hdc_mem,
            hbm,
            0,
            height as u32,
            buffer.as_mut_ptr() as *mut _,
            &mut bmi as *mut _ as *mut _,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc_mem, old_obj);
        DeleteObject(hbm as *mut _);
        DeleteDC(hdc_mem);
        ReleaseDC(null_mut(), hdc_screen);

        lines_copied != 0
    }
}

/// Capture rapide d'un rectangle de l'écran via GDI en passant une closure sans allouer de copie du buffer.
#[cfg(windows)]
pub fn with_screen_capture_gdi<R>(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    f: impl FnOnce(&[u8]) -> R,
) -> Option<R> {
    if width <= 0 || height <= 0 {
        return None;
    }

    TLS_SCREEN_BUFFER.with(|cell| {
        let mut buf = cell.borrow_mut();
        if capture_screen_gdi_into(x, y, width, height, &mut buf) {
            Some(f(&buf))
        } else {
            None
        }
    })
}

/// Capture rapide d'un rectangle de l'écran via GDI (Windows uniquement).
/// Retourne les pixels en format BGRA dans un nouveau Vec<u8>.
#[cfg(windows)]
pub fn capture_screen_gdi(x: i32, y: i32, width: i32, height: i32) -> Option<Vec<u8>> {
    with_screen_capture_gdi(x, y, width, height, |buf| buf.to_vec())
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

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowLockConfig {
    pub enabled: bool,
    pub title_filter: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub force_foreground: bool,
    pub restore_if_maximized: bool,
    #[serde(default)]
    pub embed_in_macroforge: bool,
    #[serde(default = "default_true")]
    pub lock_window_styles: bool,
    #[serde(default = "default_true")]
    pub enforce_continuous_clamp: bool,
}

impl Default for WindowLockConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            title_filter: String::new(),
            x: 100,
            y: 100,
            width: 1280,
            height: 720,
            force_foreground: true,
            restore_if_maximized: true,
            embed_in_macroforge: false,
            lock_window_styles: true,
            enforce_continuous_clamp: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OriginalWindowState {
    pub hwnd: isize,
    pub parent_hwnd: isize,
    pub style: isize,
    pub ex_style: isize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct MacroState {
    pub is_recording: bool,
    pub is_playing: bool,
    pub actions: Vec<MacroAction>,
    pub last_event_time: Option<Instant>,
    pub recording_start_time: Option<Instant>,
    pub expected_time_cursor: f64,
    pub stop_playback_flag: Arc<AtomicBool>,
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
    pub window_lock: WindowLockConfig,
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
            stop_playback_flag: Arc::new(AtomicBool::new(false)),
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
            window_lock: WindowLockConfig::default(),
        }
    }
}

const EXTREME_IMAGE_DATA: &[u8] = include_bytes!("../extreme.png");
const FAILED_IMAGE_DATA: &[u8] = include_bytes!("../failed.PNG");

pub type EmbeddedViewportRect = (i32, i32, i32, i32, bool);

lazy_static::lazy_static! {
    pub static ref MACRO_STATE: Mutex<MacroState> = Mutex::new(MacroState::new());
    pub static ref EVENT_SENDER: Mutex<Option<Sender<EngineEvent>>> = Mutex::new(None);
    pub static ref EGUI_CTX: Mutex<Option<eframe::egui::Context>> = Mutex::new(None);
    pub static ref IMAGE_CACHE: Mutex<HashMap<String, Arc<image::RgbaImage>>> = Mutex::new(HashMap::new());
    static ref LAST_RECORD_TOGGLE: Mutex<Option<Instant>> = Mutex::new(None);
    static ref SAVED_WINDOW_STATES: Mutex<HashMap<isize, OriginalWindowState>> = Mutex::new(HashMap::new());
    static ref EMBEDDED_VIEWPORT: Mutex<Option<EmbeddedViewportRect>> = Mutex::new(None);
}

#[cfg(windows)]
pub static LAST_GAME_HWND: AtomicIsize = AtomicIsize::new(0);

#[cfg(windows)]
static RAW_INPUT_FLAG: AtomicBool = AtomicBool::new(false);
#[cfg(windows)]
static RAW_INPUT_RECORDING: AtomicBool = AtomicBool::new(false);
#[cfg(windows)]
static RIGHT_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
static RAW_INPUT_LISTENER_INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

#[cfg(windows)]
const HID_USAGE_PAGE_GENERIC: u16 = 0x01;
#[cfg(windows)]
const HID_USAGE_GENERIC_MOUSE: u16 = 0x02;

pub fn set_event_sender(sender: Sender<EngineEvent>) {
    *EVENT_SENDER.lock().unwrap() = Some(sender);
}

pub fn set_egui_ctx(ctx: eframe::egui::Context) {
    *EGUI_CTX.lock().unwrap() = Some(ctx);
}

fn notify_event(event: EngineEvent) {
    if let Some(ref sender) = *EVENT_SENDER.lock().unwrap() {
        let _ = sender.send(event);
    }
    if let Some(ref ctx) = *EGUI_CTX.lock().unwrap() {
        ctx.request_repaint();
    }
}

/// Bascule atomique et protégée par anti-rebond de l'enregistrement de macro
pub fn toggle_recording() {
    let mut last = LAST_RECORD_TOGGLE.lock().unwrap();
    let now = Instant::now();
    if let Some(prev) = *last {
        if now.duration_since(prev) < std::time::Duration::from_millis(250) {
            return;
        }
    }
    *last = Some(now);

    let is_rec = {
        let s = MACRO_STATE.lock().unwrap();
        s.is_recording
    };
    if !is_rec {
        start_recording();
    } else {
        stop_recording();
    }
}

/// Bascule atomique de la relecture de macro (Play / Stop)
pub fn toggle_playback() {
    let is_playing = {
        let state = MACRO_STATE.lock().unwrap();
        state.is_playing
    };
    if is_playing {
        stop_playback();
    } else {
        play_macro();
    }
}

/// Arrêt d'urgence immédiat de toute relecture ou enregistrement
pub fn emergency_stop() {
    stop_playback();
    let was_recording = {
        let mut s = MACRO_STATE.lock().unwrap();
        let rec = s.is_recording;
        s.is_recording = false;
        rec
    };
    if was_recording {
        #[cfg(windows)]
        {
            RAW_INPUT_FLAG.store(false, Ordering::SeqCst);
            RAW_INPUT_RECORDING.store(false, Ordering::SeqCst);
            RIGHT_MOUSE_DOWN.store(false, Ordering::SeqCst);
        }
        emit_recording_state(false);
    }
}

/// Écouteur global ultra-réactif des raccourcis Windows (F8: Rec/Stop, F9: Stop Rec, F7: Play/Stop, F4: Arrêt Urgence)
/// Utilise GetAsyncKeyState avec détection de front montant et anti-rebond (debounce).
/// Fonctionne de manière universelle dans tous les jeux (Plein écran, fenêtré, DirectX, Vulkan, etc.)
#[cfg(windows)]
pub fn start_global_hotkey_listener() {
    use std::time::{Duration, Instant};
    use winapi::um::winuser::GetAsyncKeyState;

    thread::spawn(|| {
        let mut was_f8 = false;
        let mut was_f9 = false;
        let mut was_f7 = false;
        let mut was_f4 = false;
        let mut last_f8_time = Instant::now() - Duration::from_secs(1);
        let mut last_f9_time = Instant::now() - Duration::from_secs(1);
        let mut last_f7_time = Instant::now() - Duration::from_secs(1);
        let mut last_f4_time = Instant::now() - Duration::from_secs(1);

        loop {
            thread::sleep(Duration::from_millis(8)); // ~120 Hz, zéro latence perçue, <0.01% CPU

            unsafe {
                // 1. VK_F8 (0x77) -> Toggle Enregistrement global
                let is_f8 = (GetAsyncKeyState(0x77) as u16 & 0x8000) != 0;
                if is_f8 && !was_f8 && last_f8_time.elapsed() >= Duration::from_millis(250) {
                    last_f8_time = Instant::now();
                    toggle_recording();
                }
                was_f8 = is_f8;

                // 2. VK_F9 (0x78) -> Arrêter l'enregistrement
                let is_f9 = (GetAsyncKeyState(0x78) as u16 & 0x8000) != 0;
                if is_f9 && !was_f9 && last_f9_time.elapsed() >= Duration::from_millis(250) {
                    last_f9_time = Instant::now();
                    stop_recording();
                }
                was_f9 = is_f9;

                // 3. VK_F7 (0x76) -> Lancer / Basculer la relecture
                let is_f7 = (GetAsyncKeyState(0x76) as u16 & 0x8000) != 0;
                if is_f7 && !was_f7 && last_f7_time.elapsed() >= Duration::from_millis(250) {
                    last_f7_time = Instant::now();
                    toggle_playback();
                }
                was_f7 = is_f7;

                // 4. VK_F4 (0x73) -> Arrêt d'urgence
                let is_f4 = (GetAsyncKeyState(0x73) as u16 & 0x8000) != 0;
                if is_f4 && !was_f4 && last_f4_time.elapsed() >= Duration::from_millis(250) {
                    last_f4_time = Instant::now();
                    emergency_stop();
                }
                was_f4 = is_f4;
            }
        }
    });
}

#[cfg(windows)]
fn spawn_raw_input_listener() {
    // Créé une seule fois pour la durée de vie du processus (issue #31) :
    // chaque start_recording() réutilise ce thread/fenêtre permanent.
    if RAW_INPUT_LISTENER_INIT.get().is_some() {
        return;
    }
    RAW_INPUT_LISTENER_INIT
        .set(())
        .expect("Raw Input listener déjà initialisé");

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
            error!("Impossible d'enregistrer les Raw Input Devices");
            return;
        }

        let mut msg: MSG = std::mem::zeroed();
        let mut raw_buf = std::mem::MaybeUninit::<RAWINPUT>::uninit();
        let mut size = std::mem::size_of::<RAWINPUT>() as u32;

        while GetMessageW(&mut msg, hwnd, 0, 0) != 0 {
            if msg.message == WM_INPUT {
                let ret = GetRawInputData(
                    msg.lParam as *mut _,
                    RID_INPUT,
                    raw_buf.as_mut_ptr() as *mut _,
                    &mut size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                );

                if ret != u32::MAX && ret > 0 {
                    let raw = &*raw_buf.as_ptr();
                    if raw.header.dwType == winapi::um::winuser::RIM_TYPEMOUSE {
                        let mouse = raw.data.mouse();
                        let dx = mouse.lLastX;
                        let dy = mouse.lLastY;

                        // Filtrage atomique ultra-rapide sans lock si inactif (issue #18 et #19)
                        if (dx != 0 || dy != 0)
                            && RAW_INPUT_RECORDING.load(Ordering::Relaxed)
                            && RIGHT_MOUSE_DOWN.load(Ordering::Relaxed)
                        {
                            let mut state = MACRO_STATE.lock().unwrap();
                            if state.is_recording && state.is_right_mouse_down {
                                state.last_x += dx as f64;
                                state.last_y += dy as f64;

                                state.pending_dx += dx;
                                state.pending_dy += dy;

                                let now = Instant::now();
                                let should_record =
                                    if let Some(last_move) = state.last_move_record_time {
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
                                        action_type: ActionType::MouseMoveRelative(
                                            snap_dx, snap_dy,
                                        ),
                                        delay_ms,
                                    });
                                }
                            }
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

/// Garde RAII de la résolution timer Windows : passe la granularite scheduler
/// a 1 ms pendant le playback et restaure systematiquement l'etat initial
/// (issue #12), y compris en cas d'arret d'urgence ou de panic.
#[cfg(windows)]
struct TimerResolutionGuard {
    active: bool,
}

#[cfg(windows)]
impl TimerResolutionGuard {
    fn new() -> Self {
        // timeBeginPeriod(1) abaisse la granularite du scheduler Windows
        // (~15,6 ms par defaut) pour des sleep(1 ms) reels.
        let active = unsafe { timeBeginPeriod(1) } == 0 /* TIMERR_NOERROR */;
        if !active {
            warn!("timeBeginPeriod(1) a echoue, gigue timer possible.");
        }
        Self { active }
    }
}

#[cfg(windows)]
impl Drop for TimerResolutionGuard {
    fn drop(&mut self) {
        if self.active {
            unsafe { timeEndPeriod(1) };
        }
    }
}

#[inline]
fn pixels_match(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, tolerance: u8) -> bool {
    (r1 as i16 - r2 as i16).unsigned_abs() <= tolerance as u16
        && (g1 as i16 - g2 as i16).unsigned_abs() <= tolerance as u16
        && (b1 as i16 - b2 as i16).unsigned_abs() <= tolerance as u16
}

/// Recherche parallèle (via Rayon) d'un template RGBA dans une capture d'écran BGRA.
/// Retourne les coordonnées (x, y) de la première occurrence trouvée.
/// Optimisations :
/// - Rejet précoce sur points stratégiques (coin haut-gauche (0,0), centre, coin bas-droit).
/// - Vérification complète par grille espacée (step_by 2) pour accélérer le matching.
pub fn find_template_in_bgra(
    screen_raw: &[u8],
    screen_width: usize,
    screen_height: usize,
    template_raw: &[u8],
    template_width: usize,
    template_height: usize,
    tolerance: u8,
) -> Option<(usize, usize)> {
    if template_width == 0
        || template_height == 0
        || template_width > screen_width
        || template_height > screen_height
    {
        return None;
    }
    if screen_raw.len() < screen_width * screen_height * 4
        || template_raw.len() < template_width * template_height * 4
    {
        return None;
    }

    let t_mid_y = template_height / 2;
    let t_mid_x = template_width / 2;
    let t_mid_idx = (t_mid_y * template_width + t_mid_x) * 4;

    let t_last_y = template_height - 1;
    let t_last_x = template_width - 1;
    let t_last_idx = (t_last_y * template_width + t_last_x) * 4;

    (0..=(screen_height - template_height))
        .into_par_iter()
        .find_map_any(|sy| {
            let monitor_row_start = sy * screen_width * 4;
            for sx in 0..=(screen_width - template_width) {
                let monitor_pixel_idx = monitor_row_start + sx * 4;

                // 1. Point 1 : Coin supérieur gauche (0, 0)
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
                    tolerance,
                ) {
                    continue;
                }

                // 2. Point 2 : Centre
                let s_mid_idx = ((sy + t_mid_y) * screen_width + (sx + t_mid_x)) * 4;
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
                    tolerance,
                ) {
                    continue;
                }

                // 3. Point 3 : Coin inférieur droit
                let s_last_idx = ((sy + t_last_y) * screen_width + (sx + t_last_x)) * 4;
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
                    tolerance,
                ) {
                    continue;
                }

                // 4. Vérification complète sur grille (step_by 2)
                let mut matched = true;
                'tmatch: for ty in (0..template_height).step_by(2) {
                    let t_row_start = ty * template_width * 4;
                    let s_row_start = (sy + ty) * screen_width * 4;
                    for tx in (0..template_width).step_by(2) {
                        let t_idx = t_row_start + tx * 4;
                        let s_idx = s_row_start + (sx + tx) * 4;
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
                            tolerance,
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
        })
}

fn rdev_key_to_name_and_scan(key: &RdevKey) -> (std::borrow::Cow<'static, str>, u16, bool) {
    let mut is_extended = false;
    let (name, vk): (std::borrow::Cow<'static, str>, u16) = match key {
        RdevKey::Return => ("Return".into(), 0x0D),
        RdevKey::Space => ("Space".into(), 0x20),
        RdevKey::Backspace => ("Backspace".into(), 0x08),
        RdevKey::Tab => ("Tab".into(), 0x09),
        RdevKey::Escape => ("Escape".into(), 0x1B),
        RdevKey::Delete => {
            is_extended = true;
            ("Delete".into(), 0x2E)
        }
        RdevKey::Home => {
            is_extended = true;
            ("Home".into(), 0x24)
        }
        RdevKey::End => {
            is_extended = true;
            ("End".into(), 0x23)
        }
        RdevKey::PageUp => {
            is_extended = true;
            ("PageUp".into(), 0x21)
        }
        RdevKey::PageDown => {
            is_extended = true;
            ("PageDown".into(), 0x22)
        }
        RdevKey::UpArrow => {
            is_extended = true;
            ("UpArrow".into(), 0x26)
        }
        RdevKey::DownArrow => {
            is_extended = true;
            ("DownArrow".into(), 0x28)
        }
        RdevKey::LeftArrow => {
            is_extended = true;
            ("LeftArrow".into(), 0x25)
        }
        RdevKey::RightArrow => {
            is_extended = true;
            ("RightArrow".into(), 0x27)
        }
        RdevKey::ShiftLeft => ("ShiftLeft".into(), 0xA0),
        RdevKey::ShiftRight => ("ShiftRight".into(), 0xA1),
        RdevKey::ControlLeft => ("ControlLeft".into(), 0xA2),
        RdevKey::ControlRight => {
            is_extended = true;
            ("ControlRight".into(), 0xA3)
        }
        RdevKey::Alt => ("Alt".into(), 0x12),
        RdevKey::AltGr => {
            is_extended = true;
            ("AltGr".into(), 0xA5)
        }
        RdevKey::MetaLeft => {
            is_extended = true;
            ("MetaLeft".into(), 0x5B)
        }
        RdevKey::MetaRight => {
            is_extended = true;
            ("MetaRight".into(), 0x5C)
        }
        RdevKey::CapsLock => ("CapsLock".into(), 0x14),
        RdevKey::F1 => ("F1".into(), 0x70),
        RdevKey::F2 => ("F2".into(), 0x71),
        RdevKey::F3 => ("F3".into(), 0x72),
        RdevKey::F4 => ("F4".into(), 0x73),
        RdevKey::F5 => ("F5".into(), 0x74),
        RdevKey::F6 => ("F6".into(), 0x75),
        RdevKey::F7 => ("F7".into(), 0x76),
        RdevKey::F8 => ("F8".into(), 0x77),
        RdevKey::F9 => ("F9".into(), 0x78),
        RdevKey::F10 => ("F10".into(), 0x79),
        RdevKey::F11 => ("F11".into(), 0x7A),
        RdevKey::F12 => ("F12".into(), 0x7B),
        RdevKey::KeyA => ("KeyA".into(), 0x41),
        RdevKey::KeyB => ("KeyB".into(), 0x42),
        RdevKey::KeyC => ("KeyC".into(), 0x43),
        RdevKey::KeyD => ("KeyD".into(), 0x44),
        RdevKey::KeyE => ("KeyE".into(), 0x45),
        RdevKey::KeyF => ("KeyF".into(), 0x46),
        RdevKey::KeyG => ("KeyG".into(), 0x47),
        RdevKey::KeyH => ("KeyH".into(), 0x48),
        RdevKey::KeyI => ("KeyI".into(), 0x49),
        RdevKey::KeyJ => ("KeyJ".into(), 0x4A),
        RdevKey::KeyK => ("KeyK".into(), 0x4B),
        RdevKey::KeyL => ("KeyL".into(), 0x4C),
        RdevKey::KeyM => ("KeyM".into(), 0x4D),
        RdevKey::KeyN => ("KeyN".into(), 0x4E),
        RdevKey::KeyO => ("KeyO".into(), 0x4F),
        RdevKey::KeyP => ("KeyP".into(), 0x50),
        RdevKey::KeyQ => ("KeyQ".into(), 0x51),
        RdevKey::KeyR => ("KeyR".into(), 0x52),
        RdevKey::KeyS => ("KeyS".into(), 0x53),
        RdevKey::KeyT => ("KeyT".into(), 0x54),
        RdevKey::KeyU => ("KeyU".into(), 0x55),
        RdevKey::KeyV => ("KeyV".into(), 0x56),
        RdevKey::KeyW => ("KeyW".into(), 0x57),
        RdevKey::KeyX => ("KeyX".into(), 0x58),
        RdevKey::KeyY => ("KeyY".into(), 0x59),
        RdevKey::KeyZ => ("KeyZ".into(), 0x5A),
        RdevKey::Num0 => ("Num0".into(), 0x30),
        RdevKey::Num1 => ("Num1".into(), 0x31),
        RdevKey::Num2 => ("Num2".into(), 0x32),
        RdevKey::Num3 => ("Num3".into(), 0x33),
        RdevKey::Num4 => ("Num4".into(), 0x34),
        RdevKey::Num5 => ("Num5".into(), 0x35),
        RdevKey::Num6 => ("Num6".into(), 0x36),
        RdevKey::Num7 => ("Num7".into(), 0x37),
        RdevKey::Num8 => ("Num8".into(), 0x38),
        RdevKey::Num9 => ("Num9".into(), 0x39),
        RdevKey::Comma => ("Comma".into(), 0xBC),
        RdevKey::Dot => ("Dot".into(), 0xBE),
        RdevKey::Minus => ("Minus".into(), 0xBD),
        RdevKey::Equal => ("Equal".into(), 0xBB),
        RdevKey::SemiColon => ("SemiColon".into(), 0xBA),
        RdevKey::Quote => ("Quote".into(), 0xDE),
        RdevKey::BackSlash => ("BackSlash".into(), 0xDC),
        RdevKey::Slash => ("Slash".into(), 0xBF),
        RdevKey::BackQuote => ("BackQuote".into(), 0xC0),
        RdevKey::Unknown(sc) => {
            #[cfg(windows)]
            let vk = unsafe { MapVirtualKeyW(*sc, MAPVK_VSC_TO_VK_EX) as u16 };
            #[cfg(not(windows))]
            let vk = *sc as u16;
            (format!("Unknown({})", sc).into(), vk)
        }
        _ => (format!("{:?}", key).into(), 0),
    };
    (name, vk, is_extended)
}

pub fn start_recording() {
    #[cfg(windows)]
    {
        RAW_INPUT_FLAG.store(true, Ordering::SeqCst);
        RAW_INPUT_RECORDING.store(true, Ordering::SeqCst);
        RIGHT_MOUSE_DOWN.store(false, Ordering::SeqCst);
    }
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
        RAW_INPUT_FLAG.store(false, Ordering::SeqCst);
        RAW_INPUT_RECORDING.store(false, Ordering::SeqCst);
        RIGHT_MOUSE_DOWN.store(false, Ordering::SeqCst);
    }

    emit_recording_state(false);
    count
}

#[cfg(windows)]
pub fn get_screen_capture_bounds() -> (i32, i32, i32, i32) {
    let hwnd = LAST_GAME_HWND.load(Ordering::Relaxed) as winapi::shared::windef::HWND;
    if !hwnd.is_null() && unsafe { IsWindowVisible(hwnd) } != 0 {
        let mut rect = winapi::shared::windef::RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { winapi::um::winuser::GetWindowRect(hwnd, &mut rect) } != 0 {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            if width > 100 && height > 100 {
                return (rect.left, rect.top, width, height);
            }
        }
    }
    use winapi::um::winuser::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };
    let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    (vx, vy, vw, vh)
}

pub fn play_macro() {
    let mut state = MACRO_STATE.lock().unwrap();
    if state.is_playing || state.is_recording {
        return;
    }

    if state.actions.is_empty() {
        debug!("Lecture annulée : aucune action dans la macro.");
        return;
    }

    state.is_playing = true;
    let actions_to_play = state.actions.clone();
    let stop_flag = Arc::clone(&state.stop_playback_flag);
    let window_lock_cfg = state.window_lock.clone();
    stop_flag.store(false, Ordering::SeqCst);

    drop(state);

    notify_event(EngineEvent::PlaybackStateChanged(true));

    thread::spawn(move || {
        let playback_start = Instant::now();
        // Resolution timer 1 ms pour toute la duree du playback (issue #12).
        // Le garde RAII garantit timeEndPeriod(1) meme sur break 'main_loop,
        // arret F4 ou unwind panic.
        #[cfg(windows)]
        let _timer_guard = TimerResolutionGuard::new();

        // 1. Si le verrouillage de fenêtre est activé, emprisonner la fenêtre cible (taille et position fixe)
        #[cfg(windows)]
        if window_lock_cfg.enabled {
            if let Err(e) = apply_window_lock(&window_lock_cfg) {
                warn!(
                    "Impossible d'emprisonner la fenêtre cible au lancement : {}",
                    e
                );
            } else {
                info!(
                    "🎯 Fenêtre cible emprisonnée avec succès : {}x{} en ({}, {})",
                    window_lock_cfg.width,
                    window_lock_cfg.height,
                    window_lock_cfg.x,
                    window_lock_cfg.y
                );
                thread::sleep(Duration::from_millis(150));
            }
        } else {
            // Si la fenêtre active est MacroForge, redonner automatiquement le focus à l'application cible
            unsafe {
                use winapi::um::winuser::{
                    GetForegroundWindow, GetWindowTextW, IsWindowVisible, SetForegroundWindow,
                };
                let cur_hwnd = GetForegroundWindow();
                if !cur_hwnd.is_null() {
                    let mut buf = [0u16; 256];
                    let len = GetWindowTextW(cur_hwnd, buf.as_mut_ptr(), buf.len() as i32);
                    let title = if len > 0 {
                        String::from_utf16_lossy(&buf[..len as usize])
                    } else {
                        String::new()
                    };
                    if title.contains("MacroForge") {
                        let target_hwnd =
                            LAST_GAME_HWND.load(Ordering::Relaxed) as winapi::shared::windef::HWND;
                        if !target_hwnd.is_null() && IsWindowVisible(target_hwnd) != 0 {
                            SetForegroundWindow(target_hwnd);
                            thread::sleep(Duration::from_millis(150));
                        }
                    }
                }
            }
        }

        let ts = || format!("[+{:.2}s]", playback_start.elapsed().as_secs_f64());
        let total_actions = actions_to_play.len();

        info!(
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
        let mut last_mouse_emit = Instant::now() - Duration::from_secs(1);

        'main_loop: loop {
            iteration += 1;
            trace!("{} --- Itération #{} démarrée ---", ts(), iteration);

            let mut action_index = 0usize;
            let mut timeline_origin = Instant::now();
            let mut total_recorded_delay = 0u64;

            for action in &actions_to_play {
                action_index += 1;

                if stop_flag.load(Ordering::Relaxed) {
                    debug!(
                        "{} [STOP] stop_flag détecté avant action #{} — arrêt.",
                        ts(),
                        action_index
                    );
                    break 'main_loop;
                }

                total_recorded_delay += action.delay_ms;

                #[cfg(windows)]
                if window_lock_cfg.enabled && window_lock_cfg.enforce_continuous_clamp {
                    clamp_target_window_bounds(&window_lock_cfg);
                }

                if let Some(ref path) = stop_image_config {
                    let now = Instant::now();
                    let in_blackout = stop_blackout_until.map(|t| now < t).unwrap_or(false);

                    if !in_blackout && last_stop_check.elapsed() >= Duration::from_secs(3) {
                        last_stop_check = now;
                        if check_image_present(path) {
                            if MACRO_STATE.lock().unwrap().loop_playback {
                                info!(
                                    "{} [STOP IMAGE] Détectée ! Redémarrage (Blackout 15s activé).",
                                    ts()
                                );
                                stop_blackout_until = Some(now + Duration::from_secs(15));
                                continue 'main_loop;
                            } else {
                                info!("{} [STOP IMAGE] Détectée ! Arrêt définitif.", ts());
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

                    if stop_flag.load(Ordering::Relaxed) {
                        break 'main_loop;
                    }

                    if let Some(ref path) = stop_image_config {
                        let now = Instant::now();
                        let in_blackout = stop_blackout_until.map(|t| now < t).unwrap_or(false);

                        if !in_blackout && last_stop_check.elapsed() >= Duration::from_secs(3) {
                            last_stop_check = now;
                            if check_image_present(path) {
                                if MACRO_STATE.lock().unwrap().loop_playback {
                                    info!("{} [STOP IMAGE] Détectée pendant attente ! Redémarrage (Blackout 15s).", ts());
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
                            trace!(
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
                            trace!(
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
                            let now = Instant::now();
                            if action_index == total_actions
                                || now.duration_since(last_mouse_emit) >= Duration::from_millis(33)
                            {
                                last_mouse_emit = now;
                                emit_playback_action(PlaybackActionPayload {
                                    index: action_index,
                                    total: total_actions,
                                    action_type: "MoveRel".into(),
                                    x: dx as f64,
                                    y: dy as f64,
                                    detail: format!("Relative {}x{}", dx, dy),
                                });
                            }
                            send_mouse_relative(dx, dy);
                        }
                        ActionType::MouseMove(x, y) => {
                            let now = Instant::now();
                            if action_index == total_actions
                                || now.duration_since(last_mouse_emit) >= Duration::from_millis(33)
                            {
                                last_mouse_emit = now;
                                emit_playback_action(PlaybackActionPayload {
                                    index: action_index,
                                    total: total_actions,
                                    action_type: "Move".into(),
                                    x,
                                    y,
                                    detail: format!("Pos {}x{}", x, y),
                                });
                            }
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
                            send_mouse_button(u, true, x as i32, y as i32);
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
                            send_mouse_button(u, false, x as i32, y as i32);
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
                            trace!(
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
                                            error!("{} WaitImage: ERREUR chargement image intégrée: {} — action ignorée.", ts(), e);
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
                                            error!("{} WaitImage: ERREUR ouverture image '{}': {} — action ignorée.", ts(), path, e);
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
                                    if stop_flag.load(Ordering::Relaxed) {
                                        break 'main_loop;
                                    }

                                    #[cfg(windows)]
                                    {
                                        let (vx, vy, vw, vh) = get_screen_capture_bounds();

                                        if vw > 0 && vh > 0 {
                                            let found_pos = with_screen_capture_gdi(
                                                vx,
                                                vy,
                                                vw,
                                                vh,
                                                |screen_raw| {
                                                    find_template_in_bgra(
                                                        screen_raw,
                                                        vw as usize,
                                                        vh as usize,
                                                        template_raw,
                                                        tw,
                                                        th,
                                                        25,
                                                    )
                                                },
                                            )
                                            .flatten();

                                            if found_pos.is_some() {
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
                            trace!(
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
                                if stop_flag.load(Ordering::Relaxed) {
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

            if stop_flag.load(Ordering::Relaxed) {
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
        info!("{} === PLAYBACK TERMINÉ ===", ts());

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
    state.stop_playback_flag.store(true, Ordering::SeqCst);
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

pub fn get_window_lock() -> WindowLockConfig {
    let state = MACRO_STATE.lock().unwrap();
    state.window_lock.clone()
}

pub fn set_window_lock(config: WindowLockConfig) {
    let mut state = MACRO_STATE.lock().unwrap();
    state.window_lock = config;
}

#[cfg(windows)]
pub fn get_primary_screen_dimensions() -> (i32, i32) {
    use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe {
        let w = GetSystemMetrics(SM_CXSCREEN);
        let h = GetSystemMetrics(SM_CYSCREEN);
        (w, h)
    }
}

#[cfg(not(windows))]
pub fn get_primary_screen_dimensions() -> (i32, i32) {
    (1920, 1080)
}

#[cfg(windows)]
pub fn list_open_windows() -> Vec<WindowInfo> {
    use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
    use winapi::shared::windef::{HWND, RECT};
    use winapi::um::winuser::{
        EnumWindows, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    };

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let list = &mut *(lparam as *mut Vec<WindowInfo>);

        if IsWindowVisible(hwnd) == 0 {
            return TRUE;
        }

        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return TRUE;
        }

        let mut buf = vec![0u16; (len + 1) as usize];
        let copied = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if copied <= 0 {
            return TRUE;
        }

        let title = String::from_utf16_lossy(&buf[..copied as usize]);
        let trimmed = title.trim();
        if trimmed.is_empty()
            || trimmed.contains("MacroForge")
            || trimmed == "Program Manager"
            || trimmed == "Settings"
            || trimmed == "Windows Input Experience"
        {
            return TRUE;
        }

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(hwnd, &mut rect) != 0 {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            if width > 60 && height > 60 {
                list.push(WindowInfo {
                    hwnd: hwnd as isize,
                    title: trimmed.to_string(),
                    x: rect.left,
                    y: rect.top,
                    width,
                    height,
                });
            }
        }

        TRUE
    }

    let mut windows: Vec<WindowInfo> = Vec::new();
    unsafe {
        EnumWindows(Some(enum_proc), &mut windows as *mut _ as LPARAM);
    }
    windows
}

#[cfg(not(windows))]
pub fn list_open_windows() -> Vec<WindowInfo> {
    Vec::new()
}

#[cfg(windows)]
pub fn capture_active_window_info() -> Option<WindowInfo> {
    use winapi::shared::windef::{HWND, RECT};
    use winapi::um::winuser::{
        GetForegroundWindow, GetWindowRect, GetWindowTextW, IsWindowVisible,
    };

    unsafe {
        let mut hwnd = GetForegroundWindow();
        if !hwnd.is_null() {
            let mut buf = [0u16; 256];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            let title = if len > 0 {
                String::from_utf16_lossy(&buf[..len as usize])
            } else {
                String::new()
            };
            if title.contains("MacroForge") {
                let last_hwnd = LAST_GAME_HWND.load(Ordering::Relaxed) as HWND;
                if !last_hwnd.is_null() && IsWindowVisible(last_hwnd) != 0 {
                    hwnd = last_hwnd;
                }
            }
        } else {
            let last_hwnd = LAST_GAME_HWND.load(Ordering::Relaxed) as HWND;
            if !last_hwnd.is_null() && IsWindowVisible(last_hwnd) != 0 {
                hwnd = last_hwnd;
            }
        }

        if hwnd.is_null() || IsWindowVisible(hwnd) == 0 {
            return None;
        }

        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        let title = if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        };

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(hwnd, &mut rect) != 0 {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            return Some(WindowInfo {
                hwnd: hwnd as isize,
                title: title.trim().to_string(),
                x: rect.left,
                y: rect.top,
                width,
                height,
            });
        }
    }
    None
}

#[cfg(not(windows))]
pub fn capture_active_window_info() -> Option<WindowInfo> {
    None
}

#[cfg(windows)]
pub fn find_target_window_hwnd(config: &WindowLockConfig) -> Option<winapi::shared::windef::HWND> {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::IsWindowVisible;

    let filter = config.title_filter.trim().to_lowercase();
    if !filter.is_empty() {
        let windows = list_open_windows();
        for w in windows {
            if w.title.to_lowercase().contains(&filter) {
                let hwnd = w.hwnd as HWND;
                if !hwnd.is_null() && unsafe { IsWindowVisible(hwnd) } != 0 {
                    return Some(hwnd);
                }
            }
        }
    }

    let last_hwnd = LAST_GAME_HWND.load(Ordering::Relaxed) as HWND;
    if !last_hwnd.is_null() && unsafe { IsWindowVisible(last_hwnd) } != 0 {
        return Some(last_hwnd);
    }

    None
}

#[cfg(windows)]
pub fn get_macroforge_main_hwnd() -> Option<winapi::shared::windef::HWND> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::shared::minwindef::{BOOL, DWORD, LPARAM, TRUE};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{
        EnumWindows, FindWindowW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    };

    let title: Vec<u16> = OsStr::new("MacroForge (Full Natif Windows)\0")
        .encode_wide()
        .collect();
    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() && IsWindowVisible(hwnd) != 0 {
            return Some(hwnd);
        }

        let current_pid = std::process::id();
        struct SearchData {
            pid: DWORD,
            found_hwnd: Option<HWND>,
        }
        let mut data = SearchData {
            pid: current_pid,
            found_hwnd: None,
        };

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let sdata = &mut *(lparam as *mut SearchData);
            let mut proc_id: DWORD = 0;
            GetWindowThreadProcessId(hwnd, &mut proc_id);
            if proc_id == sdata.pid && IsWindowVisible(hwnd) != 0 {
                let mut buf = [0u16; 256];
                let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
                let title_str = if len > 0 {
                    String::from_utf16_lossy(&buf[..len as usize])
                } else {
                    String::new()
                };
                if title_str.contains("MacroForge")
                    && !title_str.contains("Overlay")
                    && !title_str.contains("Toolbar")
                {
                    sdata.found_hwnd = Some(hwnd);
                    return 0;
                }
            }
            TRUE
        }

        EnumWindows(Some(enum_proc), &mut data as *mut _ as LPARAM);
        data.found_hwnd
    }
}

#[cfg(not(windows))]
pub fn get_macroforge_main_hwnd() -> Option<isize> {
    None
}

/// Met à jour les dimensions et l'état de visibilité du viewport intégré dans l'UI MacroForge.
pub fn update_embedded_viewport_bounds(x: i32, y: i32, width: i32, height: i32, visible: bool) {
    let mut vp = EMBEDDED_VIEWPORT.lock().unwrap();
    let prev = *vp;
    let new_val = Some((x, y, width, height, visible));
    if prev != new_val {
        *vp = new_val;
        #[cfg(windows)]
        {
            use winapi::shared::windef::HWND;
            use winapi::um::winuser::{
                GetParent, IsWindowVisible, SetWindowPos, ShowWindow, SWP_FRAMECHANGED,
                SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW,
            };
            let hwnd = LAST_GAME_HWND.load(Ordering::Relaxed) as HWND;
            if !hwnd.is_null() && unsafe { IsWindowVisible(hwnd) } != 0 {
                let parent = unsafe { GetParent(hwnd) };
                if let Some(mf_hwnd) = get_macroforge_main_hwnd() {
                    if parent == mf_hwnd {
                        if !visible {
                            unsafe { ShowWindow(hwnd, SW_HIDE) };
                        } else if width > 20 && height > 20 {
                            unsafe {
                                ShowWindow(hwnd, SW_SHOW);
                                SetWindowPos(
                                    hwnd,
                                    std::ptr::null_mut(),
                                    x,
                                    y,
                                    width,
                                    height,
                                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(windows))]
pub fn update_embedded_viewport_bounds(_x: i32, _y: i32, _width: i32, _height: i32, _visible: bool) {}

pub fn get_embedded_target_title() -> Option<String> {
    #[cfg(windows)]
    {
        use winapi::shared::windef::HWND;
        use winapi::um::winuser::{GetParent, GetWindowTextW, IsWindowVisible};
        let hwnd = LAST_GAME_HWND.load(Ordering::Relaxed) as HWND;
        if !hwnd.is_null() && unsafe { IsWindowVisible(hwnd) } != 0 {
            if let Some(mf_hwnd) = get_macroforge_main_hwnd() {
                let parent = unsafe { GetParent(hwnd) };
                if parent == mf_hwnd {
                    let mut buf = [0u16; 256];
                    let len = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
                    if len > 0 {
                        let title = String::from_utf16_lossy(&buf[..len as usize]);
                        let trimmed = title.trim().to_string();
                        if !trimmed.is_empty() {
                            return Some(trimmed);
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn is_target_window_embedded() -> bool {
    #[cfg(windows)]
    {
        use winapi::shared::windef::HWND;
        use winapi::um::winuser::{GetParent, IsWindowVisible};
        let hwnd = LAST_GAME_HWND.load(Ordering::Relaxed) as HWND;
        if !hwnd.is_null() && unsafe { IsWindowVisible(hwnd) } != 0 {
            if let Some(mf_hwnd) = get_macroforge_main_hwnd() {
                let parent = unsafe { GetParent(hwnd) };
                return parent == mf_hwnd;
            }
        }
    }
    false
}

#[cfg(windows)]
pub fn clamp_target_window_bounds(config: &WindowLockConfig) {
    use winapi::shared::windef::{HWND, RECT};
    use winapi::um::winuser::{
        GetWindowRect, IsIconic, IsWindowVisible, IsZoomed, SetWindowPos, ShowWindow,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_RESTORE,
    };

    let hwnd = LAST_GAME_HWND.load(Ordering::Relaxed) as HWND;
    if hwnd.is_null() || unsafe { IsWindowVisible(hwnd) } == 0 {
        return;
    }

    unsafe {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(hwnd, &mut rect) != 0 {
            let cur_w = rect.right - rect.left;
            let cur_h = rect.bottom - rect.top;
            let cur_x = rect.left;
            let cur_y = rect.top;

            let (target_x, target_y, target_w, target_h) = if config.embed_in_macroforge {
                let vp_opt = *EMBEDDED_VIEWPORT.lock().unwrap();
                if let Some((vx, vy, vw, vh, _vis)) = vp_opt {
                    (vx, vy, vw.max(50), vh.max(50))
                } else {
                    (config.x, config.y, config.width.max(50), config.height.max(50))
                }
            } else {
                (config.x, config.y, config.width.max(50), config.height.max(50))
            };

            let needs_clamp = IsZoomed(hwnd) != 0
                || IsIconic(hwnd) != 0
                || (cur_w - target_w).abs() > 4
                || (cur_h - target_h).abs() > 4
                || (!config.embed_in_macroforge
                    && ((cur_x - target_x).abs() > 4 || (cur_y - target_y).abs() > 4));

            if needs_clamp {
                if IsZoomed(hwnd) != 0 || IsIconic(hwnd) != 0 {
                    ShowWindow(hwnd, SW_RESTORE);
                }
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    target_x,
                    target_y,
                    target_w,
                    target_h,
                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
                );
            }
        }
    }
}

#[cfg(not(windows))]
pub fn clamp_target_window_bounds(_config: &WindowLockConfig) {}

#[cfg(windows)]
pub fn apply_window_lock(config: &WindowLockConfig) -> Result<(), String> {
    use winapi::shared::windef::RECT;
    use winapi::um::winuser::{
        GetParent, GetWindowLongPtrW, GetWindowRect, GetWindowTextW, IsIconic, IsZoomed,
        SetForegroundWindow, SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE,
        GWL_STYLE, SWP_FRAMECHANGED, SWP_NOZORDER, SWP_SHOWWINDOW, SW_RESTORE, WS_CAPTION,
        WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP,
        WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
    };

    let hwnd = find_target_window_hwnd(config)
        .ok_or_else(|| "Aucune fenêtre cible trouvée ou active.".to_string())?;

    let width = config.width.max(50);
    let height = config.height.max(50);

    unsafe {
        // 1. Sauvegarder l'état initial s'il n'est pas déjà enregistré
        {
            let mut states = SAVED_WINDOW_STATES.lock().unwrap();
            let key = hwnd as isize;
            states.entry(key).or_insert_with(|| {
                let parent = GetParent(hwnd);
                let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
                let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                GetWindowRect(hwnd, &mut rect);
                OriginalWindowState {
                    hwnd: key,
                    parent_hwnd: parent as isize,
                    style,
                    ex_style,
                    x: rect.left,
                    y: rect.top,
                    width: rect.right - rect.left,
                    height: rect.bottom - rect.top,
                }
            });
        }

        // 2. Restaurer si maximisée ou minimisée (pour débloquer le mode plein écran initial)
        if config.restore_if_maximized && (IsZoomed(hwnd) != 0 || IsIconic(hwnd) != 0) {
            ShowWindow(hwnd, SW_RESTORE);
            thread::sleep(Duration::from_millis(60));
        }

        // 3. Intégration en tant que fenêtre enfant (SetParent) si demandée
        if config.embed_in_macroforge {
            if let Some(parent_hwnd) = get_macroforge_main_hwnd() {
                if parent_hwnd != hwnd {
                    SetParent(hwnd, parent_hwnd);
                }
            }
        }

        // 4. Verrouillage strict des styles (suppression des bordures de redimensionnement et bouton maximiser/F11)
        if config.lock_window_styles || config.embed_in_macroforge {
            let mut current_style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;

            if config.embed_in_macroforge {
                current_style &= !(WS_POPUP
                    | WS_CAPTION
                    | WS_THICKFRAME
                    | WS_MINIMIZEBOX
                    | WS_MAXIMIZEBOX
                    | WS_SYSMENU);
                current_style |= WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN;
            } else if config.lock_window_styles {
                current_style &= !(WS_THICKFRAME | WS_MAXIMIZEBOX);
            }

            SetWindowLongPtrW(hwnd, GWL_STYLE, current_style as isize);
        }

        // 5. Application des dimensions et coordonnées
        let (apply_x, apply_y, apply_w, apply_h) = if config.embed_in_macroforge {
            let vp = *EMBEDDED_VIEWPORT.lock().unwrap();
            if let Some((vx, vy, vw, vh, _vis)) = vp {
                (vx, vy, vw.max(50), vh.max(50))
            } else {
                (config.x, config.y, width, height)
            }
        } else {
            (config.x, config.y, width, height)
        };

        let flags = SWP_NOZORDER | SWP_SHOWWINDOW | SWP_FRAMECHANGED;
        let success = SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            apply_x,
            apply_y,
            apply_w,
            apply_h,
            flags,
        );

        if success == 0 {
            return Err("Échec de l'appel SetWindowPos sur la fenêtre cible.".to_string());
        }

        if config.force_foreground && !config.embed_in_macroforge {
            SetForegroundWindow(hwnd);
            thread::sleep(Duration::from_millis(50));
        }

        LAST_GAME_HWND.store(hwnd as isize, Ordering::Relaxed);

        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        let title = if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            "Fenêtre".to_string()
        };

        info!(
            "🎯 Fenêtre cible '{}' emprisonnée avec succès (taille: {}x{}, pos: {}, {}, embed: {}, lock_styles: {})",
            title.trim(),
            apply_w,
            apply_h,
            apply_x,
            apply_y,
            config.embed_in_macroforge,
            config.lock_window_styles
        );
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn apply_window_lock(_config: &WindowLockConfig) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
pub fn restore_target_window(config: &WindowLockConfig) -> Result<(), String> {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{
        SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE,
        SWP_FRAMECHANGED, SWP_NOZORDER, SWP_SHOWWINDOW, SW_SHOWNORMAL,
    };

    let hwnd = find_target_window_hwnd(config)
        .ok_or_else(|| "Aucune fenêtre cible trouvée pour la restauration.".to_string())?;

    // Réinitialiser le viewport
    *EMBEDDED_VIEWPORT.lock().unwrap() = None;

    unsafe {
        let state_opt = {
            let mut states = SAVED_WINDOW_STATES.lock().unwrap();
            states.remove(&(hwnd as isize))
        };

        if let Some(state) = state_opt {
            let parent = if state.parent_hwnd != 0 {
                state.parent_hwnd as HWND
            } else {
                std::ptr::null_mut()
            };

            SetParent(hwnd, parent);
            SetWindowLongPtrW(hwnd, GWL_STYLE, state.style);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, state.ex_style);
            ShowWindow(hwnd, SW_SHOWNORMAL);

            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                state.x,
                state.y,
                state.width.max(100),
                state.height.max(100),
                SWP_NOZORDER | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
            );

            info!("🔓 Fenêtre cible rétablie dans son état d'origine.");
        } else {
            SetParent(hwnd, std::ptr::null_mut());
            ShowWindow(hwnd, SW_SHOWNORMAL);
        }
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn restore_target_window(_config: &WindowLockConfig) -> Result<(), String> {
    Ok(())
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
        let (vx, vy, vw, vh) = get_screen_capture_bounds();

        if vw <= 0 || vh <= 0 {
            return false;
        }

        with_screen_capture_gdi(vx, vy, vw, vh, |screen_raw| {
            find_template_in_bgra(
                screen_raw,
                vw as usize,
                vh as usize,
                template_raw,
                tw,
                th,
                25,
            )
            .is_some()
        })
        .unwrap_or(false)
    }
    #[cfg(not(windows))]
    false
}

pub fn handle_rdev_event(event: Event) {
    if let EventType::KeyPress(key) = &event.event_type {
        match key {
            RdevKey::F8 => {
                #[cfg(not(windows))]
                toggle_recording();
                return;
            }
            RdevKey::F9 => {
                #[cfg(not(windows))]
                stop_recording();
                return;
            }
            RdevKey::F7 => {
                #[cfg(not(windows))]
                toggle_playback();
                return;
            }
            RdevKey::F4 => {
                #[cfg(not(windows))]
                emergency_stop();
                return;
            }
            _ => {}
        }
    }

    if let EventType::KeyRelease(RdevKey::F8 | RdevKey::F9 | RdevKey::F7 | RdevKey::F4) =
        &event.event_type
    {
        return;
    }

    let mut state = MACRO_STATE.lock().unwrap();
    if !state.is_recording {
        return;
    }

    let action_type_opt = match &event.event_type {
        EventType::KeyPress(key) => {
            let (name, vk, is_ext) = rdev_key_to_name_and_scan(key);
            if vk == 0 || vk == 0x77 || vk == 0x78 || vk == 0x76 || vk == 0x73 {
                None
            } else if let std::collections::hash_map::Entry::Vacant(e) =
                state.key_press_times.entry(vk)
            {
                e.insert(Instant::now());
                Some(ActionType::KeyPress(name.into_owned(), vk, is_ext))
            } else {
                None
            }
        }
        EventType::KeyRelease(key) => {
            let (name, vk, is_ext) = rdev_key_to_name_and_scan(key);
            if vk == 0 || vk == 0x77 || vk == 0x78 || vk == 0x76 || vk == 0x73 {
                None
            } else {
                Some(ActionType::KeyRelease(name.into_owned(), vk, is_ext))
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
                #[cfg(windows)]
                RIGHT_MOUSE_DOWN.store(true, Ordering::SeqCst);
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
                #[cfg(windows)]
                RIGHT_MOUSE_DOWN.store(false, Ordering::SeqCst);
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

        // Drag and drop reorder test
        clear_actions();
        for i in 0..5 {
            add_action(MacroAction {
                action_type: ActionType::Wait(i * 100),
                delay_ms: 0,
            });
        }

        // Simuler glissement de #0 vers après #2 (target 3 -> actual 2)
        let from = 0;
        let to = 3;
        let actual_to = if to > from { to - 1 } else { to };
        assert!(move_action(from, actual_to));
        let current = get_actions();
        assert_eq!(current[0].action_type, ActionType::Wait(100));
        assert_eq!(current[1].action_type, ActionType::Wait(200));
        assert_eq!(current[2].action_type, ActionType::Wait(0)); // Déplacé ici

        // Simuler glissement de #4 vers avant #0 (target 0 -> actual 0)
        let from = 4;
        let to = 0;
        let actual_to = if to > from { to - 1 } else { to };
        assert!(move_action(from, actual_to));
        let current = get_actions();
        assert_eq!(current[0].action_type, ActionType::Wait(400)); // Placé en tête

        clear_actions();
        assert_eq!(get_actions_count(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn test_mouse_button_dwflags_do_not_move_cursor() {
        // Les clics ne doivent contenir QUE le flag de bouton : ni MOVE, ni ABSOLUTE.
        assert_eq!(
            mouse_button_dwflags(1, true),
            winapi::um::winuser::MOUSEEVENTF_LEFTDOWN
        );
        assert_eq!(
            mouse_button_dwflags(1, false),
            winapi::um::winuser::MOUSEEVENTF_LEFTUP
        );
        assert_eq!(
            mouse_button_dwflags(2, true),
            winapi::um::winuser::MOUSEEVENTF_RIGHTDOWN
        );
        assert_eq!(
            mouse_button_dwflags(2, false),
            winapi::um::winuser::MOUSEEVENTF_RIGHTUP
        );
        assert_eq!(
            mouse_button_dwflags(3, true),
            winapi::um::winuser::MOUSEEVENTF_MIDDLEDOWN
        );
        assert_eq!(
            mouse_button_dwflags(42, false),
            winapi::um::winuser::MOUSEEVENTF_MIDDLEUP
        );
    }

    #[test]
    fn test_find_template_in_bgra_exact_match_at_origin() {
        let sw = 10;
        let sh = 10;
        let tw = 3;
        let th = 3;
        let mut screen_bgra = vec![0u8; sw * sh * 4];
        let mut template_rgba = vec![0u8; tw * th * 4];

        // Template couleur uniforme rouge (R=255, G=0, B=0, A=255)
        for i in 0..(tw * th) {
            template_rgba[i * 4] = 255;
            template_rgba[i * 4 + 1] = 0;
            template_rgba[i * 4 + 2] = 0;
            template_rgba[i * 4 + 3] = 255;
        }

        // Placer le pattern rouge en (0, 0) dans l'écran BGRA (B=0, G=0, R=255, A=255)
        for ty in 0..th {
            for tx in 0..tw {
                let idx = (ty * sw + tx) * 4;
                screen_bgra[idx] = 0; // B
                screen_bgra[idx + 1] = 0; // G
                screen_bgra[idx + 2] = 255; // R
                screen_bgra[idx + 3] = 255; // A
            }
        }

        let result = find_template_in_bgra(&screen_bgra, sw, sh, &template_rgba, tw, th, 10);
        assert_eq!(result, Some((0, 0)));
    }

    #[test]
    fn test_find_template_in_bgra_offset_position() {
        let sw = 100;
        let sh = 80;
        let tw = 8;
        let th = 6;
        let mut screen_bgra = vec![128u8; sw * sh * 4]; // Fond gris
        let mut template_rgba = vec![0u8; tw * th * 4];

        let target_x = 42;
        let target_y = 35;

        // Créer un motif distinct dans le template
        for ty in 0..th {
            for tx in 0..tw {
                let t_idx = (ty * tw + tx) * 4;
                let r = (tx * 20 + 10) as u8;
                let g = (ty * 30 + 20) as u8;
                let b = 200u8;
                template_rgba[t_idx] = r;
                template_rgba[t_idx + 1] = g;
                template_rgba[t_idx + 2] = b;
                template_rgba[t_idx + 3] = 255;

                // Copier dans l'écran à la position cible en format BGRA
                let s_idx = ((target_y + ty) * sw + (target_x + tx)) * 4;
                screen_bgra[s_idx] = b;
                screen_bgra[s_idx + 1] = g;
                screen_bgra[s_idx + 2] = r;
                screen_bgra[s_idx + 3] = 255;
            }
        }

        let result = find_template_in_bgra(&screen_bgra, sw, sh, &template_rgba, tw, th, 5);
        assert_eq!(result, Some((target_x, target_y)));
    }

    #[test]
    fn test_find_template_in_bgra_bottom_right_corner() {
        let sw = 50;
        let sh = 40;
        let tw = 5;
        let th = 4;
        let mut screen_bgra = vec![50u8; sw * sh * 4];
        let mut template_rgba = vec![0u8; tw * th * 4];

        let target_x = sw - tw;
        let target_y = sh - th;

        for ty in 0..th {
            for tx in 0..tw {
                let t_idx = (ty * tw + tx) * 4;
                template_rgba[t_idx] = 10;
                template_rgba[t_idx + 1] = 220;
                template_rgba[t_idx + 2] = 80;
                template_rgba[t_idx + 3] = 255;

                let s_idx = ((target_y + ty) * sw + (target_x + tx)) * 4;
                screen_bgra[s_idx] = 80; // B
                screen_bgra[s_idx + 1] = 220; // G
                screen_bgra[s_idx + 2] = 10; // R
                screen_bgra[s_idx + 3] = 255;
            }
        }

        let result = find_template_in_bgra(&screen_bgra, sw, sh, &template_rgba, tw, th, 10);
        assert_eq!(result, Some((target_x, target_y)));
    }

    #[test]
    fn test_find_template_in_bgra_tolerance_and_no_match() {
        let sw = 20;
        let sh = 20;
        let tw = 4;
        let th = 4;
        let mut screen_bgra = vec![0u8; sw * sh * 4];
        let mut template_rgba = vec![0u8; tw * th * 4];

        // Template R=100, G=100, B=100
        for i in 0..(tw * th) {
            template_rgba[i * 4] = 100;
            template_rgba[i * 4 + 1] = 100;
            template_rgba[i * 4 + 2] = 100;
            template_rgba[i * 4 + 3] = 255;
        }

        // Écran à (5, 5) avec R=115, G=110, B=90 (diff max = 15)
        for ty in 0..th {
            for tx in 0..tw {
                let idx = ((5 + ty) * sw + (5 + tx)) * 4;
                screen_bgra[idx] = 90; // B
                screen_bgra[idx + 1] = 110; // G
                screen_bgra[idx + 2] = 115; // R
                screen_bgra[idx + 3] = 255;
            }
        }

        // Tolérance 10 : échec
        assert_eq!(
            find_template_in_bgra(&screen_bgra, sw, sh, &template_rgba, tw, th, 10),
            None
        );

        // Tolérance 20 : succès
        assert_eq!(
            find_template_in_bgra(&screen_bgra, sw, sh, &template_rgba, tw, th, 20),
            Some((5, 5))
        );
    }

    #[test]
    fn test_find_template_in_bgra_out_of_bounds() {
        let screen = vec![0u8; 100];
        let template = vec![0u8; 100];
        assert_eq!(
            find_template_in_bgra(&screen, 5, 5, &template, 10, 10, 25),
            None
        );
        assert_eq!(
            find_template_in_bgra(&screen, 5, 5, &template, 0, 5, 25),
            None
        );
        assert_eq!(
            find_template_in_bgra(&screen, 5, 5, &template, 5, 0, 25),
            None
        );
    }

    #[test]
    fn test_find_template_perf_1080p_and_4k() {
        // 1080p : 1920x1080
        let sw_1080 = 1920;
        let sh_1080 = 1080;
        let tw = 32;
        let th = 32;
        let mut screen_1080 = vec![30u8; sw_1080 * sh_1080 * 4];
        let mut template = vec![0u8; tw * th * 4];

        let target_x = 1200;
        let target_y = 750;

        for ty in 0..th {
            for tx in 0..tw {
                let t_idx = (ty * tw + tx) * 4;
                template[t_idx] = 200;
                template[t_idx + 1] = 100;
                template[t_idx + 2] = 50;
                template[t_idx + 3] = 255;

                let s_idx = ((target_y + ty) * sw_1080 + (target_x + tx)) * 4;
                screen_1080[s_idx] = 50;
                screen_1080[s_idx + 1] = 100;
                screen_1080[s_idx + 2] = 200;
                screen_1080[s_idx + 3] = 255;
            }
        }

        let start_1080 = Instant::now();
        let found_1080 =
            find_template_in_bgra(&screen_1080, sw_1080, sh_1080, &template, tw, th, 25);
        let elapsed_1080 = start_1080.elapsed();
        assert_eq!(found_1080, Some((target_x, target_y)));
        println!("Benchmark 1080p matching time: {:?}", elapsed_1080);
        assert!(
            elapsed_1080.as_millis() < 50,
            "1080p template matching should be ultra fast (< 50ms, target < 16ms), took {:?}",
            elapsed_1080
        );

        // 4K : 3840x2160
        let sw_4k = 3840;
        let sh_4k = 2160;
        let mut screen_4k = vec![30u8; sw_4k * sh_4k * 4];
        let target_4k_x = 2800;
        let target_4k_y = 1500;
        for ty in 0..th {
            for tx in 0..tw {
                let s_idx = ((target_4k_y + ty) * sw_4k + (target_4k_x + tx)) * 4;
                screen_4k[s_idx] = 50;
                screen_4k[s_idx + 1] = 100;
                screen_4k[s_idx + 2] = 200;
                screen_4k[s_idx + 3] = 255;
            }
        }

        let start_4k = Instant::now();
        let found_4k = find_template_in_bgra(&screen_4k, sw_4k, sh_4k, &template, tw, th, 25);
        let elapsed_4k = start_4k.elapsed();
        assert_eq!(found_4k, Some((target_4k_x, target_4k_y)));
        println!("Benchmark 4K matching time: {:?}", elapsed_4k);
    }

    #[test]
    fn test_stop_playback_flag_atomic_reactivity() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!flag.load(Ordering::Relaxed));

        flag.store(true, Ordering::SeqCst);
        assert!(flag.load(Ordering::Relaxed));

        flag.store(false, Ordering::SeqCst);
        assert!(!flag.load(Ordering::Relaxed));
    }

    #[test]
    fn test_rdev_key_static_names() {
        let (name_a, vk_a, is_ext_a) = rdev_key_to_name_and_scan(&RdevKey::KeyA);
        assert_eq!(name_a, "KeyA");
        assert_eq!(vk_a, 0x41);
        assert!(!is_ext_a);

        let (name_del, vk_del, is_ext_del) = rdev_key_to_name_and_scan(&RdevKey::Delete);
        assert_eq!(name_del, "Delete");
        assert_eq!(vk_del, 0x2E);
        assert!(is_ext_del);

        let (name_f4, vk_f4, _) = rdev_key_to_name_and_scan(&RdevKey::F4);
        assert_eq!(name_f4, "F4");
        assert_eq!(vk_f4, 0x73);

        let (name_f7, vk_f7, _) = rdev_key_to_name_and_scan(&RdevKey::F7);
        assert_eq!(name_f7, "F7");
        assert_eq!(vk_f7, 0x76);
    }

    #[test]
    fn test_toggle_playback_state_machine() {
        {
            let mut state = MACRO_STATE.lock().unwrap();
            state.is_playing = false;
            state.actions.clear();
        }

        // Sans actions, toggle_playback ne démarre pas la lecture
        toggle_playback();
        assert!(!MACRO_STATE.lock().unwrap().is_playing);

        // Avec une action valide
        add_action(MacroAction {
            action_type: ActionType::Wait(50),
            delay_ms: 0,
        });
        toggle_playback();
        assert!(MACRO_STATE.lock().unwrap().is_playing);

        // Deuxième toggle arrête la lecture
        toggle_playback();
        assert!(MACRO_STATE
            .lock()
            .unwrap()
            .stop_playback_flag
            .load(Ordering::Relaxed));

        // Nettoyage
        clear_actions();
        MACRO_STATE.lock().unwrap().is_playing = false;
    }

    #[cfg(windows)]
    #[test]
    fn test_raw_input_atomic_flags_consistency() {
        RAW_INPUT_RECORDING.store(false, Ordering::SeqCst);
        RIGHT_MOUSE_DOWN.store(false, Ordering::SeqCst);

        assert!(!RAW_INPUT_RECORDING.load(Ordering::Relaxed));
        assert!(!RIGHT_MOUSE_DOWN.load(Ordering::Relaxed));

        RAW_INPUT_RECORDING.store(true, Ordering::SeqCst);
        RIGHT_MOUSE_DOWN.store(true, Ordering::SeqCst);

        assert!(RAW_INPUT_RECORDING.load(Ordering::Relaxed));
        assert!(RIGHT_MOUSE_DOWN.load(Ordering::Relaxed));

        // Reset
        RAW_INPUT_RECORDING.store(false, Ordering::SeqCst);
        RIGHT_MOUSE_DOWN.store(false, Ordering::SeqCst);
    }

    #[cfg(windows)]
    #[test]
    fn test_get_screen_capture_bounds_positive_dimensions() {
        let (x, y, w, h) = get_screen_capture_bounds();
        let _ = (x, y);
        assert!(w > 0, "Largeur de capture d'écran doit être positive");
        assert!(h > 0, "Hauteur de capture d'écran doit être positive");
    }

    #[test]
    fn test_window_lock_config_defaults_and_mutation() {
        let default_cfg = WindowLockConfig::default();
        assert!(!default_cfg.enabled);
        assert_eq!(default_cfg.width, 1280);
        assert_eq!(default_cfg.height, 720);
        assert_eq!(default_cfg.x, 100);
        assert_eq!(default_cfg.y, 100);
        assert!(default_cfg.force_foreground);
        assert!(default_cfg.restore_if_maximized);
        assert!(!default_cfg.embed_in_macroforge);
        assert!(default_cfg.lock_window_styles);
        assert!(default_cfg.enforce_continuous_clamp);

        let custom = WindowLockConfig {
            enabled: true,
            title_filter: "Game Window".to_string(),
            x: 50,
            y: 50,
            width: 1920,
            height: 1080,
            force_foreground: true,
            restore_if_maximized: false,
            embed_in_macroforge: true,
            lock_window_styles: true,
            enforce_continuous_clamp: true,
        };

        set_window_lock(custom.clone());
        let fetched = get_window_lock();
        assert_eq!(fetched, custom);

        // Serialization test
        let json = serde_json::to_string(&custom).expect("serialize WindowLockConfig");
        let deserialized: WindowLockConfig =
            serde_json::from_str(&json).expect("deserialize WindowLockConfig");
        assert_eq!(deserialized, custom);

        // Backward compatibility deserialization test without new fields
        let old_json = r#"{"enabled":true,"title_filter":"Old","x":10,"y":20,"width":800,"height":600,"force_foreground":false,"restore_if_maximized":false}"#;
        let old_deserialized: WindowLockConfig =
            serde_json::from_str(old_json).expect("deserialize old json");
        assert!(!old_deserialized.embed_in_macroforge);
        assert!(old_deserialized.lock_window_styles);
        assert!(old_deserialized.enforce_continuous_clamp);

        // Reset
        set_window_lock(default_cfg);
    }

    #[test]
    fn test_embedded_viewport_tracking_and_query() {
        // Test updating viewport bounds
        update_embedded_viewport_bounds(120, 240, 640, 480, true);
        {
            let vp = EMBEDDED_VIEWPORT.lock().unwrap();
            assert_eq!(*vp, Some((120, 240, 640, 480, true)));
        }

        // Test hiding viewport
        update_embedded_viewport_bounds(0, 0, 0, 0, false);
        {
            let vp = EMBEDDED_VIEWPORT.lock().unwrap();
            assert_eq!(*vp, Some((0, 0, 0, 0, false)));
        }

        // Nettoyage
        *EMBEDDED_VIEWPORT.lock().unwrap() = None;
    }
}
