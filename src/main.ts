import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save, message } from "@tauri-apps/plugin-dialog";

let btnRecord: HTMLButtonElement;
let btnPlay: HTMLButtonElement;
let btnSave: HTMLButtonElement;
let btnLoad: HTMLButtonElement;
let actionListEl: HTMLElement;
let badgeCountEl: HTMLElement;
let checkLoop: HTMLInputElement;

let isRecording = false;
let currentActions: any[] = [];

// --- Mouse-based drag & drop (bypasses WebView2/Tauri native drag interception) ---
let dragState: {
    active: boolean;
    sourceIndex: number;
    ghost: HTMLElement | null;
    offsetY: number;
    currentTarget: number | null;
} = { active: false, sourceIndex: -1, ghost: null, offsetY: 0, currentTarget: null };

function initMouseDnd() {
    actionListEl.addEventListener('mousedown', (e: MouseEvent) => {
        const handle = (e.target as HTMLElement).closest('.drag-icon');
        if (!handle) return;
        const item = (e.target as HTMLElement).closest('.action-item') as HTMLElement;
        if (!item) return;

        e.preventDefault();
        const idx = parseInt(item.getAttribute('data-index') || '0', 10);
        const rect = item.getBoundingClientRect();

        // Create ghost element
        const ghost = item.cloneNode(true) as HTMLElement;
        ghost.style.cssText = `
            position: fixed;
            width: ${rect.width}px;
            left: ${rect.left}px;
            top: ${rect.top}px;
            opacity: 0.75;
            pointer-events: none;
            z-index: 9999;
            border: 1px dashed var(--accent);
            background: rgba(59,130,246,0.15);
            border-radius: 8px;
            box-sizing: border-box;
        `;
        document.body.appendChild(ghost);

        item.style.opacity = '0.3';

        dragState = {
            active: true,
            sourceIndex: idx,
            ghost,
            offsetY: e.clientY - rect.top,
            currentTarget: null,
        };
    });

    document.addEventListener('mousemove', (e: MouseEvent) => {
        if (!dragState.active || !dragState.ghost) return;
        e.preventDefault();

        dragState.ghost.style.top = `${e.clientY - dragState.offsetY}px`;

        // Find target item under cursor
        dragState.ghost.style.display = 'none';
        const el = document.elementFromPoint(e.clientX, e.clientY);
        dragState.ghost.style.display = '';

        const targetItem = el ? (el as HTMLElement).closest('.action-item') as HTMLElement : null;

        // Clear previous indicators
        actionListEl.querySelectorAll('.action-item').forEach((n: any) => {
            n.style.borderTop = '';
            n.style.borderBottom = '';
        });

        if (targetItem) {
            const targetIdx = parseInt(targetItem.getAttribute('data-index') || '0', 10);
            if (targetIdx !== dragState.sourceIndex) {
                const rect = targetItem.getBoundingClientRect();
                if (e.clientY < rect.top + rect.height / 2) {
                    targetItem.style.borderTop = '2px solid var(--accent)';
                    dragState.currentTarget = targetIdx;
                } else {
                    targetItem.style.borderBottom = '2px solid var(--accent)';
                    dragState.currentTarget = targetIdx + 0.5; // after this item
                }
            }
        }
    });

    document.addEventListener('mouseup', async () => {
        if (!dragState.active) return;

        // Cleanup ghost
        if (dragState.ghost) {
            dragState.ghost.remove();
            dragState.ghost = null;
        }

        // Restore opacity
        const sourceItem = actionListEl.querySelector(`[data-index="${dragState.sourceIndex}"]`) as HTMLElement;
        if (sourceItem) sourceItem.style.opacity = '';

        // Clear indicators
        actionListEl.querySelectorAll('.action-item').forEach((n: any) => {
            n.style.borderTop = '';
            n.style.borderBottom = '';
        });

        const { sourceIndex, currentTarget } = dragState;
        dragState = { active: false, sourceIndex: -1, ghost: null, offsetY: 0, currentTarget: null };

        if (currentTarget === null) return;

        // currentTarget is targetIdx (before) or targetIdx + 0.5 (after)
        const reordered = [...currentActions];
        const [item] = reordered.splice(sourceIndex, 1);
        // Insertion position after removing source
        let pos = currentTarget > sourceIndex
            ? Math.ceil(currentTarget) - 1
            : Math.floor(currentTarget);
        pos = Math.max(0, Math.min(pos, reordered.length));
        if (pos === sourceIndex) return; // no change

        reordered.splice(pos, 0, item);
        currentActions = reordered;
        renderActions(currentActions);
        await invoke("set_macro_actions", { actions: currentActions });
    });
}

