import { RefreshCw, XCircle, ClipboardList, Play, Pause } from "lucide-react";
import { useJobStore } from "../stores/jobStore";
import { useConnectionStore } from "../stores/connectionStore";
import { useToast } from "../components/Toast";
import PhaseIndicator from "../components/PhaseIndicator";
import * as tauri from "../lib/tauri";
import { useState, useEffect, useRef } from "react";

export default function JobMonitor() {
  const { jobs, setJobs } = useJobStore();
  const connected = useConnectionStore((s) => s.connected);
  const { toast } = useToast();
  const [loading, setLoading] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [cancelConfirm, setCancelConfirm] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const refreshingRef = useRef(false);

  const refresh = async () => {
    if (refreshingRef.current) return;
    refreshingRef.current = true;
    setLoading(true);
    try { setJobs(await tauri.listJobs()); } catch { /* ignore */ }
    setLoading(false);
    refreshingRef.current = false;
  };

  useEffect(() => {
    if (autoRefresh && connected) {
      refresh();
      timerRef.current = setInterval(refresh, 3000);
    }
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, [autoRefresh, connected]);

  const cancel = async (id: string) => {
    if (cancelConfirm !== id) {
      setCancelConfirm(id);
      setTimeout(() => setCancelConfirm(null), 3000);
      return;
    }
    setCancelConfirm(null);
    try {
      await tauri.cancelJob(id);
      toast("info", "Job Cancelled", `Job ${id.slice(0, 8)}... has been cancelled`);
      await refresh();
    } catch { /* ignore */ }
  };

  const stateBadge = (state: string) => {
    const cls: Record<string, string> = {
      Completed: "badge-success",
      Running: "badge-info",
      Failed: "badge-error",
      Cancelled: "badge-neutral",
      Pending: "badge-warning",
      Paused: "badge-orange",
    };
    return <span className={`badge ${cls[state] ?? cls.Pending}`}>{state}</span>;
  };

  const activeCount = jobs.filter((j) => j.state === "Running").length;
  const pendingCount = jobs.filter((j) => j.state === "Pending").length;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="page-title">Jobs</h1>
          <p className="page-description">
            {jobs.length === 0
              ? "No jobs recorded yet"
              : `${jobs.length} total · ${activeCount} running · ${pendingCount} pending`}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            disabled={!connected}
            className={`btn-sm ${autoRefresh ? "btn-primary" : "btn-secondary"}`}
          >
            {autoRefresh ? <Pause className="h-3.5 w-3.5" /> : <Play className="h-3.5 w-3.5" />}
            {autoRefresh ? "Stop" : "Auto-Refresh"}
          </button>
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

      {jobs.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center">
          <div className="rounded-full bg-muted p-3">
            <ClipboardList className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="mt-3 font-semibold">No Jobs Found</h3>
          <p className="mt-1 max-w-xs text-sm text-muted-foreground">
            Start a flash operation or run diagnostics to see jobs here.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {jobs.map((job) => (
            <div key={job.id} className="rounded-lg border bg-card p-4 transition-colors hover:bg-card/80">
              <div className="flex items-start justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-sm font-medium">{job.target_component}</span>
                    {stateBadge(job.state)}
                  </div>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {job.job_type} &middot; {job.id.slice(0, 8)}... &middot; {new Date(job.created_at).toLocaleString()}
                  </p>
                </div>
                {(job.state === "Running" || job.state === "Pending") && (
                  <button
                    onClick={() => cancel(job.id)}
                    className={`btn-ghost btn-sm h-7 px-2 text-xs ${
                      cancelConfirm === job.id
                        ? "text-destructive hover:bg-destructive/10"
                        : "text-muted-foreground hover:text-destructive"
                    }`}
                    title={cancelConfirm === job.id ? "Click again to confirm" : "Cancel job"}
                  >
                    <XCircle className="h-3.5 w-3.5" />
                    {cancelConfirm === job.id ? "Confirm?" : "Cancel"}
                  </button>
                )}
              </div>

              {job.state === "Running" && (
                <div className="mt-3 space-y-2">
                  <PhaseIndicator currentPhase={job.phase} />
                  {job.progress_percent != null && (
                    <div className="flex items-center gap-3">
                      <div className="h-2 flex-1 overflow-hidden rounded-full bg-muted">
                        <div
                          className="h-full rounded-full bg-primary transition-all duration-500"
                          style={{ width: `${job.progress_percent}%` }}
                        />
                      </div>
                      <span className="min-w-[3ch] text-right text-xs font-medium tabular-nums">{job.progress_percent}%</span>
                    </div>
                  )}
                </div>
              )}

              {job.state === "Completed" && (
                <div className="mt-2">
                  <PhaseIndicator currentPhase={job.phase} completed />
                </div>
              )}

              {job.state === "Failed" && job.error_message && (
                <div className="mt-2 rounded-md bg-destructive/5 px-3 py-2 text-xs text-destructive">
                  {job.error_message}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
