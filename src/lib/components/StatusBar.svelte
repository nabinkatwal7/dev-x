<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type DisplayMode = "clock" | "countdown" | "network" | "memory" | "cpu" | "disk" | "uptime";

  let mode: DisplayMode = "clock";
  let displayText = "";
  let targetHour = 18;
  let prevRx = 0;
  let prevTx = 0;
  let prevTime = 0;

  let cachedMetrics: Record<string, unknown> | null = null;

  const modes: DisplayMode[] = ["clock", "countdown", "network", "memory", "cpu", "disk", "uptime"];
  let modeIndex = 0;
  let cycleInterval: ReturnType<typeof setInterval>;
  let tickInterval: ReturnType<typeof setInterval>;
  let metricsInterval: ReturnType<typeof setInterval>;

  function pad(n: number): string {
    return n.toString().padStart(2, "0");
  }

  function num(v: unknown, fallback: number): number {
    if (typeof v === "number" && !isNaN(v) && isFinite(v)) return v;
    return fallback;
  }

  async function refreshMetrics() {
    try {
      cachedMetrics = await invoke("get_system_metrics");
    } catch { /* keep stale cache */ }
  }

  function updateClock() {
    const now = new Date();
    displayText = `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`;
  }

  function updateCountdown() {
    const now = new Date();
    let target = new Date(now);
    target.setHours(targetHour, 0, 0, 0);
    if (now >= target) target.setDate(target.getDate() + 1);
    const diffMs = target.getTime() - now.getTime();
    const hours = Math.floor(diffMs / 3600000);
    const mins = Math.floor((diffMs % 3600000) / 60000);
    const secs = Math.floor((diffMs % 60000) / 1000);
    if (hours > 0) {
      displayText = `${hours}h ${pad(mins)}m to ${pad(targetHour)}:00`;
    } else {
      displayText = `${pad(mins)}:${pad(secs)} to ${pad(targetHour)}:00`;
    }
  }

  function updateNetwork() {
    const m = cachedMetrics;
    if (!m) { displayText = "net --"; return; }
    const now = Date.now();
    const rx = num(m.networkRxBytes, 0);
    const tx = num(m.networkTxBytes, 0);
    if (prevTime > 0) {
      const elapsed = (now - prevTime) / 1000;
      if (elapsed > 0) {
        const rxSpeed = ((rx - prevRx) / elapsed) / 1024;
        const txSpeed = ((tx - prevTx) / elapsed) / 1024;
        displayText = `↓${formatSpeed(rxSpeed)} ↑${formatSpeed(txSpeed)}`;
      }
    } else {
      displayText = "net --";
    }
    prevRx = rx;
    prevTx = tx;
    prevTime = now;
  }

  function updateMemory() {
    const m = cachedMetrics;
    if (!m) { displayText = "RAM --"; return; }
    const used = num(m.memoryUsedGb, 0);
    const total = num(m.memoryTotalGb, 0);
    displayText = `RAM ${used.toFixed(1)}/${total.toFixed(1)}GB`;
  }

  function updateCpu() {
    const m = cachedMetrics;
    if (!m) { displayText = "CPU --"; return; }
    const cpu = num(m.cpuPercent, 0);
    const proc = num(m.processCount, 0);
    displayText = `CPU ${cpu.toFixed(1)}%  (${proc} proc)`;
  }

  function updateDisk() {
    const m = cachedMetrics;
    if (!m) { displayText = "DISK --"; return; }
    const used = num(m.diskUsedGb, 0);
    const total = num(m.diskTotalGb, 0);
    displayText = `DISK ${used.toFixed(1)}/${total.toFixed(1)}GB`;
  }

  function updateUptime() {
    const m = cachedMetrics;
    if (!m) { displayText = "up --"; return; }
    const secs = num(m.uptimeSecs, 0);
    const d = Math.floor(secs / 86400);
    const h = Math.floor((secs % 86400) / 3600);
    const min = Math.floor((secs % 3600) / 60);
    if (d > 0) {
      displayText = `${d}d ${h}h ${pad(min)}m up`;
    } else if (h > 0) {
      displayText = `${h}h ${pad(min)}m up`;
    } else {
      displayText = `${pad(min)}m up`;
    }
  }

  function formatSpeed(kbps: number): string {
    if (!isFinite(kbps) || kbps < 0) return "0K";
    if (kbps >= 1024) return `${(kbps / 1024).toFixed(1)}M`;
    return `${Math.round(kbps)}K`;
  }

  function cycleMode() {
    modeIndex = (modeIndex + 1) % modes.length;
    mode = modes[modeIndex];
    renderCurrent();
  }

  function openApp() {
    invoke("show_overlay").catch(() => {});
  }

  function renderCurrent() {
    switch (mode) {
      case "clock": updateClock(); break;
      case "countdown": updateCountdown(); break;
      case "network": updateNetwork(); break;
      case "memory": updateMemory(); break;
      case "cpu": updateCpu(); break;
      case "disk": updateDisk(); break;
      case "uptime": updateUptime(); break;
    }
  }

  onMount(() => {
    refreshMetrics().then(() => renderCurrent());
    tickInterval = setInterval(renderCurrent, 1000);
    metricsInterval = setInterval(refreshMetrics, 2000);
    cycleInterval = setInterval(cycleMode, 5000);
  });

  onDestroy(() => {
    if (tickInterval) clearInterval(tickInterval);
    if (metricsInterval) clearInterval(metricsInterval);
    if (cycleInterval) clearInterval(cycleInterval);
  });
</script>

<div
  class="status-bar"
  on:click={openApp}
  on:contextmenu|preventDefault={cycleMode}
  on:keydown={(e) => e.key === "Enter" && openApp()}
  role="button"
  tabindex="0"
>
  <svg class="icon" viewBox="0 0 16 16" fill="none" width="14" height="14">
    <rect x="1" y="3" width="14" height="10" rx="2" stroke="currentColor" stroke-width="1.3"/>
    <rect x="3" y="6" width="2" height="5" rx="0.5" fill="#38BDF8"/>
    <rect x="6" y="5" width="2" height="6" rx="0.5" fill="#38BDF8"/>
    <rect x="9" y="4" width="2" height="7" rx="0.5" fill="#38BDF8"/>
    <rect x="12" y="7" width="2" height="4" rx="0.5" fill="#38BDF8"/>
  </svg>
  <span class="text">{displayText}</span>
</div>

<style>
  .status-bar {
    -webkit-app-region: drag;
    user-select: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 100%;
    width: 100%;
    background: rgba(15, 19, 26, 0.88);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(56, 189, 248, 0.35);
    border-radius: 20px;
    padding: 0 16px;
    box-sizing: border-box;
    box-shadow:
      0 0 6px rgba(56, 189, 248, 0.15),
      0 0 16px rgba(56, 189, 248, 0.08),
      inset 0 1px 0 rgba(56, 189, 248, 0.06);
    transition: box-shadow 0.2s;
  }
  .status-bar:hover {
    box-shadow:
      0 0 10px rgba(56, 189, 248, 0.3),
      0 0 24px rgba(56, 189, 248, 0.12),
      inset 0 1px 0 rgba(56, 189, 248, 0.1);
  }
  .icon {
    flex-shrink: 0;
    color: #38BDF8;
    opacity: 0.85;
  }
  .text {
    font-family: "Fira Code", "SF Mono", "Consolas", monospace;
    font-size: 12px;
    font-weight: 500;
    color: #F1F5F9;
    letter-spacing: 0.02em;
    white-space: nowrap;
  }
</style>
