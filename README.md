# MacroForge 🛠️

**MacroForge** is a high-performance macro engine for Windows, designed for precision, reliability, and FPS game support. Developed with **Tauri**, **Rust**, and **TypeScript**, it offers a robust solution for advanced automation.

![MacroForge](public/logo.png) <!-- Optional: if the logo exists -->

## ✨ Key Features

- 🌍 **Multilingual Support (i18n)**: Interface available in **French** and **English** with instant switching.
- ⏱️ **High Precision Timing**: Absolute timeline system to eliminate time drift, even over long durations.
- 🖱️ **Raw Input Mode (FPS)**: Captures mouse deltas directly via the Windows API, ideal for camera movements in games (Roblox, Minecraft, etc.).
- 🖼️ **Computer Vision (GDI/Rayon)**: Ultra-fast image detection with parallelized search for minimal latency.
- 🛑 **Visual Emergency Stop**: Define a "stop image" that immediately interrupts the macro if detected on screen.
- 🗔 **Floating Interface (Toolbar)**: A compact toolbar to control your macros while staying focused on your task.
- ↕️ **Intuitive Editing**: Reorganize actions via **Drag & Drop** and modify delays or coordinates with a single click.
- 📺 **Monitoring Overlay**: Transparent interface displaying the currently executing action on top of your applications.
- 💾 **Profile Management**: Export/Import your macros in `.mforge` (JSON) format.

## 🚀 Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (LTS recommended)
- Windows (required for low-level features)

### Dependency Installation

```bash
npm install
```

### Launch in Development Mode

```bash
npm run tauri dev
```

### Build the Executable

```bash
npm run tauri build
```

## 🛠️ Technical Stack

- **Backend**: Rust (Tauri v2)
  - `rdev`: Global input capture.
  - `winapi`: Raw Input, GDI, SendInput interactions.
  - `rayon`: Parallelized image processing.
- **Frontend**: TypeScript + Modern HTML/CSS
  - `Vite 6`: Ultra-fast bundling.
- **UI/UX**: Premium Design (Glassmorphism), smooth animations, and intelligent tooltip system.

## ⌨️ Hotkeys

- **F8**: Start / Stop recording.
- **F9**: Force stop recording.
- **F4**: Emergency stop of the currently playing macro.

## 📖 Usage

1. **Recording**: Press **F8** to start.
   - **FPS Note**: Hold **right-click** to capture relative movements (deltas) in games.
2. **Editing**: Use the main list to reorder actions (Drag & Drop) or delete/edit them.
3. **Toolbar**: Click "Toolbar" to switch to the floating mini-controller mode.
4. **Image Search**: Add an "Image" action to wait for a visual element to appear before continuing.
5. **Loops**: Enable "Loop" to repeat the macro indefinitely.

## ⚠️ Security and Ethics

MacroForge is a powerful tool. Please use it responsibly. Automation in some games may violate their terms of service.