window.addEventListener('DOMContentLoaded', async () => {
    btnRecord = document.getElementById("btn-record") as HTMLButtonElement;
    btnPlay = document.getElementById("btn-play") as HTMLButtonElement;
    btnSave = document.getElementById("btn-save") as HTMLButtonElement;
    btnLoad = document.getElementById("btn-load") as HTMLButtonElement;
    actionListEl = document.getElementById("action-list") as HTMLElement;
    badgeCountEl = document.getElementById("badge-count") as HTMLElement;
    checkLoop = document.getElementById("check-loop") as HTMLInputElement;

    if (!actionListEl) return;

    initMouseDnd();

    // Jump-to-action bar
    const jumpInput  = document.getElementById("jump-input")  as HTMLInputElement;
    const jumpBtn    = document.getElementById("jump-btn")    as HTMLButtonElement;
    const jumpResult = document.getElementById("jump-result") as HTMLSpanElement;
    const doJump = () => {
        const n = parseInt(jumpInput.value, 10);
        if (isNaN(n) || n < 1 || n > currentActions.length) {
            jumpResult.textContent = `Hors plage (1–${currentActions.length})`;
            return;
        }
        jumpResult.textContent = '';
        scrollToAction(n - 1, true);
    };
    jumpBtn.addEventListener("click", doJump);
    jumpInput.addEventListener("keydown", (e) => { if (e.key === 'Enter') doJump(); });

    btnRecord.addEventListener("click", async () => {
        if (!isRecording) {
            await invoke("start_macro_recording");
            updateMainRecordingUI(true);
        } else {
            await invoke("stop_macro_recording");
            updateMainRecordingUI(false);
        }
    });

    // Initialiser l'état de la boucle depuis le backend
    const initialLoop = await invoke("get_loop_playback");
    checkLoop.checked = initialLoop as boolean;

    checkLoop.addEventListener("change", async () => {
        await invoke("set_loop_playback", { looping: checkLoop.checked });
    });

    const checkShowMouse = document.getElementById("check-show-mouse") as HTMLInputElement;
    checkShowMouse.addEventListener("change", () => {
        renderActions(currentActions);
    });

    btnPlay.addEventListener("click", async () => {
        await invoke("play_macro_command");
    });

    btnSave.addEventListener("click", async () => {
        const path = await save({
            filters: [{ name: 'MacroForge', extensions: ['mforge'] }]
        });
        if (path) {
            try {
                await invoke("save_macro", { path });
                await message("Macro sauvegardée avec succès!", { title: "Succès", kind: "info" });
            } catch (e) {
                await message("Erreur de sauvegarde: " + e, { title: "Erreur", kind: "error" });
            }
        }
    });

    btnLoad.addEventListener("click", async () => {
        const path = await open({
            multiple: false,
            filters: [{ name: 'MacroForge', extensions: ['mforge', 'json'] }]
        });
        if (path && typeof path === "string") {
            try {
                await invoke("load_macro", { path });
                await refreshActions();
                await message("Macro chargée avec succès!", { title: "Succès", kind: "info" });
            } catch (e) {
                await message("Erreur de chargement: " + e, { title: "Erreur", kind: "error" });
            }
        }
    });

    await refreshActions();

    // Manual actions listeners
    setupManualActionListeners();
});

let keyCaptureAbortController: AbortController | null = null;

