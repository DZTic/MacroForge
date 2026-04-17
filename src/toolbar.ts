import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { initGlobalFeatures, t } from "./utils";

const btnRecord = document.getElementById("btn-record") as HTMLButtonElement;
const btnPlay = document.getElementById("btn-play") as HTMLButtonElement;
const btnStop = document.getElementById("btn-stop") as HTMLButtonElement;
const btnEdit = document.getElementById("btn-edit") as HTMLButtonElement;
const btnClose = document.getElementById("btn-close") as HTMLButtonElement;
const dragHandle = document.querySelector(".drag-handle") as HTMLDivElement;
const recBadge = document.getElementById("rec-badge") as HTMLDivElement;
const playbackStatus = document.getElementById("playback-status") as HTMLDivElement;

let isRecording = false;
let showProgressSetting = localStorage.getItem("show-action-progress") !== "false";

interface PlaybackActionPayload {
    index: number;
    total: number;
    action_type: string;
    detail: string;
}

// UI synchronize function
function updateRecordingUI(recording: boolean) {
    isRecording = recording;
    if (recording) {
        btnRecord.innerHTML = '<div class="square white"></div>';
        btnRecord.classList.replace("danger", "warning");
        recBadge.classList.add("visible");
        btnEdit.style.display = "none";
        btnClose.style.display = "none";
    } else {
        btnRecord.innerHTML = '<div class="circle red"></div>';
        btnRecord.classList.replace("warning", "danger");
        recBadge.classList.remove("visible");
        btnEdit.style.display = "flex";
        btnClose.style.display = "flex";
    }
}

listen<boolean>("recording-state-changed", (event) => {
    updateRecordingUI(event.payload);
});

listen<{ showProgress: boolean }>("settings-changed", (event) => {
    showProgressSetting = event.payload.showProgress;
    const playbackStatus = document.getElementById("playback-status");
    if (playbackStatus) {
        if (!showProgressSetting) {
            playbackStatus.classList.remove("visible");
        }
    }
});

listen<boolean>("playback-state-changed", (event) => {
    if (event.payload) {
        btnPlay.style.opacity = "0.5";
        btnPlay.dataset.tooltip = t("btn_playing");
        if (showProgressSetting) {
            playbackStatus.classList.add("visible");
        }
        const idxEl = document.getElementById("playback-index");
        if (idxEl) idxEl.innerText = "1";
    } else {
        btnPlay.style.opacity = "1";
        btnPlay.dataset.tooltip = t("btn_play").replace(/<[^>]+>/g, "").trim(); // Strip icon for tooltip
        playbackStatus.classList.remove("visible");
    }
});

listen<PlaybackActionPayload>("playback-action", (event) => {
    const { index, total } = event.payload;
    const playbackIndex = document.getElementById("playback-index");
    const playbackTotal = document.getElementById("playback-total");
    
    if (playbackIndex && playbackIndex.innerText !== index.toString()) {
        playbackIndex.innerText = index.toString();
        // Trigger animation
        playbackIndex.classList.remove("animating");
        void playbackIndex.offsetWidth; // Trigger reflow
        playbackIndex.classList.add("animating");
    }
    
    if (playbackTotal && playbackTotal.innerText !== total.toString()) {
        playbackTotal.innerText = total.toString();
    }
});

// Enable robust native window dragging for the custom toolbar
dragHandle.addEventListener("mousedown", async (e) => {
    if (e.button === 0) {
        try {
            await getCurrentWindow().startDragging();
        } catch (err) {
            console.error(err);
        }
    }
});

btnRecord.addEventListener("click", async () => {
    if (!isRecording) {
        await invoke("start_macro_recording");
        updateRecordingUI(true);
    } else {
        await invoke("stop_macro_recording");
        updateRecordingUI(false);
    }
});

btnPlay.addEventListener("click", async () => {
    await invoke("play_macro_command");
});

btnStop.addEventListener("click", async () => {
    if (isRecording) {
        await invoke("stop_macro_recording");
        updateRecordingUI(false);
    }
    await invoke("stop_macro_playback");
});

btnEdit.addEventListener("click", async () => {
    console.log("Tentative d'ouverture de l'éditeur (main window)...");
    try {
        await invoke("show_main_window");
    } catch (err) {
        console.error("Erreur lors de l'appel à show_main_window:", err);
    }
});

btnClose.addEventListener("click", async () => {
    await invoke("close_toolbar");
});

window.addEventListener("DOMContentLoaded", () => {
    initGlobalFeatures();
});
