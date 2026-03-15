import { useState, useRef } from "react";
import { Layers, Zap, CheckCircle, Loader2, XCircle } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useToast } from "../components/Toast";
import * as tauri from "../lib/tauri";
import { sanitizeError } from "../lib/tauri";
import type { SoftwarePackage } from "../types";

type FlashResult = { componentId: string; status: "pending" | "running" | "success" | "error"; error?: string };

export default function BulkFlash() {
  const { components, connected } = useConnectionStore();
  const { toast } = useToast();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [pkgName, setPkgName] = useState("");
  const [pkgVersion, setPkgVersion] = useState("");
  const [flashing, setFlashing] = useState(false);
  const [results, setResults] = useState<FlashResult[]>([]);
  const cancelledRef = useRef(false);

  const available = components.filter((c) => c.status === "Available");

  const toggleComponent = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const toggleAll = () => {
    if (selected.size === available.length) {
      setSelected(new Set());
    } else {
      setSelected(new Set(available.map((c) => c.id)));
    }
  };

  const handleStartBulk = async () => {
    if (selected.size === 0 || !pkgName || !pkgVersion) return;
    const ids = [...selected];
    const initial: FlashResult[] = ids.map((id) => ({ componentId: id, status: "pending" }));
    cancelledRef.current = false;
    setResults(initial);
    setFlashing(true);
    toast("info", "Bulk Flash", `Starting sequential flash for ${ids.length} components`);

    let successCount = 0;
    let failCount = 0;
    for (let i = 0; i < ids.length; i++) {
      if (cancelledRef.current) break;
      const componentId = ids[i];
      setResults((prev) => prev.map((r, j) => j === i ? { ...r, status: "running" } : r));
      try {
        const pkg: SoftwarePackage = { name: pkgName, version: pkgVersion, target_component: componentId };
        await tauri.startFlash(componentId, pkg);
        if (cancelledRef.current) break;
        setResults((prev) => prev.map((r, j) => j === i ? { ...r, status: "success" } : r));
        successCount++;
      } catch (e) {
        if (cancelledRef.current) break;
        setResults((prev) => prev.map((r, j) => j === i ? { ...r, status: "error", error: sanitizeError(e) } : r));
        failCount++;
      }
    }
    if (!cancelledRef.current) {
      setFlashing(false);
      toast(
        failCount === 0 ? "success" : "warning",
        "Bulk Flash Complete",
        `${successCount} succeeded, ${failCount} failed out of ${ids.length} components`,
      );
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="page-title">Bulk Flash</h1>
        <p className="page-description">
          Deploy software updates to multiple components sequentially
        </p>
      </div>

      {!connected ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center">
          <div className="rounded-full bg-muted p-3">
            <Layers className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="mt-3 font-semibold">Not Connected</h3>
          <p className="mt-1 max-w-xs text-sm text-muted-foreground">
            Connect to a SOVD server to use bulk flash operations.
          </p>
        </div>
      ) : (
        <>
          {/* Component Selection */}
          <div className="rounded-lg border p-5">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                  Select Components
                </h3>
                <p className="mt-0.5 text-xs text-muted-foreground">
                  {selected.size} of {available.length} selected
                </p>
              </div>
              <button onClick={toggleAll} className="btn-secondary btn-sm">
                {selected.size === available.length ? "Deselect All" : "Select All"}
              </button>
            </div>
            <div className="mt-3 grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3">
              {available.map((comp) => (
                <button
                  key={comp.id}
                  onClick={() => toggleComponent(comp.id)}
                  className={`flex items-center gap-3 rounded-lg border p-3 text-left transition-all ${
                    selected.has(comp.id)
                      ? "border-primary bg-primary/5 ring-1 ring-primary/20"
                      : "hover:bg-accent"
                  }`}
                >
                  <div className={`flex h-5 w-5 items-center justify-center rounded border transition-colors ${
                    selected.has(comp.id) ? "border-primary bg-primary" : "border-muted-foreground/30"
                  }`}>
                    {selected.has(comp.id) && <CheckCircle className="h-3 w-3 text-white" />}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{comp.name}</p>
                    <p className="truncate font-mono text-[10px] text-muted-foreground">
                      {comp.id} · {comp.software_version ?? "—"}
                    </p>
                  </div>
                </button>
              ))}
              {available.length === 0 && (
                <p className="col-span-full py-4 text-center text-sm text-muted-foreground">
                  No available components on this server
                </p>
              )}
            </div>
          </div>

          {/* Package Config */}
          <div className="rounded-lg border p-5">
            <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
              Software Package
            </h3>
            <div className="mt-3 flex items-end gap-3">
              <div className="flex-1">
                <label className="mb-1.5 block text-sm font-medium">Package Name</label>
                <input
                  value={pkgName}
                  onChange={(e) => setPkgName(e.target.value)}
                  className="input w-full"
                  placeholder="e.g. ecu-software-update"
                />
              </div>
              <div className="flex-1">
                <label className="mb-1.5 block text-sm font-medium">Version</label>
                <input
                  value={pkgVersion}
                  onChange={(e) => setPkgVersion(e.target.value)}
                  className="input w-full font-mono"
                  placeholder="e.g. 2.1.0"
                />
              </div>
              <button
                onClick={handleStartBulk}
                disabled={selected.size === 0 || !pkgName || !pkgVersion || flashing}
                className="btn-primary"
              >
                {flashing ? <Loader2 className="h-4 w-4 animate-spin" /> : <Zap className="h-4 w-4" />}
                {flashing ? "Flashing..." : `Flash ${selected.size} Component${selected.size !== 1 ? "s" : ""}`}
              </button>
            </div>
          </div>

          {/* Results */}
          {results.length > 0 && (
            <div className="rounded-lg border p-5">
              <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                Flash Progress
              </h3>
              <div className="mt-3 space-y-2">
                {results.map((r) => (
                  <div key={r.componentId} className="flex items-center gap-3 rounded-md bg-muted/50 p-3">
                    <div className="flex h-6 w-6 items-center justify-center">
                      {r.status === "pending" && <div className="h-2 w-2 rounded-full bg-muted-foreground/30" />}
                      {r.status === "running" && <Loader2 className="h-4 w-4 animate-spin text-primary" />}
                      {r.status === "success" && <CheckCircle className="h-4 w-4 text-green-500" />}
                      {r.status === "error" && <XCircle className="h-4 w-4 text-destructive" />}
                    </div>
                    <span className="flex-1 font-mono text-sm">{r.componentId}</span>
                    <span className={`text-xs font-medium ${
                      r.status === "success" ? "text-green-600" : r.status === "error" ? "text-destructive" : r.status === "running" ? "text-primary" : "text-muted-foreground"
                    }`}>
                      {r.status === "pending" ? "Waiting" : r.status === "running" ? "Flashing..." : r.status === "success" ? "Done" : "Failed"}
                    </span>
                    {r.error && <span className="max-w-xs truncate text-[10px] text-destructive">{r.error}</span>}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Summary */}
          {selected.size > 0 && !flashing && results.length === 0 && (
            <div className="rounded-md bg-blue-50 p-4 text-sm text-blue-800 dark:bg-blue-950/30 dark:text-blue-200">
              <p className="font-medium">Bulk Flash Summary</p>
              <p className="mt-1 text-xs opacity-80">
                {selected.size} component{selected.size !== 1 ? "s" : ""} selected:
                {" "}{[...selected].join(", ")}
              </p>
            </div>
          )}
        </>
      )}
    </div>
  );
}
