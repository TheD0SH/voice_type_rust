import { useEffect, useState } from "react";
import type { MutableRefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import { configsEqual } from "../lib/config";
import type { Config } from "../types";

export function useAutosave(
  loading: boolean,
  capturingHotkey: boolean,
  config: Config,
  dirty: boolean,
  setError: (e: string | null) => void,
  setSavedConfig: (c: Config | ((prev: Config) => Config)) => void,
  savedConfigRef: MutableRefObject<Config>
) {
  const [autosaving, setAutosaving] = useState(false);

  async function persistConfig(nextConfig: Config) {
    setAutosaving(true);
    setError(null);

    try {
      await invoke("save_config", { config: nextConfig });
      savedConfigRef.current = nextConfig;
      setSavedConfig(nextConfig);
    } catch (saveError) {
      setError(String(saveError));
    } finally {
      setAutosaving(false);
    }
  }

  useEffect(() => {
    if (loading || capturingHotkey || autosaving || !dirty) {
      return;
    }

    const timeoutId = window.setTimeout(() => {
      void persistConfig(config);
    }, 900);

    return () => window.clearTimeout(timeoutId);
  }, [autosaving, capturingHotkey, config, dirty, loading]);

  useEffect(() => {
    const handleBeforeUnload = () => {
      if (dirty && !configsEqual(config, savedConfigRef.current)) {
        void invoke("save_config", { config });
      }
    };

    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => {
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  }, [config, dirty]);

  return { autosaving, persistConfig };
}