function setupManualActionListeners() {
    // Buttons
    document.getElementById("btn-add-key")?.addEventListener("click", () => {
        showModal("modal-key");
        startKeyCapture();
    });
    document.getElementById("btn-add-mouse")?.addEventListener("click", () => showModal("modal-mouse"));
    document.getElementById("btn-add-wait")?.addEventListener("click", () => showModal("modal-wait"));
    document.getElementById("btn-add-image")?.addEventListener("click", addImageAction);

    // Cancel buttons
    document.getElementById("btn-key-cancel")?.addEventListener("click", () => {
        stopKeyCapture();
        hideModal("modal-key");
    });
    document.getElementById("btn-mouse-cancel")?.addEventListener("click", () => hideModal("modal-mouse"));
    document.getElementById("btn-wait-cancel")?.addEventListener("click", () => hideModal("modal-wait"));

    // Mouse Confirm
    document.getElementById("btn-mouse-confirm")?.addEventListener("click", async () => {
        const inputX = document.getElementById("input-mouse-x") as HTMLInputElement;
        const inputY = document.getElementById("input-mouse-y") as HTMLInputElement;
        const x = parseInt(inputX.value) || 0;
        const y = parseInt(inputY.value) || 0;
        const action = {
            action_type: { MousePress: [1, x, y] },
            delay_ms: 100
        };
        await manualAddAction(action);
        const release = {
            action_type: { MouseRelease: [1, x, y] },
            delay_ms: 50
        };
        await manualAddAction(release);
        inputX.value = "";
        inputY.value = "";
        hideModal("modal-mouse");
    });

    // Wait Confirm
    document.getElementById("btn-wait-confirm")?.addEventListener("click", async () => {
        const inputMs = document.getElementById("input-wait-ms") as HTMLInputElement;
        const ms = parseInt(inputMs.value) || 500;
        const action = {
            action_type: { Wait: ms },
            delay_ms: 0
        };
        await manualAddAction(action);
        inputMs.value = "";
        hideModal("modal-wait");
    });
}

function startKeyCapture() {
    keyCaptureAbortController = new AbortController();
    const display = document.getElementById("key-display");
    if (display) display.textContent = "---";

    window.addEventListener("keydown", async (e) => {
        e.preventDefault();
        e.stopPropagation();

        const key = e.key;
        const keyCode = e.keyCode;
        
        // Display the key
        if (display) display.textContent = key;

        // Determine if extended (roughly)
        // In rdev mapping: Up, Down, Left, Right, Home, End, PageUp, PageDown, Delete, Insert, Meta, AltGr are extended
        const extendedKeys = ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown", "Delete", "Insert", "Meta", "AltGraph"];
        const isExtended = extendedKeys.includes(key) || e.location === 2; // location 2 is usually Right side keys

        // Auto-add and close
        const action = {
            action_type: { KeyPress: [key, keyCode, isExtended] },
            delay_ms: 100
        };
        await manualAddAction(action);
        const release = {
            action_type: { KeyRelease: [key, keyCode, isExtended] },
            delay_ms: 50
        };
        await manualAddAction(release);

        // Feedback and close
        setTimeout(() => {
            stopKeyCapture();
            hideModal("modal-key");
        }, 300);

    }, { signal: keyCaptureAbortController.signal });
}

function stopKeyCapture() {
    if (keyCaptureAbortController) {
        keyCaptureAbortController.abort();
        keyCaptureAbortController = null;
    }
}

function showModal(id: string) {
    document.getElementById(id)?.classList.remove("hidden");
    const firstInput = document.getElementById(id)?.querySelector('input:not([type="hidden"])') as HTMLInputElement;
    if (firstInput) firstInput.focus();
}

function hideModal(id: string) {
    document.getElementById(id)?.classList.add("hidden");
}

async function manualAddAction(action: any) {
    currentActions.push(action);
    await invoke("set_macro_actions", { actions: currentActions });
    renderActions(currentActions);
}

async function addImageAction() {
    const path = await open({
        multiple: false,
        filters: [{ name: 'Image', extensions: ['png', 'jpg', 'jpeg'] }]
    });
    if (!path || typeof path !== "string") return;

    const timeoutStr = await promptTimeout("5000");
    if (timeoutStr === null) return;

    const timeout = parseInt(timeoutStr || "5000", 10);
    const action = {
        action_type: { WaitImage: [path, timeout] },
        delay_ms: 1000
    };
    await manualAddAction(action);
}

function updateMainRecordingUI(recording: boolean) {
    isRecording = recording;
    if (recording) {
        btnRecord.innerHTML = '<div class="square white"></div> Arrêter (F9)';
        btnRecord.classList.replace("record", "danger");
    } else {
        btnRecord.innerHTML = '<div class="circle red"></div> Enregistrer (F8)';
        btnRecord.classList.replace("danger", "record");
    }
    refreshActions();
}

listen<boolean>("recording-state-changed", (event) => {
    updateMainRecordingUI(event.payload);
});

