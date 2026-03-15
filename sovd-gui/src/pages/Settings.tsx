import { useState } from "react";
import { Settings as SettingsIcon, Server, Palette, ScrollText, Activity, Info, Plus, Trash2 } from "lucide-react";
import { useSettingsStore, type ConnectionPreset } from "../stores/settingsStore";
import { useConnectionStore } from "../stores/connectionStore";
import { useToast } from "../components/Toast";

type Tab = "connection" | "appearance" | "logging" | "monitoring" | "about";

const tabs: { key: Tab; label: string; icon: typeof SettingsIcon }[] = [
  { key: "connection", label: "Connection", icon: Server },
  { key: "appearance", label: "Appearance", icon: Palette },
  { key: "logging", label: "Logging", icon: ScrollText },
  { key: "monitoring", label: "Monitoring", icon: Activity },
  { key: "about", label: "About", icon: Info },
];

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState<Tab>("connection");

  return (
    <div className="space-y-6">
      <div>
        <h1 className="page-title">Settings</h1>
        <p className="page-description">Configure application preferences and connection options</p>
      </div>

      <div className="flex gap-6">
        {/* Tab navigation */}
        <nav className="w-48 shrink-0 space-y-0.5">
          {tabs.map(({ key, label, icon: Icon }) => (
            <button
              key={key}
              onClick={() => setActiveTab(key)}
              className={`flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-sm transition-colors ${
                activeTab === key
                  ? "bg-primary/10 font-medium text-primary"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              }`}
            >
              <Icon className="h-4 w-4" />
              {label}
            </button>
          ))}
        </nav>

        {/* Tab content */}
        <div className="min-w-0 flex-1">
          {activeTab === "connection" && <ConnectionSettings />}
          {activeTab === "appearance" && <AppearanceSettings />}
          {activeTab === "logging" && <LoggingSettings />}
          {activeTab === "monitoring" && <MonitoringSettings />}
          {activeTab === "about" && <AboutSection />}
        </div>
      </div>
    </div>
  );
}

