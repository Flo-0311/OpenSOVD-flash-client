import { RefreshCw, Zap, AlertTriangle, Info, Cpu, Radio, Search, ArrowUpDown } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useNavigate } from "react-router-dom";
import * as tauri from "../lib/tauri";
import { useState, useMemo } from "react";
type SortKey = "id" | "name" | "component_type" | "status" | "software_version";
type SortDir = "asc" | "desc";

export default function ComponentExplorer() {
  const { connected, components, setComponents } = useConnectionStore();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  const refresh = async () => {
    setLoading(true);
    try {
      const list = await tauri.listComponents();
      setComponents(list);
    } catch { /* shown via connection error */ }
    setLoading(false);
  };

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const filtered = useMemo(() => {
    const f = filter.toLowerCase();
    const list = components.filter(
      (c) => c.id.toLowerCase().includes(f) || c.name.toLowerCase().includes(f),
    );
    return list.sort((a, b) => {
      const av = (a[sortKey] ?? "") as string;
      const bv = (b[sortKey] ?? "") as string;
      const cmp = av.localeCompare(bv);
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [components, filter, sortKey, sortDir]);

  const typeIcon = (type: string) => {
    switch (type) {
      case "NativeSovd": return <Radio className="h-3.5 w-3.5 text-blue-500" />;
      case "ClassicUds": return <Cpu className="h-3.5 w-3.5 text-orange-500" />;
      default: return <Info className="h-3.5 w-3.5 text-muted-foreground" />;
    }
  };

  const statusBadge = (status: string) => {
    const cls =
      status === "Available" ? "badge-success"
        : status === "Busy" ? "badge-info"
          : status === "Error" ? "badge-error"
            : status === "Offline" ? "badge-warning"
              : "badge-neutral";
    return <span className={`badge ${cls}`}>{status}</span>;
  };

  const SortHeader = ({ label, col }: { label: string; col: SortKey }) => (
    <button
      onClick={() => toggleSort(col)}
      className="inline-flex items-center gap-1 text-left font-medium hover:text-foreground"
    >
      {label}
      <ArrowUpDown className={`h-3 w-3 ${sortKey === col ? "text-primary" : "text-muted-foreground/40"}`} />
    </button>
  );

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="page-title">Components</h1>
          <p className="page-description">
            {connected
              ? `${components.length} component${components.length !== 1 ? "s" : ""} discovered · ${components.filter((c) => c.status === "Available").length} available`
              : "Connect to a server to discover components"}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Search components..."
              className="input h-8 w-56 pl-8 text-xs"
            />
          </div>
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
            <Cpu className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="mt-3 font-semibold">No Components Found</h3>
          <p className="mt-1 max-w-xs text-sm text-muted-foreground">
            {!connected
              ? "Connect to a SOVD server to discover available ECU components."
              : filter
                ? `No components match "${filter}". Try a different search.`
                : "No components are available on this server."}
          </p>
          {!connected && (
            <p className="mt-3 text-xs text-muted-foreground">
              Use the connection bar above to connect.
            </p>
          )}
        </div>
      ) : (
        <div className="overflow-hidden rounded-lg border">
          <table className="w-full text-sm">
            <thead className="bg-muted/50">
              <tr>
                <th className="px-4 py-2.5 text-left text-xs"><SortHeader label="ID" col="id" /></th>
                <th className="px-4 py-2.5 text-left text-xs"><SortHeader label="Name" col="name" /></th>
                <th className="px-4 py-2.5 text-left text-xs"><SortHeader label="Type" col="component_type" /></th>
                <th className="px-4 py-2.5 text-left text-xs"><SortHeader label="Status" col="status" /></th>
                <th className="px-4 py-2.5 text-left text-xs"><SortHeader label="SW Version" col="software_version" /></th>
                <th className="px-4 py-2.5 text-right text-xs font-medium">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {filtered.map((comp) => (
                <tr key={comp.id} className="transition-colors hover:bg-muted/30">
                  <td className="px-4 py-2.5 font-mono text-xs">{comp.id}</td>
                  <td className="px-4 py-2.5 font-medium">{comp.name}</td>
                  <td className="px-4 py-2.5">
                    <div className="flex items-center gap-1.5">
                      {typeIcon(comp.component_type)}
                      <span className="text-xs text-muted-foreground">
                        {comp.component_type === "NativeSovd" ? "Native SOVD" : comp.component_type === "ClassicUds" ? "Classic UDS" : "Unknown"}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-2.5">{statusBadge(comp.status)}</td>
                  <td className="px-4 py-2.5 font-mono text-xs">
                    {comp.software_version ?? "—"}
                  </td>
                  <td className="px-4 py-2.5 text-right">
                    <div className="flex justify-end gap-1">
                      <button
                        onClick={() => navigate(`/flash?component=${comp.id}`)}
                        disabled={comp.status !== "Available"}
                        className="btn-ghost btn-sm h-7 px-2 text-[11px]"
                      >
                        <Zap className="h-3 w-3" />
                        Flash
                      </button>
                      <button
                        onClick={() => navigate(`/dtcs?component=${comp.id}`)}
                        className="btn-ghost btn-sm h-7 px-2 text-[11px]"
                      >
                        <AlertTriangle className="h-3 w-3" />
                        DTCs
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
