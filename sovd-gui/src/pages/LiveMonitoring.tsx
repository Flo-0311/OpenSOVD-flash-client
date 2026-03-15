import { useState, useEffect, useRef } from "react";
import { Activity, Play, Pause, RefreshCw } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useSettingsStore } from "../stores/settingsStore";
import * as tauri from "../lib/tauri";
import type { DataValue } from "../types";

export default function LiveMonitoring() {
  const { components, connected } = useConnectionStore();
  const [selectedComponent, setSelectedComponent] = useState("");
  const [data, setData] = useState<DataValue[]>([]);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const defaultInterval = useSettingsStore((s) => s.defaultRefreshInterval);
  const [interval, setInterval_] = useState(defaultInterval);
  const [fetching, setFetching] = useState(false);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const fetchingRef = useRef(false);

  const fetchData = async () => {
    if (!selectedComponent || !connected || fetchingRef.current) return;
    fetchingRef.current = true;
    setFetching(true);
    try { setData(await tauri.getLiveData(selectedComponent)); } catch { /* ignore */ }
    setFetching(false);
    fetchingRef.current = false;
  };

  useEffect(() => {
    if (autoRefresh && selectedComponent && connected) {
      fetchData();
      timerRef.current = setInterval(fetchData, interval);
    }
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, [autoRefresh, selectedComponent, connected, interval]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="page-title">Live Monitoring</h1>
          <p className="page-description">
            {data.length > 0
              ? `${data.length} data point${data.length !== 1 ? "s" : ""}${autoRefresh ? ` · refreshing every ${interval}ms` : ""}`
              : "Select a component to view live diagnostic data"}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={selectedComponent}
            onChange={(e) => { setSelectedComponent(e.target.value); setData([]); setAutoRefresh(false); }}
            className="input h-8 w-56 text-xs"
          >
            <option value="">Select component...</option>
            {components.map((c) => (
              <option key={c.id} value={c.id}>{c.name} ({c.id})</option>
            ))}
          </select>
          <div className="flex items-center gap-1 rounded-md border bg-background px-2">
            <span className="text-[10px] text-muted-foreground">ms</span>
            <input
              type="number"
              value={interval}
              onChange={(e) => setInterval_(Math.max(200, Number(e.target.value)))}
              className="h-8 w-16 bg-transparent px-1 text-center text-xs tabular-nums focus:outline-none"
              min={200}
              step={100}
            />
          </div>
          <button
            onClick={fetchData}
            disabled={!connected || !selectedComponent || fetching}
            className="btn-secondary btn-sm"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${fetching ? "animate-spin" : ""}`} />
            Fetch
          </button>
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            disabled={!connected || !selectedComponent}
            className={`btn-sm ${autoRefresh ? "btn-primary bg-green-600 hover:bg-green-700" : "btn-secondary"}`}
          >
            {autoRefresh ? <Pause className="h-3.5 w-3.5" /> : <Play className="h-3.5 w-3.5" />}
            {autoRefresh ? "Stop" : "Live"}
          </button>
        </div>
      </div>

      {autoRefresh && (
        <div className="flex items-center gap-2 rounded-md bg-green-50 px-3 py-1.5 text-xs text-green-700 dark:bg-green-950/30 dark:text-green-300">
          <span className="h-2 w-2 rounded-full bg-green-500 animate-pulse-dot" />
          Live monitoring active — refreshing every {interval}ms
        </div>
      )}

      {data.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center">
          <div className="rounded-full bg-muted p-3">
            <Activity className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="mt-3 font-semibold">No Data Available</h3>
          <p className="mt-1 max-w-xs text-sm text-muted-foreground">
            {!selectedComponent
              ? "Select a component to start monitoring its live data."
              : "Click Fetch or enable Live mode to start receiving data."}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {data.map((d) => (
            <div key={d.id} className="rounded-lg border bg-card p-4 transition-all hover:shadow-sm">
              <p className="truncate text-xs font-medium text-muted-foreground">{d.name ?? d.id}</p>
              <div className="mt-1.5 flex items-baseline gap-2">
                <span className="text-2xl font-bold tabular-nums tracking-tight">{String(d.value)}</span>
                {d.unit && <span className="text-sm text-muted-foreground">{d.unit}</span>}
              </div>
              {d.timestamp && (
                <p className="mt-1.5 text-[10px] tabular-nums text-muted-foreground">
                  {new Date(d.timestamp).toLocaleTimeString()}
                </p>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
