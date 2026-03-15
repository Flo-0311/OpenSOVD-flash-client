import { useState } from "react";
import { Stethoscope, RefreshCw, Send, PenLine } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useToast } from "../components/Toast";
import * as tauri from "../lib/tauri";
import { sanitizeError } from "../lib/tauri";
import type { DataValue } from "../types";

type Mode = "read" | "write";

export default function Diagnostics() {
  const { components, connected } = useConnectionStore();
  const { toast } = useToast();
  const [mode, setMode] = useState<Mode>("read");
  const [selectedComponent, setSelectedComponent] = useState("");
  const [dataId, setDataId] = useState("");
  const [writeValue, setWriteValue] = useState("");
  const [results, setResults] = useState<DataValue[]>([]);
  const [loading, setLoading] = useState(false);

  const handleRead = async () => {
    if (!selectedComponent || !dataId) return;
    setLoading(true);
    try {
      const result = await tauri.readData(selectedComponent, dataId);
      setResults((prev) => [result, ...prev]);
      toast("success", "Data Read", `${result.name ?? dataId}: ${String(result.value)}${result.unit ? ` ${result.unit}` : ""}`);
    } catch (e) {
      toast("error", "Read Failed", sanitizeError(e));
    }
    setLoading(false);
  };

  const handleWrite = async () => {
    if (!selectedComponent || !dataId || writeValue === "") return;
    setLoading(true);
    try {
      let parsed: unknown = writeValue;
      try { parsed = JSON.parse(writeValue); } catch { /* keep as string */ }
      await tauri.writeData(selectedComponent, dataId, parsed);
      toast("success", "Data Written", `${dataId} = ${writeValue}`);
      // Re-read to confirm the written value
      try {
        const result = await tauri.readData(selectedComponent, dataId);
        setResults((prev) => [result, ...prev]);
      } catch { /* read-back optional */ }
    } catch (e) {
      toast("error", "Write Failed", sanitizeError(e));
    }
    setLoading(false);
  };

  const handleSubmit = () => { if (mode === "read") handleRead(); else handleWrite(); };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSubmit();
  };

  return (
    <div className="space-y-4">
      <div>
        <h1 className="page-title">Diagnostics</h1>
        <p className="page-description">
          Read and write diagnostic data values for SOVD components
        </p>
      </div>

      {/* Mode toggle + Controls */}
      <div className="space-y-3 rounded-lg border p-4">
        <div className="flex items-center gap-1 rounded-md bg-muted p-0.5" style={{ width: "fit-content" }}>
          <button
            onClick={() => setMode("read")}
            className={`rounded px-3 py-1 text-xs font-medium transition-colors ${mode === "read" ? "bg-background shadow-sm" : "text-muted-foreground hover:text-foreground"}`}
          >
            Read
          </button>
          <button
            onClick={() => setMode("write")}
            className={`rounded px-3 py-1 text-xs font-medium transition-colors ${mode === "write" ? "bg-background shadow-sm" : "text-muted-foreground hover:text-foreground"}`}
          >
            Write
          </button>
        </div>
        <div className="flex items-end gap-3">
          <div className="flex-1">
            <label className="mb-1.5 block text-sm font-medium">Component</label>
            <select
              value={selectedComponent}
              onChange={(e) => setSelectedComponent(e.target.value)}
              className="input h-9 w-full text-sm"
              disabled={!connected}
            >
              <option value="">Select component...</option>
              {components.map((c) => (
                <option key={c.id} value={c.id}>{c.name} ({c.id})</option>
              ))}
            </select>
          </div>
          <div className="flex-1">
            <label className="mb-1.5 block text-sm font-medium">Data ID</label>
            <input
              type="text"
              value={dataId}
              onChange={(e) => setDataId(e.target.value)}
              onKeyDown={handleKeyDown}
              className="input h-9 w-full font-mono text-sm"
              placeholder="e.g. engine.rpm, battery.voltage"
              disabled={!connected}
            />
          </div>
          {mode === "write" && (
            <div className="flex-1">
              <label className="mb-1.5 block text-sm font-medium">Value</label>
              <input
                type="text"
                value={writeValue}
                onChange={(e) => setWriteValue(e.target.value)}
                onKeyDown={handleKeyDown}
                className="input h-9 w-full font-mono text-sm"
                placeholder='e.g. 42, "on", true'
                disabled={!connected}
              />
            </div>
          )}
          <button
            onClick={handleSubmit}
            disabled={!connected || !selectedComponent || !dataId || loading || (mode === "write" && writeValue === "")}
            className={mode === "write" ? "btn-primary bg-yellow-600 hover:bg-yellow-700 dark:bg-yellow-600 dark:hover:bg-yellow-700" : "btn-primary"}
          >
            {loading ? <RefreshCw className="h-4 w-4 animate-spin" /> : mode === "read" ? <Send className="h-4 w-4" /> : <PenLine className="h-4 w-4" />}
            {mode === "read" ? "Read" : "Write"}
          </button>
        </div>
      </div>

      {/* Results */}
      {results.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center">
          <div className="rounded-full bg-muted p-3">
            <Stethoscope className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="mt-3 font-semibold">No Diagnostic Data</h3>
          <p className="mt-1 max-w-xs text-sm text-muted-foreground">
            {!connected
              ? "Connect to a SOVD server to read diagnostic data."
              : "Select a component and enter a data ID to read a value."}
          </p>
        </div>
      ) : (
        <div className="space-y-6">
          {/* Latest result highlighted */}
          <div className="rounded-lg border border-primary/20 bg-primary/5 p-5">
            <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">Latest Value</p>
            <div className="mt-2 flex items-baseline gap-3">
              <span className="text-3xl font-bold tabular-nums tracking-tight">{String(results[0].value)}</span>
              {results[0].unit && <span className="text-lg text-muted-foreground">{results[0].unit}</span>}
            </div>
            <div className="mt-2 flex items-center gap-3 text-xs text-muted-foreground">
              <span className="font-mono">{results[0].name ?? results[0].id}</span>
              {results[0].timestamp && (
                <>
                  <span>·</span>
                  <span>{new Date(results[0].timestamp).toLocaleString()}</span>
                </>
              )}
            </div>
          </div>

          {/* History table */}
          {results.length > 1 && (
            <div className="rounded-lg border">
              <div className="flex items-center justify-between border-b px-4 py-2.5">
                <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                  Reading History ({results.length})
                </h3>
                <button onClick={() => setResults([])} className="btn-ghost btn-sm h-7 text-[11px] text-muted-foreground">
                  Clear
                </button>
              </div>
              <table className="w-full text-sm">
                <thead className="bg-muted/50">
                  <tr>
                    <th className="px-4 py-2 text-left text-xs font-medium">Data ID</th>
                    <th className="px-4 py-2 text-left text-xs font-medium">Value</th>
                    <th className="px-4 py-2 text-left text-xs font-medium">Unit</th>
                    <th className="px-4 py-2 text-left text-xs font-medium">Timestamp</th>
                  </tr>
                </thead>
                <tbody className="divide-y">
                  {results.map((r, i) => (
                    <tr key={`${r.id}-${i}`} className={`transition-colors hover:bg-muted/30 ${i === 0 ? "bg-primary/5" : ""}`}>
                      <td className="px-4 py-2 font-mono text-xs">{r.name ?? r.id}</td>
                      <td className="px-4 py-2 font-mono font-medium tabular-nums">{String(r.value)}</td>
                      <td className="px-4 py-2 text-xs text-muted-foreground">{r.unit ?? "—"}</td>
                      <td className="px-4 py-2 text-xs tabular-nums text-muted-foreground">
                        {r.timestamp ? new Date(r.timestamp).toLocaleTimeString() : "—"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
