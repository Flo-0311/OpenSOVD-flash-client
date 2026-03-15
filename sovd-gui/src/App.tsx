import { Routes, Route, NavLink, useLocation } from "react-router-dom";
import {
  LayoutDashboard,
  Cpu,
  Zap,
  ClipboardList,
  AlertTriangle,
  Activity,
  FileText,
  Puzzle,
  Settings,
  Sun,
  Moon,
  ChevronDown,
  ChevronUp,
  Server,
  Stethoscope,
  Layers,
  FileBarChart,
} from "lucide-react";
import { useState, useEffect, useCallback } from "react";
import { useConnectionStore } from "./stores/connectionStore";
import { useJobStore } from "./stores/jobStore";
import { useSettingsStore } from "./stores/settingsStore";
import Dashboard from "./pages/Dashboard";
import ComponentExplorer from "./pages/ComponentExplorer";
import FlashWizard from "./pages/FlashWizard";
import JobMonitor from "./pages/JobMonitor";
import DtcViewer from "./pages/DtcViewer";
import LiveMonitoring from "./pages/LiveMonitoring";
import LogViewer from "./pages/LogViewer";
import PluginManager from "./pages/PluginManager";
import Diagnostics from "./pages/Diagnostics";
import BulkFlash from "./pages/BulkFlash";
import Reports from "./pages/Reports";
import SettingsPage from "./pages/Settings";
import ConnectionBar from "./components/ConnectionBar";
import * as tauri from "./lib/tauri";

const navItems = [
  { path: "/", icon: LayoutDashboard, label: "Dashboard", shortcut: "1" },
  { path: "/components", icon: Cpu, label: "Components", shortcut: "2" },
  { path: "/flash", icon: Zap, label: "Flash Wizard", shortcut: "3" },
  { path: "/jobs", icon: ClipboardList, label: "Jobs", shortcut: "4" },
  { path: "/dtcs", icon: AlertTriangle, label: "DTCs", shortcut: "5" },
  { path: "/diagnostics", icon: Stethoscope, label: "Diagnostics", shortcut: "6" },
  { path: "/monitoring", icon: Activity, label: "Monitoring", shortcut: "7" },
  { path: "/bulk-flash", icon: Layers, label: "Bulk Flash", shortcut: "8" },
  { path: "/reports", icon: FileBarChart, label: "Reports", shortcut: "9" },
  { path: "/logs", icon: FileText, label: "Logs", shortcut: "0" },
  { path: "/plugins", icon: Puzzle, label: "Plugins", shortcut: "" },
];