function ConnectionSettings() {
  const { defaultUrl, setDefaultUrl, connectionPresets, addPreset, removePreset, autoConnect, setAutoConnect } = useSettingsStore();
  const setUrl = useConnectionStore((s) => s.setUrl);
  const setToken = useConnectionStore((s) => s.setToken);
  const { toast } = useToast();

  const [newName, setNewName] = useState("");
  const [newUrl, setNewUrl] = useState("");
  const [newToken, setNewToken] = useState("");

  const handleAddPreset = () => {
    if (!newName || !newUrl) return;
    addPreset({ name: newName, url: newUrl, token: newToken });
    setNewName("");
    setNewUrl("");
    setNewToken("");
    toast("success", "Preset Added", `"${newName}" saved`);
  };

  const handleUsePreset = (preset: ConnectionPreset) => {
    setUrl(preset.url);
    setToken(preset.token);
    toast("info", "Preset Applied", `Connection set to "${preset.name}"`);
  };

  return (
    <div className="space-y-6">
      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">SOVD Server / Gateway</h3>
        <p className="mt-1 text-xs text-muted-foreground">
          The SOVD-compliant endpoint this client connects to (e.g. CDA Gateway for classic ECUs, or a native SOVD HPC).
        </p>
        <div className="mt-3 space-y-3">
          <div>
            <label className="mb-1.5 block text-sm font-medium">Default SOVD Server URL</label>
            <input
              type="url"
              value={defaultUrl}
              onChange={(e) => setDefaultUrl(e.target.value)}
              className="input w-full max-w-md font-mono text-xs"
              placeholder="http://localhost:8080"
            />
            <p className="mt-1 text-xs text-muted-foreground">
              Pre-filled in the connection bar when the application starts
            </p>
          </div>
          <div className="flex items-center gap-3">
            <label className="relative inline-flex cursor-pointer items-center">
              <input
                type="checkbox"
                checked={autoConnect}
                onChange={(e) => setAutoConnect(e.target.checked)}
                className="peer sr-only"
              />
              <div className="h-5 w-9 rounded-full bg-muted after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-primary peer-checked:after:translate-x-full" />
            </label>
            <span className="text-sm">Auto-connect on startup</span>
          </div>
        </div>
      </section>

      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Connection Presets</h3>
        <p className="mt-1 text-xs text-muted-foreground">
          Save frequently used server connections for quick access
        </p>

        <div className="mt-4 space-y-2">
          {connectionPresets.length === 0 ? (
            <p className="py-4 text-center text-sm text-muted-foreground">No presets saved</p>
          ) : (
            connectionPresets.map((preset) => (
              <div key={preset.name} className="flex items-center gap-3 rounded-md bg-muted/50 p-3">
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium">{preset.name}</p>
                  <p className="truncate font-mono text-xs text-muted-foreground">{preset.url}</p>
                </div>
                <button
                  onClick={() => handleUsePreset(preset)}
                  className="btn-secondary btn-sm h-7 text-[11px]"
                >
                  Use
                </button>
                <button
                  onClick={() => removePreset(preset.name)}
                  className="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </button>
              </div>
            ))
          )}
        </div>

        <div className="mt-4 rounded-md border border-dashed p-3">
          <p className="mb-2 text-xs font-medium text-muted-foreground">Add New Preset</p>
          <div className="flex items-end gap-2">
            <div className="flex-1">
              <label className="mb-1 block text-[11px] text-muted-foreground">Name</label>
              <input value={newName} onChange={(e) => setNewName(e.target.value)} className="input h-8 w-full text-xs" placeholder="Production" />
            </div>
            <div className="flex-[2]">
              <label className="mb-1 block text-[11px] text-muted-foreground">URL</label>
              <input value={newUrl} onChange={(e) => setNewUrl(e.target.value)} className="input h-8 w-full font-mono text-xs" placeholder="https://sovd.example.com:8080" />
            </div>
            <div className="flex-1">
              <label className="mb-1 block text-[11px] text-muted-foreground">Token</label>
              <input value={newToken} onChange={(e) => setNewToken(e.target.value)} type="password" className="input h-8 w-full text-xs" placeholder="(optional)" />
            </div>
            <button onClick={handleAddPreset} disabled={!newName || !newUrl} className="btn-primary btn-sm h-8">
              <Plus className="h-3.5 w-3.5" />
              Add
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}

function AppearanceSettings() {
  const { theme, setTheme, compactMode, setCompactMode } = useSettingsStore();

  return (
    <div className="space-y-6">
      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Theme</h3>
        <div className="mt-3 grid grid-cols-3 gap-3">
          {(["system", "light", "dark"] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTheme(t)}
              className={`flex flex-col items-center gap-2 rounded-lg border p-4 transition-all ${
                theme === t ? "border-primary bg-primary/5 ring-2 ring-primary/20" : "hover:bg-accent"
              }`}
            >
              <div className={`h-8 w-12 rounded border ${
                t === "dark" ? "bg-gray-900" : t === "light" ? "bg-white" : "bg-gradient-to-r from-white to-gray-900"
              }`} />
              <span className="text-xs font-medium capitalize">{t}</span>
            </button>
          ))}
        </div>
        <p className="mt-2 text-xs text-muted-foreground">
          You can also press <kbd className="rounded border bg-muted px-1 py-0.5 text-[10px] font-mono">D</kbd> to toggle dark mode
        </p>
      </section>

      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Layout</h3>
        <div className="mt-3 flex items-center gap-3">
          <label className="relative inline-flex cursor-pointer items-center">
            <input
              type="checkbox"
              checked={compactMode}
              onChange={(e) => setCompactMode(e.target.checked)}
              className="peer sr-only"
            />
            <div className="h-5 w-9 rounded-full bg-muted after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-primary peer-checked:after:translate-x-full" />
          </label>
          <div>
            <span className="text-sm font-medium">Compact Mode</span>
            <p className="text-xs text-muted-foreground">Reduce spacing and use smaller fonts</p>
          </div>
        </div>
      </section>
    </div>
  );
}

