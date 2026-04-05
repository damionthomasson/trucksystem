<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface TelemetryPayload {
  sdkActive: boolean;
  speed: number;
  gear: number;
  fuel: number;
  fuelRange: number;
  fuelAvgCons: number;
  odometer: number;
}

const telemetry = ref<TelemetryPayload>({
  sdkActive: false,
  speed: 0,
  gear: 0,
  fuel: 0,
  fuelRange: 0,
  fuelAvgCons: 0,
  odometer: 0,
});

const displayRange = computed(() => {
  const r = telemetry.value.fuelRange;
  if (!telemetry.value.sdkActive || r <= 5) return "---";
  return Math.round(r).toLocaleString();
});

const displayAvg = computed(() => {
  const avg = telemetry.value.fuelAvgCons;
  const speed = telemetry.value.speed;
  if (!telemetry.value.sdkActive || speed < 1) return "0.0";
  return avg > 199 ? "---" : avg.toFixed(1);
});

const gearDisplay = computed(() => {
  const g = telemetry.value.gear;
  if (g > 0) return g;
  if (g < 0) return 'R' + Math.abs(g); 
  return 'N';
});

onMounted(async () => {
  await invoke("start_telemetry");
  await listen<TelemetryPayload>("telemetry-update", (event) => {
    telemetry.value = event.payload;
  });
});
</script>

<template>
  <div class="app-container">
    <header class="hud-header">
      <div class="system-status">
        <div :class="['status-light', telemetry.sdkActive ? 'active' : 'inactive']"></div>
        <span class="status-text">{{ telemetry.sdkActive ? 'DIAGNOSTICS ONLINE' : 'SYSTEM LINK OFFLINE' }}</span>
      </div>
      <div class="odometer-display">
        <span class="label">ODO</span>
        <span class="value">{{ Math.round(telemetry.odometer || 0).toLocaleString() }} km</span>
      </div>
    </header>

    <main class="dashboard-core">
      <div class="main-gauge">
        <div class="gauge-ring"></div>
        <div class="speed-content">
          <div class="speed-number">{{ Math.round(telemetry.speed) }}</div>
          <div class="speed-unit">km/h</div>
          <div class="gear-chip">{{ gearDisplay }}</div>
        </div>
      </div>

      <section class="info-card" v-if="telemetry.sdkActive">
        <div class="data-grid">
          <div class="data-box">
            <span class="data-label">EST. RANGE</span>
            <span class="data-value highlight">{{ displayRange }} <small>KM</small></span>
          </div>
          <div class="data-box">
            <span class="data-label">CONS. AVG</span>
            <span class="data-value">{{ displayAvg }} <small>L/100</small></span>
          </div>
        </div>

        <div class="fuel-section">
          <div class="fuel-header">
            <span>FUEL SYSTEM</span>
            <span>{{ Math.round(telemetry.fuel || 0) }}%</span>
          </div>
          <div class="fuel-track">
            <div 
              class="fuel-level" 
              :style="{ 
                width: (telemetry.fuel || 0) + '%', 
                backgroundColor: telemetry.fuel < 15 ? '#ff3e3e' : '#00d4ff' 
              }"
            ></div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<style>
@import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@400;700&family=Rajdhani:wght@500;700&display=swap');

:root {
  --bg-dark: #0a0b0d;
  --glass-panel: rgba(255, 255, 255, 0.03);
  --accent-cyan: #00d4ff;
  --accent-amber: #ffcc00;
  --text-muted: #626a73;
}

body, html {
  margin: 0; padding: 0;
  background-color: var(--bg-dark);
  color: #fff;
  font-family: 'Rajdhani', sans-serif;
  overflow: hidden;
  letter-spacing: 1px;
}

.app-container {
  height: 100vh;
  padding: 20px;
  display: flex;
  flex-direction: column;
  background: radial-gradient(circle at center, #1a1c20 0%, #0a0b0d 100%);
}

/* Header UI */
.hud-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 20px;
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.system-status { display: flex; align-items: center; gap: 10px; }
.status-light { width: 8px; height: 8px; border-radius: 50%; transition: 0.3s; }
.status-light.active { background: var(--accent-cyan); box-shadow: 0 0 10px var(--accent-cyan); }
.status-light.inactive { background: #ff3e3e; }
.status-text { font-size: 0.75rem; font-weight: bold; color: var(--text-muted); }

.odometer-display .label { font-size: 0.7rem; color: var(--text-muted); margin-right: 8px; }
.odometer-display .value { font-family: 'Orbitron', sans-serif; font-size: 0.9rem; }

/* Gauge UI */
.dashboard-core {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.main-gauge {
  position: relative;
  width: 320px;
  height: 320px;
  display: flex;
  justify-content: center;
  align-items: center;
}

.gauge-ring {
  position: absolute;
  width: 100%;
  height: 100%;
  border: 2px solid rgba(255,255,255,0.05);
  border-top-color: var(--accent-cyan);
  border-radius: 50%;
  filter: drop-shadow(0 0 15px rgba(0, 212, 255, 0.2));
  mask-image: linear-gradient(to bottom, black 80%, transparent 100%);
}

.speed-content { text-align: center; }
.speed-number {
  font-family: 'Orbitron', sans-serif;
  font-size: 7rem;
  font-weight: 700;
  line-height: 1;
  background: linear-gradient(to bottom, #fff 60%, var(--text-muted));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}
.speed-unit { font-size: 1.2rem; color: var(--text-muted); font-weight: bold; margin-top: -10px; }

.gear-chip {
  margin-top: 15px;
  background: rgba(255,255,255,0.05);
  padding: 5px 20px;
  border-radius: 4px;
  font-size: 1.5rem;
  font-family: 'Orbitron';
  border: 1px solid rgba(255,255,255,0.1);
}

/* Info Panel */
.info-card {
  margin-top: 30px;
  width: 380px;
  background: var(--glass-panel);
  backdrop-filter: blur(10px);
  padding: 25px;
  border-radius: 20px;
  border: 1px solid rgba(255,255,255,0.05);
}

.data-grid { display: flex; justify-content: space-between; margin-bottom: 25px; }
.data-box { display: flex; flex-direction: column; }
.data-label { font-size: 0.7rem; color: var(--text-muted); font-weight: bold; margin-bottom: 5px; }
.data-value { font-family: 'Orbitron'; font-size: 1.8rem; }
.data-value small { font-size: 0.8rem; color: var(--text-muted); }
.highlight { color: var(--accent-cyan); }

/* Fuel UI */
.fuel-header {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  font-weight: bold;
  color: var(--text-muted);
  margin-bottom: 8px;
}
.fuel-track {
  height: 6px;
  background: rgba(255,255,255,0.05);
  border-radius: 3px;
  overflow: hidden;
}
.fuel-level {
  height: 100%;
  transition: width 1s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
}
</style>