listen<boolean>("playback-state-changed", (event) => {
    const badgeCurrent = document.getElementById("badge-current") as HTMLElement;
    if (event.payload) {
        btnPlay.innerText = "Lecture en cours... (F10 stop)";
        btnPlay.classList.add("playing-active");
        if (badgeCurrent) badgeCurrent.style.display = '';
    } else {
        btnPlay.innerHTML = '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M8 5V19L19 12L8 5Z"></path></svg> Jouer la Macro';
        btnPlay.classList.remove("playing-active");
        if (badgeCurrent) { badgeCurrent.style.display = 'none'; badgeCurrent.textContent = '\u25b6 #\u2014'; }
        // Retirer le surlignage en cours
        document.querySelectorAll('.action-item.playing').forEach(el => el.classList.remove('playing'));
    }
});

listen<{index:number, total:number, action_type:string, x:number, y:number, detail:string}>("playback-action", (event) => {
    const { index } = event.payload;
    const badgeCurrent = document.getElementById("badge-current") as HTMLElement;
    if (badgeCurrent) badgeCurrent.textContent = `\u25b6 #${index}`;
    scrollToAction(index - 1, false);
});

function scrollToAction(idx: number, highlight: boolean) {
    document.querySelectorAll('.action-item.playing').forEach(el => el.classList.remove('playing'));
    const el = actionListEl.querySelector(`[data-index="${idx}"]`) as HTMLElement | null;
    if (!el) return;
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    el.classList.add('playing');
    if (highlight) {
        setTimeout(() => el.classList.remove('playing'), 2000);
    }
}

currentActions = [];

setInterval(async () => {
    if (isRecording) {
        await refreshActions();
    }
}, 500);

(window as any).deleteAction = async (idx: number) => {
    currentActions.splice(idx, 1);
    await invoke("set_macro_actions", { actions: currentActions });
    renderActions(currentActions);
};

(window as any).editAction = async (idx: number) => {
    const a = currentActions[idx];
    let title = "Modifier l'action";
    let fields: { label: string, value: string }[] = [];

    // Préparation des champs selon le type
    if (a.action_type.MouseMove) {
        title = "Mouvement Absolu";
        fields = [
            { label: "Coordonnée X", value: a.action_type.MouseMove[0].toString() },
            { label: "Coordonnée Y", value: a.action_type.MouseMove[1].toString() }
        ];
    } 
    else if (a.action_type.MouseMoveRelative) {
        title = "Mouvement Relatif (FPS)";
        fields = [
            { label: "Décalage dX", value: a.action_type.MouseMoveRelative[0].toString() },
            { label: "Décalage dY", value: a.action_type.MouseMoveRelative[1].toString() }
        ];
    }
    else if (a.action_type.MousePress !== undefined) {
        title = "Clic Enfoncé";
        fields = [
            { label: "X du clic", value: a.action_type.MousePress[1].toString() },
            { label: "Y du clic", value: a.action_type.MousePress[2].toString() }
        ];
    }
    else if (a.action_type.MouseRelease !== undefined) {
        title = "Clic Relâché";
        fields = [
            { label: "X de relâchement", value: a.action_type.MouseRelease[1].toString() },
            { label: "Y de relâchement", value: a.action_type.MouseRelease[2].toString() }
        ];
    }
    else if (a.action_type.WaitImage) {
        title = "Recherche Image";
        fields = [
            { label: "Délai max d'attente (ms)", value: a.action_type.WaitImage[1].toString() }
        ];
    }
    else if (a.action_type.Wait !== undefined) {
        title = "Pause";
        fields = [
            { label: "Durée de la pause (ms)", value: a.action_type.Wait.toString() }
        ];
    }

    // On ajoute toujours le délai après l'action
    fields.push({ label: "Attente après action (ms)", value: a.delay_ms.toString() });

    // Affichage de la modale unique
    const results = await promptEditMulti(title, fields);
    if (!results) return; // Annulé

    // Application des résultats
    if (a.action_type.MouseMove) {
        a.action_type.MouseMove[0] = parseFloat(results[0]);
        a.action_type.MouseMove[1] = parseFloat(results[1]);
        a.delay_ms = parseInt(results[2], 10);
    } 
    else if (a.action_type.MouseMoveRelative) {
        a.action_type.MouseMoveRelative[0] = parseInt(results[0], 10);
        a.action_type.MouseMoveRelative[1] = parseInt(results[1], 10);
        a.delay_ms = parseInt(results[2], 10);
    }
    else if (a.action_type.MousePress !== undefined) {
        a.action_type.MousePress[1] = parseFloat(results[0]);
        a.action_type.MousePress[2] = parseFloat(results[1]);
        a.delay_ms = parseInt(results[2], 10);
    }
    else if (a.action_type.MouseRelease !== undefined) {
        a.action_type.MouseRelease[1] = parseFloat(results[0]);
        a.action_type.MouseRelease[2] = parseFloat(results[1]);
        a.delay_ms = parseInt(results[2], 10);
    }
    else if (a.action_type.WaitImage) {
        a.action_type.WaitImage[1] = parseInt(results[0], 10);
        a.delay_ms = parseInt(results[1], 10);
    }
    else if (a.action_type.Wait !== undefined) {
        a.action_type.Wait = parseInt(results[0], 10);
        a.delay_ms = parseInt(results[1], 10);
    } else {
        // Autres actions (KeyPress, etc) - juste le délai
        a.delay_ms = parseInt(results[0], 10);
    }

    // Sauvegarde et rafraîchissement
    await invoke("set_macro_actions", { actions: currentActions });
    renderActions(currentActions);
};

