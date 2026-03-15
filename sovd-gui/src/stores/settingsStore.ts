import { create } from "zustand";

interface ConnectionPreset {
  name: string;
  url: string;
  token: string;
}

interface SettingsStore {
  // Connection
  defaultUrl: string;
  connectionPresets: ConnectionPreset[];
  autoConnect: boolean;
  // Appearance
  theme: "system" | "light" | "dark";
  compactMode: boolean;
  // Logging
  logLevel: "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";
  maxLogEntries: number;
  // Monitoring
  defaultRefreshInterval: number;
  // Actions
  setDefaultUrl: (url: string) => void;
  setConnectionPresets: (presets: ConnectionPreset[]) => void;
  addPreset: (preset: ConnectionPreset) => void;
  removePreset: (name: string) => void;
  setAutoConnect: (autoConnect: boolean) => void;
  setTheme: (theme: "system" | "light" | "dark") => void;
  setCompactMode: (compact: boolean) => void;
  setLogLevel: (level: "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR") => void;
  setMaxLogEntries: (max: number) => void;
  setDefaultRefreshInterval: (ms: number) => void;
}

const STORAGE_KEY = "sovd-settings";

function loadSettings(): Partial<SettingsStore> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function saveSettings(state: Partial<SettingsStore>) {
  try {
    const toSave = {
      defaultUrl: state.defaultUrl,
      connectionPresets: (state.connectionPresets ?? []).map(({ name, url }) => ({ name, url })),
      autoConnect: state.autoConnect,
      theme: state.theme,
      compactMode: state.compactMode,
      logLevel: state.logLevel,
      maxLogEntries: state.maxLogEntries,
      defaultRefreshInterval: state.defaultRefreshInterval,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(toSave));
  } catch { /* ignore */ }
}

const saved = loadSettings();

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  defaultUrl: saved.defaultUrl ?? "http://localhost:8080",
  connectionPresets: ((saved.connectionPresets as Array<{ name: string; url: string; token?: string }>) ?? [
    { name: "Local Dev", url: "http://localhost:8080" },
  ]).map((p) => ({ name: p.name, url: p.url, token: p.token ?? "" })),
  autoConnect: saved.autoConnect ?? false,
  theme: (saved.theme as "system" | "light" | "dark") ?? "system",
  compactMode: saved.compactMode ?? false,
  logLevel: (saved.logLevel as "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR") ?? "INFO",
  maxLogEntries: saved.maxLogEntries ?? 1000,
  defaultRefreshInterval: saved.defaultRefreshInterval ?? 1000,

  setDefaultUrl: (url) => { set({ defaultUrl: url }); saveSettings({ ...get(), defaultUrl: url }); },
  setConnectionPresets: (presets) => { set({ connectionPresets: presets }); saveSettings({ ...get(), connectionPresets: presets }); },
  addPreset: (preset) => {
    const presets = [...get().connectionPresets, preset];
    set({ connectionPresets: presets });
    saveSettings({ ...get(), connectionPresets: presets });
  },
  removePreset: (name) => {
    const presets = get().connectionPresets.filter((p) => p.name !== name);
    set({ connectionPresets: presets });
    saveSettings({ ...get(), connectionPresets: presets });
  },
  setAutoConnect: (autoConnect) => { set({ autoConnect }); saveSettings({ ...get(), autoConnect }); },
  setTheme: (theme) => { set({ theme }); saveSettings({ ...get(), theme }); },
  setCompactMode: (compact) => { set({ compactMode: compact }); saveSettings({ ...get(), compactMode: compact }); },
  setLogLevel: (level) => { set({ logLevel: level }); saveSettings({ ...get(), logLevel: level }); },
  setMaxLogEntries: (max) => { set({ maxLogEntries: max }); saveSettings({ ...get(), maxLogEntries: max }); },
  setDefaultRefreshInterval: (ms) => { set({ defaultRefreshInterval: ms }); saveSettings({ ...get(), defaultRefreshInterval: ms }); },
}));

export type { ConnectionPreset };
