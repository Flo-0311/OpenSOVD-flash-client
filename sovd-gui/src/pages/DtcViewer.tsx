import { useState, useEffect, useRef } from "react";
import { useSearchParams } from "react-router-dom";
import { RefreshCw, Trash2, ShieldAlert } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useToast } from "../components/Toast";
import * as tauri from "../lib/tauri";
import type { DiagnosticTroubleCode } from "../types";

export default function DtcViewer() {
  const [searchParams] = useSearchParams();
  const { components, connected } = useConnectionStore();
  const { toast } = useToast();
  const [selectedComponent, setSelectedComponent] = useState(searchParams.get("component") ?? "");
  const [dtcs, setDtcs] = useState<DiagnosticTroubleCode[]>([]);
  const [loading, setLoading] = useState(false);
  const [clearConfirm, setClearConfirm] = useState(false);
  const requestIdRef = useRef(0);

  const refresh = async () => {
    if (!selectedComponent) return;
    const id = ++requestIdRef.current;
    setLoading(true);
    try {
      const result = await tauri.readDtcs(selectedComponent);
      if (id !== requestIdRef.current) return;
      setDtcs(result);
      if (result.length > 0) {
        toast("warning", `${result.length} DTC${result.length !== 1 ? "s" : ""} found`, `on ${selectedComponent}`);
      }
    } catch { /* ignore */ }
    if (id === requestIdRef.current) setLoading(false);
  };

  const handleComponentChange = (id: string) => {
    setSelectedComponent(id);
    setDtcs([]);
    setClearConfirm(false);
  };

  useEffect(() => {
    if (selectedComponent && connected) refresh();
  }, [selectedComponent, connected]);

  const clear = async () => {
    if (!clearConfirm) {
      setClearConfirm(true);
      setTimeout(() => setClearConfirm(false), 3000);
      return;
    }
    setClearConfirm(false);
    if (!selectedComponent) return;
    try {
      await tauri.clearDtcs(selectedComponent);
      setDtcs([]);
      toast("success", "DTCs Cleared", `Cleared all DTCs on ${selectedComponent}`);
    } catch { /* ignore */ }
  };

  const severityBadge = (severity?: string) => {
    const cls =
      severity === "Critical" ? "badge-error"
        : severity === "Error" ? "badge-error"
          : severity === "Warning" ? "badge-warning"
            : "badge-info";
    return <span className={`badge ${cls}`}>{severity ?? "—"}</span>;
  };

  const statusBadge = (status: string) => {
    const cls =
      status === "Active" ? "badge-error"
        : status === "Confirmed" ? "badge-warning"
          : status === "Pending" ? "badge-info"
            : "badge-neutral";
    return <span className={`badge ${cls}`}>{status}</span>;
  };

  const criticalCount = dtcs.filter((d) => d.severity === "Critical" || d.severity === "Error").length;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="page-title">
            Diagnostic Trouble Codes
            {dtcs.length > 0 && (
              <span className="ml-2 align-middle text-base font-normal text-muted-foreground">
                ({dtcs.length})
              </span>
            )}
          </h1>
          <p className="page-description">
            {dtcs.length === 0
              ? "Select a component and read DTCs"
              : `${dtcs.length} DTC${dtcs.length !== 1 ? "s" : ""} found${criticalCount > 0 ? ` · ${criticalCount} critical/error` : ""}`}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={selectedComponent}
            onChange={(e) => handleComponentChange(e.target.value)}
            className="input h-8 w-56 text-xs"
          >
            <option value="">Select component...</option>
            {components.map((c) => (
              <option key={c.id} value={c.id}>{c.name} ({c.id})</option>
            ))}
          </select>
          <button
            onClick={refresh}
            disabled={!connected || !selectedComponent || loading}
            className="btn-secondary btn-sm"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            Read DTCs
          </button>
          <button
            onClick={clear}
            disabled={!connected || !selectedComponent || dtcs.length === 0}
            className={`btn-sm ${
              clearConfirm
                ? "btn-destructive"
                : "btn-secondary text-destructive hover:bg-destructive/10"
            }`}
          >
            <Trash2 className="h-3.5 w-3.5" />
            {clearConfirm ? "Confirm Clear?" : "Clear DTCs"}
          </button>
        </div>
      </div>

      {dtcs.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center">
          <div className="rounded-full bg-muted p-3">
            <ShieldAlert className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="mt-3 font-semibold">No DTCs Found</h3>
          <p className="mt-1 max-w-xs text-sm text-muted-foreground">
            {!selectedComponent
              ? "Select a component to read its diagnostic trouble codes."
              : "No diagnostic trouble codes are stored for this component."}
          </p>
        </div>
      ) : (
        <div className="overflow-hidden rounded-lg border">
          <table className="w-full text-sm">
            <thead className="bg-muted/50">
              <tr>
                <th className="px-4 py-2.5 text-left text-xs font-medium">Code</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium">Description</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium">Status</th>
                <th className="px-4 py-2.5 text-left text-xs font-medium">Severity</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {dtcs.map((dtc) => (
                <tr key={dtc.id} className="transition-colors hover:bg-muted/30">
                  <td className="px-4 py-2.5 font-mono text-xs font-medium">{dtc.code}</td>
                  <td className="px-4 py-2.5">{dtc.description ?? "—"}</td>
                  <td className="px-4 py-2.5">{statusBadge(dtc.status)}</td>
                  <td className="px-4 py-2.5">{severityBadge(dtc.severity)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