function promptEditMulti(title: string, fields: { label: string, value: string }[]): Promise<string[] | null> {
    return new Promise((resolve) => {
        const modal = document.getElementById("edit-modal") as HTMLDivElement;
        const btnOk = document.getElementById("edit-btn-ok") as HTMLButtonElement;
        const btnCancel = document.getElementById("edit-btn-cancel") as HTMLButtonElement;
        const h3 = document.getElementById("edit-title") as HTMLElement;
        
        h3.textContent = title;
        
        // Initialisation des champs
        for (let i = 1; i <= 3; i++) {
            const row = document.getElementById(`field-row-${i}`) as HTMLElement;
            const label = document.getElementById(`label-${i}`) as HTMLElement;
            const input = document.getElementById(`edit-input-${i}`) as HTMLInputElement;
            
            if (i <= fields.length) {
                row.classList.remove("hidden");
                label.textContent = fields[i-1].label;
                input.value = fields[i-1].value;
            } else {
                row.classList.add("hidden");
            }
        }
        
        modal.classList.remove("hidden");
        (document.getElementById("edit-input-1") as HTMLInputElement).focus();

        const cleanup = () => {
            modal.classList.add("hidden");
            btnOk.removeEventListener("click", onOk);
            btnCancel.removeEventListener("click", onCancel);
        };

        const onOk = () => {
            const res: string[] = [];
            for (let i = 1; i <= fields.length; i++) {
                res.push((document.getElementById(`edit-input-${i}`) as HTMLInputElement).value);
            }
            cleanup();
            resolve(res);
        };
        const onCancel = () => { cleanup(); resolve(null); };

        btnOk.addEventListener("click", onOk);
        btnCancel.addEventListener("click", onCancel);
    });
}

function promptTimeout(defaultVal: string = "5000", title?: string, desc?: string): Promise<string | null> {
    return new Promise((resolve) => {
        const modal = document.getElementById("timeout-modal") as HTMLDivElement;
        const input = document.getElementById("timeout-input") as HTMLInputElement;
        const btnOk = document.getElementById("modal-btn-ok") as HTMLButtonElement;
        const btnCancel = document.getElementById("modal-btn-cancel") as HTMLButtonElement;
        const h3 = modal.querySelector("h3") as HTMLElement;
        const p = modal.querySelector("p") as HTMLElement;

        if (title) h3.textContent = title;
        else h3.textContent = "Délai d'attente (ms)";
        
        if (desc) p.textContent = desc;
        else p.textContent = "Veuillez entrer le délai maximum d'attente pour l'image :";

        input.value = defaultVal;
        modal.classList.remove("hidden");
        input.focus();

        const cleanup = () => {
            modal.classList.add("hidden");
            btnOk.removeEventListener("click", onOk);
            btnCancel.removeEventListener("click", onCancel);
        };

        const onOk = () => { cleanup(); resolve(input.value); };
        const onCancel = () => { cleanup(); resolve(null); };

        btnOk.addEventListener("click", onOk);
        btnCancel.addEventListener("click", onCancel);
    });
}

