import { useState, useEffect } from "react";
import { Puzzle, RefreshCw, Search } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import * as tauri from "../lib/tauri";
import type { PluginInfo } from "../types";

const typeBadge = (type: string) => {
  const cls =
    type === "security" ? "badge-error"
      : type === "backend" ? "badge-info"
        : type === "workflow" ? "badge-warning"
          : type === "reporting" ? "badge-success"
            : "badge-neutral";
  return <span className={`badge ${cls}`}>{type}</span>;
};

export default function PluginManager() {
  const connected = useConnectionStore((s) => s.connected);
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState("");

  const refresh = async () => {
    setLoading(true);
    try { setPlugins(await tauri.listPlugins()); } catch { /* ignore */ }
    setLoading(false);
  };

  useEffect(() => { if (connected) refresh(); }, [connected]);

  const filtered = plugins.filter(
    (p) =>
      p.name.toLowerCase().includes(filter.toLowerCase()) ||
      p.description.toLowerCase().includes(filter.toLowerCase()) ||
      p.plugin_type.toLowerCase().includes(filter.toLowerCase()),
  );

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="page-title">
            Plugins
            {plugins.length > 0 && (
              <span className="ml-2 align-middle text-base font-normal text-muted-foreground">
                ({plugins.length})
              </span>
            )}
          </h1>
          <p className="page-description">
            {plugins.length === 0
              ? "View loaded plugins and extensions"
              : `${plugins.length} plugin${plugins.length !== 1 ? "s" : ""} loaded`}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {plugins.length > 0 && (
            <div className="relative">
              <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
              <input
                type="text"
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
                placeholder="Search plugins..."
                className="input h-8 w-48 pl-8 text-xs"
              />
            </div>
          )}
          <button
            onClick={refresh}
            disabled={!connected || loading}
            className="btn-secondary btn-sm"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </button>
        </div>
      </div>

      {filtered.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center">
          <div className="rounded-full bg-muted p-3">
            <Puzzle className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="mt-3 font-semibold">
            {plugins.length > 0 ? "No Matching Plugins" : "No Plugins Loaded"}
          </h3>
          <p className="mt-1 max-w-xs text-sm text-muted-foreground">
            {plugins.length > 0
              ? `No plugins match "${filter}". Try a different search.`
              : "Plugins extend the client with custom security, workflows, and reporting capabilities."}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {filtered.map((p) => (
            <div key={p.name} className="card-interactive">
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0 flex-1">
                  <h3 className="truncate font-medium">{p.name}</h3>
                  <p className="mt-0.5 text-xs text-muted-foreground line-clamp-2">{p.description}</p>
                </div>
                {typeBadge(p.plugin_type)}
              </div>
              <p className="mt-3 font-mono text-xs text-muted-foreground">v{p.version}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
