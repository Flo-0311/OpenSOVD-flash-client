import { create } from "zustand";
import type { CapabilitySummary, SovdComponent } from "../types";

function getDefaultUrl(): string {
  try {
    const raw = localStorage.getItem("sovd-settings");
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed.defaultUrl) return parsed.defaultUrl;
    }
  } catch { /* ignore */ }
  return "http://localhost:8080";
}

interface ConnectionStore {
  url: string;
  token: string;
  connected: boolean;
  connecting: boolean;
  error: string | null;
  sovdVersion: string | null;
  capabilities: CapabilitySummary | null;
  components: SovdComponent[];
  setUrl: (url: string) => void;
  setToken: (token: string) => void;
  setConnected: (connected: boolean) => void;
  setConnecting: (connecting: boolean) => void;
  setError: (error: string | null) => void;
  setSovdVersion: (version: string | null) => void;
  setCapabilities: (caps: CapabilitySummary | null) => void;
  setComponents: (components: SovdComponent[]) => void;
  reset: () => void;
}

export const useConnectionStore = create<ConnectionStore>((set) => ({
  url: getDefaultUrl(),
  token: "",
  connected: false,
  connecting: false,
  error: null,
  sovdVersion: null,
  capabilities: null,
  components: [],
  setUrl: (url) => set({ url }),
  setToken: (token) => set({ token }),
  setConnected: (connected) => set({ connected }),
  setConnecting: (connecting) => set({ connecting }),
  setError: (error) => set({ error }),
  setSovdVersion: (version) => set({ sovdVersion: version }),
  setCapabilities: (caps) => set({ capabilities: caps }),
  setComponents: (components) => set({ components }),
  reset: () =>
    set({
      connected: false,
      connecting: false,
      error: null,
      sovdVersion: null,
      capabilities: null,
      components: [],
    }),
}));
