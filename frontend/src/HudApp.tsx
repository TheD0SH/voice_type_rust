import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { buildShellStyle, normalizeTheme } from "./lib/appearance";
import { fallbackRuntime } from "./lib/options";
import type { RuntimeSnapshot } from "./types";
import { HudCard } from "./components/HudCard";

function clampUnit(value: number) {
  return Math.max(0, Math.min(1, value));
}

function smoothingFactor(deltaMs: number, durationMs: number) {
  return 1 - Math.exp(-deltaMs / durationMs);
}

export default function HudApp() {
  const [runtime, setRuntime] = useState<RuntimeSnapshot>(fallbackRuntime);

  // Refs for the animation loop — updated every frame without triggering
  // React re-renders.  The smoothed values are written to CSS custom
  // properties on the shell div so RuntimeMeter can read them.
  const shellRef = useRef<HTMLDivElement>(null);
  const audioLevelTargetRef = useRef(runtime.audioLevel);
  const runtimeStateRef = useRef(runtime.appState);
  const meterRef = useRef({ level: 0, peak: 0 });

  useEffect(() => {
    audioLevelTargetRef.current = runtime.audioLevel;
  }, [runtime.audioLevel]);

  useEffect(() => {
    runtimeStateRef.current = runtime.appState;
  }, [runtime.appState]);

  // Animation loop — drives the meter via CSS custom properties, NOT React
  // state.  This eliminates ~60 React re-renders/second during recording.
  useEffect(() => {
    let frameId = 0;
    let lastFrame = performance.now();

    const animate = (now: number) => {
      const deltaMs = Math.min(64, Math.max(8, now - lastFrame));
      lastFrame = now;

      const recording = runtimeStateRef.current === "recording";
      const target = recording ? clampUnit(audioLevelTargetRef.current) : 0;
      const current = meterRef.current;

      const levelFactor = smoothingFactor(deltaMs, target > current.level ? 110 : 320);
      const nextLevel = current.level + (target - current.level) * levelFactor;

      let nextPeak = current.peak;
      if (recording) {
        const peakTarget = Math.max(target, nextLevel);
        const peakFactor = smoothingFactor(deltaMs, peakTarget >= current.peak ? 64 : 520);
        nextPeak = current.peak + (peakTarget - current.peak) * peakFactor;
      } else {
        nextPeak = current.peak * Math.exp(-deltaMs / 180);
      }

      const level = nextLevel < 0.002 ? 0 : clampUnit(nextLevel);
      const peak = nextPeak < 0.006 ? 0 : clampUnit(nextPeak);
      meterRef.current = { level, peak };

      // Push values to CSS custom properties — no React re-render needed.
      const el = shellRef.current;
      if (el) {
        const processing = runtimeStateRef.current === "processing";
        const cssLevel = processing ? 0.22 : level;
        const cssPeak = processing ? 0.5 : peak;
        el.style.setProperty("--meter-level", String(cssLevel));
        el.style.setProperty("--meter-peak", String(cssPeak));
        el.style.setProperty(
          "--meter-active",
          recording || processing ? "1" : "0",
        );
      }

      frameId = window.requestAnimationFrame(animate);
    };

    frameId = window.requestAnimationFrame(animate);
    return () => window.cancelAnimationFrame(frameId);
  }, []);

  // Boot — subscribe to runtime events and fetch initial snapshot.
  useEffect(() => {
    let mounted = true;
    let unlistenRuntime: (() => void | Promise<void>) | null = null;

    const boot = async () => {
      try {
        unlistenRuntime = await listen<RuntimeSnapshot>("voice-type://runtime", (event) => {
          if (mounted) {
            setRuntime(event.payload);
          }
        });

        const snapshot = await invoke<RuntimeSnapshot>("get_runtime_snapshot");
        if (mounted) {
          setRuntime(snapshot);
        }
      } catch (error) {
        // Stay on fallback runtime — the HUD has no error UI to show.
        console.error("HUD boot failed:", error);
      }
    };

    void boot();

    return () => {
      mounted = false;
      if (unlistenRuntime) {
        void unlistenRuntime();
      }
    };
  }, []);

  // Load HUD background image if configured.
  const [backgroundUrl, setBackgroundUrl] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const path = runtime.config.hud_background_path.trim();

    if (!path) {
      setBackgroundUrl(null);
      return;
    }

    void invoke<string>("load_background_image_data_url", { path })
      .then((dataUrl) => {
        if (!cancelled) {
          setBackgroundUrl(dataUrl);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setBackgroundUrl(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [runtime.config.hud_background_path]);

  const theme = normalizeTheme(runtime.config.theme);
  const shellStyle = buildShellStyle(runtime.config, theme);
  const pinned = runtime.config.hud_pinned;

  // When pinned, mousedown on the HUD starts a native window drag.
  // After dragging ends, persist the new position.
  const handleMouseDown = async () => {
    if (!pinned) return;
    try {
      await getCurrentWindow().startDragging();
      // After drag completes, read and save the new position.
      const factor = await getCurrentWindow().scaleFactor();
      const pos = await getCurrentWindow().outerPosition();
      const x = Math.round(pos.x / factor);
      const y = Math.round(pos.y / factor);
      await invoke("save_hud_position", { x, y });
    } catch {
      // Drag failed or was cancelled — not critical.
    }
  };

  return (
    <div
      ref={shellRef}
      className={`hud-shell theme-${theme}${pinned ? " hud-shell-pinned" : ""}`}
      style={shellStyle}
      onMouseDown={handleMouseDown}
    >
      <HudCard
        active={runtime.appState === "recording" || runtime.appState === "processing"}
        backgroundUrl={backgroundUrl}
        config={runtime.config}
        level={0}
        peak={0}
        snapshot={runtime}
      />
    </div>
  );
}