function renderActions(actions: any[]) {
    const showMouse = (document.getElementById("check-show-mouse") as HTMLInputElement)?.checked ?? false;
    
    // On filtre les actions si nécessaire
    const visibleActions = showMouse 
        ? actions 
        : actions.filter(a => !a.action_type.MouseMove && !a.action_type.MouseMoveRelative);

    badgeCountEl.textContent = `${actions.length} action(s) (${visibleActions.length} visibles)`;

    if (visibleActions.length === 0) {
        actionListEl.innerHTML = `
        <div class="empty-state">
            <svg viewBox="0 0 24 24" fill="none"><path d="M12 8V12L15 15M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
            <p>Aucune action visibleifiée.</p>
            <span>${actions.length > 0 ? "Activez 'Mouvements Souris' pour les voir." : "Enregistrez une macro en appuyant sur F8."}</span>
        </div>`;
        return;
    }

    let html = '';
    // On limite à 1000 actions pour éviter le crash du navigateur quoi qu'il arrive
    const toRender = visibleActions.slice(0, 1000);
    
    // Map pour retrouver l'index original dans currentActions
    toRender.forEach((a) => {
        const originalIndex = actions.indexOf(a);
        let details = '';
        let typeStr = '';

        if (a.action_type.MouseMove) {
            typeStr = 'Move';
            details = `X: ${a.action_type.MouseMove[0].toFixed(0)}, Y: ${a.action_type.MouseMove[1].toFixed(0)}`;
        } else if (a.action_type.MouseMoveRelative) {
            typeStr = 'Move Rel';
            details = `dX: ${a.action_type.MouseMoveRelative[0]}, dY: ${a.action_type.MouseMoveRelative[1]}`;
        } else if (a.action_type.MousePress !== undefined) {
            typeStr = 'Click Down';
            details = `Bouton ${a.action_type.MousePress}`;
        } else if (a.action_type.MouseRelease !== undefined) {
            typeStr = 'Click Up';
            details = `Bouton ${a.action_type.MouseRelease}`;
        } else if (a.action_type.KeyPress !== undefined) {
            typeStr = 'Key Down';
            details = `Touche ${Array.isArray(a.action_type.KeyPress) ? a.action_type.KeyPress[0] : a.action_type.KeyPress}`;
        } else if (a.action_type.KeyRelease !== undefined) {
            typeStr = 'Key Up';
            details = `Touche ${Array.isArray(a.action_type.KeyRelease) ? a.action_type.KeyRelease[0] : a.action_type.KeyRelease}`;
        } else if (a.action_type.Scroll) {
            typeStr = 'Scroll';
            details = `dx: ${a.action_type.Scroll[0]}, dy: ${a.action_type.Scroll[1]}`;
        } else if (a.action_type.WaitImage) {
            typeStr = 'Wait Image';
            details = `Path: ${a.action_type.WaitImage[0]}, Timeout: ${a.action_type.WaitImage[1]}ms`;
        } else if (a.action_type.Wait !== undefined) {
            typeStr = 'Pause';
            details = `Durée: ${a.action_type.Wait}ms`;
        }

        html += `
        <div class="action-item" data-index="${originalIndex}">
            <div class="drag-icon" style="cursor:grab">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 9h8M8 15h8"/></svg>
            </div>
            <div class="action-line-num">${originalIndex + 1}</div>
            <div class="action-delay">${a.delay_ms} ms</div>
            <div class="action-type">${typeStr}</div>
            <div class="action-details">${details}</div>
            <div class="action-controls">
                <button class="edit-btn" onclick="window.editAction(${originalIndex})">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                </button>
                <button class="danger-text" onclick="window.deleteAction(${originalIndex})">✕</button>
            </div>
        </div>`;
    });

    if (visibleActions.length > 1000) {
        html += `<div style="text-align:center; padding:10px; color:var(--text-secondary); font-size:12px;">+ ${visibleActions.length - 1000} autres actions non affichées pour les performances</div>`;
    }

    actionListEl.innerHTML = html;
}

function refreshActions() {
    if (dragState.active) return;
    try {
        invoke<any[]>("get_macro_actions").then(actions => {
            currentActions = actions;
            renderActions(currentActions);
        });
    } catch (e) {
        console.error("Erreur lors de la récupération des actions", e);
    }
}

const btnOpenToolbar = document.getElementById("btn-open-toolbar") as HTMLButtonElement | null;
if (btnOpenToolbar) {
    btnOpenToolbar.addEventListener("click", async () => {
        await invoke("open_toolbar");
    });
}

window.addEventListener("focus", refreshActions);
refreshActions();