function LoggingSettings() {
  const { logLevel, setLogLevel, maxLogEntries, setMaxLogEntries } = useSettingsStore();

  return (
    <div className="space-y-6">
      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Log Configuration</h3>
        <div className="mt-3 space-y-4">
          <div>
            <label className="mb-1.5 block text-sm font-medium">Default Log Level Filter</label>
            <select
              value={logLevel}
              onChange={(e) => setLogLevel(e.target.value as typeof logLevel)}
              className="input h-9 w-40"
            >
              <option value="TRACE">TRACE</option>
              <option value="DEBUG">DEBUG</option>
              <option value="INFO">INFO</option>
              <option value="WARN">WARN</option>
              <option value="ERROR">ERROR</option>
            </select>
            <p className="mt-1 text-xs text-muted-foreground">
              Pre-selects this level in the Log Viewer filter dropdown
            </p>
          </div>
          <div>
            <label className="mb-1.5 block text-sm font-medium">Max Log Entries</label>
            <input
              type="number"
              value={maxLogEntries}
              onChange={(e) => setMaxLogEntries(Math.max(100, Number(e.target.value)))}
              className="input h-9 w-40 font-mono"
              min={100}
              max={10000}
              step={100}
            />
            <p className="mt-1 text-xs text-muted-foreground">
              Older entries are discarded when this limit is reached (100–10,000)
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}

function MonitoringSettings() {
  const { defaultRefreshInterval, setDefaultRefreshInterval } = useSettingsStore();

  return (
    <div className="space-y-6">
      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Live Monitoring</h3>
        <div className="mt-3 space-y-4">
          <div>
            <label className="mb-1.5 block text-sm font-medium">Default Refresh Interval</label>
            <div className="flex items-center gap-2">
              <input
                type="number"
                value={defaultRefreshInterval}
                onChange={(e) => setDefaultRefreshInterval(Math.max(200, Number(e.target.value)))}
                className="input h-9 w-32 font-mono"
                min={200}
                step={100}
              />
              <span className="text-sm text-muted-foreground">ms</span>
            </div>
            <p className="mt-1 text-xs text-muted-foreground">
              Default polling interval for live data (minimum 200ms)
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}

function AboutSection() {
  const connected = useConnectionStore((s) => s.connected);
  const capabilities = useConnectionStore((s) => s.capabilities);
  const components = useConnectionStore((s) => s.components);

  return (
    <div className="space-y-6">
      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Application</h3>
        <div className="mt-3 space-y-2 text-sm">
          <Row label="Name" value="OpenSOVD Flash Client" />
          <Row label="Version" value={__APP_VERSION__} />
          <Row label="Framework" value="Tauri 2.0 + React 18" />
          <Row label="Build" value="Vite 6" />
          <Row label="License" value="Eclipse Public License 2.0" />
        </div>
      </section>

      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">SOVD Server</h3>
        <div className="mt-3 space-y-2 text-sm">
          <Row label="Status" value={connected ? "Connected" : "Disconnected"} />
          <Row label="SOVD Version" value={capabilities?.sovd_version ?? "—"} />
          <Row label="Total Capabilities" value={String(capabilities?.total ?? 0)} />
          <Row label="Components" value={String(components.length)} />
        </div>
      </section>

      <section className="rounded-lg border p-5">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Ecosystem</h3>
        <p className="mt-2 text-xs text-muted-foreground">
          Part of the Eclipse OpenSOVD project — an open-source implementation of the SOVD (Service-Oriented Vehicle Diagnostics) standard by ASAM.
        </p>
        <div className="mt-3 flex flex-wrap gap-2">
          {["SOVD REST", "UDS/DoIP via CDA", "ASAM SOVD Standard", "Plugin Architecture"].map((tag) => (
            <span key={tag} className="badge badge-info">{tag}</span>
          ))}
        </div>
      </section>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between py-1">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium">{value}</span>
    </div>
  );
}

declare const __APP_VERSION__: string;