export default function App() {
  const theme = useSettingsStore((s) => s.theme);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const compactMode = useSettingsStore((s) => s.compactMode);
  const autoConnect = useSettingsStore((s) => s.autoConnect);
  const [connectionExpanded, setConnectionExpanded] = useState(true);
  const location = useLocation();
  const connected = useConnectionStore((s) => s.connected);
  const url = useConnectionStore((s) => s.url);
  const token = useConnectionStore((s) => s.token);
  const setConnected = useConnectionStore((s) => s.setConnected);
  const setConnecting = useConnectionStore((s) => s.setConnecting);
  const setCapabilities = useConnectionStore((s) => s.setCapabilities);
  const setSovdVersion = useConnectionStore((s) => s.setSovdVersion);
  const setComponents = useConnectionStore((s) => s.setComponents);
  const capabilities = useConnectionStore((s) => s.capabilities);
  const components = useConnectionStore((s) => s.components);
  const jobs = useJobStore((s) => s.jobs);
  const activeJobs = jobs.filter((j) => j.state === "Running").length;

  const resolvedDark = theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", resolvedDark);
    document.documentElement.classList.toggle("compact", compactMode);
  }, [resolvedDark, compactMode]);

  // A4: Auto-connect on startup
  useEffect(() => {
    if (!autoConnect || connected || !url) return;
    let cancelled = false;
    (async () => {
      setConnecting(true);
      try {
        const caps = await tauri.connectToServer(url, token || undefined);
        if (cancelled) return;
        setCapabilities(caps);
        setSovdVersion(caps.sovd_version ?? null);
        setConnected(true);
        const comps = await tauri.listComponents();
        if (!cancelled) setComponents(comps);
      } catch {
        // silent fail on auto-connect
      } finally {
        if (!cancelled) setConnecting(false);
      }
    })();
    return () => { cancelled = true; };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const toggleTheme = useCallback(() => {
    if (theme === "dark") setTheme("light");
    else if (theme === "light") setTheme("dark");
    else setTheme(resolvedDark ? "light" : "dark");
  }, [theme, resolvedDark, setTheme]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement) return;
      if (e.metaKey || e.ctrlKey) return;
      if (e.key === "d") { toggleTheme(); e.preventDefault(); }
    },
    [toggleTheme],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Sidebar */}
      <aside className="flex w-56 flex-col border-r bg-card">
        <div className="flex items-center gap-2.5 border-b px-4 py-3.5">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
            <Zap className="h-4 w-4 text-primary-foreground" />
          </div>
          <div className="flex flex-col">
            <span className="text-sm font-bold leading-none tracking-tight">
              OpenSOVD
            </span>
            <span className="text-[10px] text-muted-foreground">Flash Client</span>
          </div>
        </div>

        <nav className="flex-1 space-y-0.5 overflow-y-auto px-2 py-2">
          {navItems.map(({ path, icon: Icon, label, shortcut }) => (
            <NavLink
              key={path}
              to={path}
              end={path === "/"}
              className={({ isActive }) =>
                `group relative flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-all duration-150 ${
                  isActive
                    ? "bg-primary/10 font-medium text-primary"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                }`
              }
            >
              {({ isActive }) => (
                <>
                  {isActive && (
                    <div className="absolute -left-2 top-1/2 h-5 w-1 -translate-y-1/2 rounded-r-full bg-primary" />
                  )}
                  <Icon className="h-4 w-4" />
                  <span className="flex-1">{label}</span>
                  <span className="hidden text-[10px] text-muted-foreground/50 group-hover:inline">
                    {shortcut}
                  </span>
                </>
              )}
            </NavLink>
          ))}
        </nav>

        <div className="border-t px-2 py-2">
          <NavLink
            to="/settings"
            className={({ isActive }) =>
              `group relative flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-all duration-150 ${
                isActive
                  ? "bg-primary/10 font-medium text-primary"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              }`
            }
          >
            {({ isActive }) => (
              <>
                {isActive && (
                  <div className="absolute -left-2 top-1/2 h-5 w-1 -translate-y-1/2 rounded-r-full bg-primary" />
                )}
                <Settings className="h-4 w-4" />
                Settings
              </>
            )}
          </NavLink>
        </div>

        <div className="flex items-center justify-between border-t px-3 py-2">
          <button
            onClick={toggleTheme}
            className="rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
            title="Toggle theme (D)"
            aria-label="Toggle dark mode"
          >
            {resolvedDark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
          </button>
          <span className="text-[10px] text-muted-foreground">
            {theme === "system" ? "System" : resolvedDark ? "Dark" : "Light"}
          </span>
        </div>
      </aside>

      {/* Main */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Collapsible Connection Bar */}
        {connected ? (
          <button
            onClick={() => setConnectionExpanded(!connectionExpanded)}
            className="flex items-center gap-2 border-b bg-card px-4 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-accent"
          >
            <span className="h-2 w-2 rounded-full bg-green-500 animate-pulse-dot" />
            <Server className="h-3 w-3" />
            <span className="font-medium text-foreground">Connected</span>
            <span className="text-muted-foreground">{url}</span>
            <span className="ml-auto">
              {connectionExpanded ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
            </span>
          </button>
        ) : null}
        {(!connected || connectionExpanded) && <ConnectionBar />}

        <main className="flex-1 overflow-y-auto p-6">
          <div key={location.pathname} className="animate-in">
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/components" element={<ComponentExplorer />} />
              <Route path="/flash" element={<FlashWizard />} />
              <Route path="/jobs" element={<JobMonitor />} />
              <Route path="/dtcs" element={<DtcViewer />} />
              <Route path="/monitoring" element={<LiveMonitoring />} />
              <Route path="/diagnostics" element={<Diagnostics />} />
              <Route path="/bulk-flash" element={<BulkFlash />} />
              <Route path="/reports" element={<Reports />} />
              <Route path="/logs" element={<LogViewer />} />
              <Route path="/plugins" element={<PluginManager />} />
              <Route path="/settings" element={<SettingsPage />} />
            </Routes>
          </div>
        </main>

        {/* Status Bar */}
        <footer className="flex items-center gap-3 border-t bg-card px-4 py-1.5 text-[11px] text-muted-foreground">
          <span className="font-medium">
            SOVD {capabilities?.sovd_version ?? "—"}
          </span>
          <span className="h-3 w-px bg-border" />
          <span>{components.length} Components</span>
          <span className="h-3 w-px bg-border" />
          <span>{capabilities?.total ?? 0} Capabilities</span>
          {activeJobs > 0 && (
            <>
              <span className="h-3 w-px bg-border" />
              <span className="flex items-center gap-1 font-medium text-blue-600 dark:text-blue-400">
                <span className="h-1.5 w-1.5 rounded-full bg-blue-500 animate-pulse-dot" />
                {activeJobs} active {activeJobs === 1 ? "job" : "jobs"}
              </span>
            </>
          )}
          <span className="ml-auto flex items-center gap-1.5">
            <span
              className={`h-2 w-2 rounded-full ${connected ? "bg-green-500 animate-pulse-dot" : "bg-red-500"}`}
            />
            {connected ? "Online" : "Offline"}
          </span>
        </footer>
      </div>
    </div>
  );
}
