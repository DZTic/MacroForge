import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

const btnRecord = document.getElementById("btn-record") as HTMLButtonElement;
const btnPlay = document.getElementById("btn-play") as HTMLButtonElement;
const btnStop = document.getElementById("btn-stop") as HTMLButtonElement;
const btnEdit = document.getElementById("btn-edit") as HTMLButtonElement;
const btnClose = document.getElementById("btn-close") as HTMLButtonElement;
const dragHandle = document.querySelector(".drag-handle") as HTMLDivElement;
const recBadge = document.getElementById("rec-badge") as HTMLDivElement;
const playbackStatus = document.getElementById("playback-status") as HTMLDivElement;

let isRecording = false;

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

listen<boolean>("playback-state-changed", (event) => {
    if (event.payload) {
        btnPlay.style.opacity = "0.5";
        btnPlay.title = "Macro en cours...";
        playbackStatus.classList.add("visible");
    } else {
        btnPlay.style.opacity = "1";
        btnPlay.title = "Jouer la Macro";
        playbackStatus.classList.remove("visible");
        playbackStatus.innerText = "";
    }
});

listen<PlaybackActionPayload>("playback-action", (event) => {
    const { index, total } = event.payload;
    playbackStatus.innerText = `Action ${index} / ${total}`;
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
