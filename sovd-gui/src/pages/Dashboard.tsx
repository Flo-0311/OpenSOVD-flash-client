import { useNavigate } from "react-router-dom";
import { Cpu, Zap, CheckCircle, Wifi, WifiOff, ArrowRight, AlertTriangle, Activity } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useJobStore } from "../stores/jobStore";

export default function Dashboard() {
  const { connected, capabilities, components } = useConnectionStore();
  const jobs = useJobStore((s) => s.jobs);
  const navigate = useNavigate();

  const activeJobs = jobs.filter((j) => j.state === "Running").length;
  const completedJobs = jobs.filter((j) => j.state === "Completed").length;
  const failedJobs = jobs.filter((j) => j.state === "Failed").length;
  const availableComponents = components.filter((c) => c.status === "Available").length;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="page-title">Dashboard</h1>
        <p className="page-description">
          {connected
            ? `Connected to SOVD ${capabilities?.sovd_version ?? ""} server with ${components.length} components`
            : "Connect to a SOVD server to get started"}
        </p>
      </div>

      {connected ? (
        <>
          {/* KPI Cards — clickable */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <button onClick={() => navigate("/components")} className="card-interactive text-left">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Components</span>
                <div className="rounded-md bg-primary/10 p-1.5">
                  <Cpu className="h-4 w-4 text-primary" />
                </div>
              </div>
              <p className="mt-2 text-3xl font-bold tracking-tight">{components.length}</p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {availableComponents} available · {components.length - availableComponents} unavailable
              </p>
            </button>

            <button onClick={() => navigate("/jobs")} className="card-interactive text-left">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Active Jobs</span>
                <div className="rounded-md bg-blue-500/10 p-1.5">
                  <Zap className="h-4 w-4 text-blue-500" />
                </div>
              </div>
              <p className="mt-2 text-3xl font-bold tracking-tight">{activeJobs}</p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {jobs.filter((j) => j.state === "Pending").length} pending · {completedJobs} completed
              </p>
            </button>

            <button onClick={() => navigate("/jobs")} className="card-interactive text-left">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Completed</span>
                <div className="rounded-md bg-green-500/10 p-1.5">
                  <CheckCircle className="h-4 w-4 text-green-500" />
                </div>
              </div>
              <p className="mt-2 text-3xl font-bold tracking-tight">{completedJobs}</p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {failedJobs > 0 ? (
                  <span className="text-red-500">{failedJobs} failed</span>
                ) : (
                  "No failures"
                )}
              </p>
            </button>

            <div className="card-interactive">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Server</span>
                <div className="rounded-md bg-green-500/10 p-1.5">
                  <Wifi className="h-4 w-4 text-green-500" />
                </div>
              </div>
              <p className="mt-2 text-lg font-bold tracking-tight">Connected</p>
              <p className="mt-0.5 text-xs font-mono text-muted-foreground">
                SOVD {capabilities?.sovd_version ?? "—"}
              </p>
            </div>
          </div>

          {/* Quick Actions */}
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
            <button
              onClick={() => navigate("/flash")}
              className="flex items-center gap-3 rounded-lg border bg-card p-4 text-left transition-colors hover:bg-accent"
            >
              <div className="rounded-lg bg-primary/10 p-2.5">
                <Zap className="h-5 w-5 text-primary" />
              </div>
              <div className="flex-1">
                <p className="text-sm font-medium">Flash ECU</p>
                <p className="text-xs text-muted-foreground">Start a new flash operation</p>
              </div>
              <ArrowRight className="h-4 w-4 text-muted-foreground" />
            </button>
            <button
              onClick={() => navigate("/dtcs")}
              className="flex items-center gap-3 rounded-lg border bg-card p-4 text-left transition-colors hover:bg-accent"
            >
              <div className="rounded-lg bg-yellow-500/10 p-2.5">
                <AlertTriangle className="h-5 w-5 text-yellow-500" />
              </div>
              <div className="flex-1">
                <p className="text-sm font-medium">Read DTCs</p>
                <p className="text-xs text-muted-foreground">Check diagnostic trouble codes</p>
              </div>
              <ArrowRight className="h-4 w-4 text-muted-foreground" />
            </button>
            <button
              onClick={() => navigate("/monitoring")}
              className="flex items-center gap-3 rounded-lg border bg-card p-4 text-left transition-colors hover:bg-accent"
            >
              <div className="rounded-lg bg-green-500/10 p-2.5">
                <Activity className="h-5 w-5 text-green-500" />
              </div>
              <div className="flex-1">
                <p className="text-sm font-medium">Live Monitoring</p>
                <p className="text-xs text-muted-foreground">View live diagnostic data</p>
              </div>
              <ArrowRight className="h-4 w-4 text-muted-foreground" />
            </button>
          </div>

          {/* Capabilities Summary */}
          {capabilities && (
            <div className="rounded-lg border bg-card p-4">
              <h2 className="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                Capabilities
              </h2>
              <div className="grid grid-cols-2 gap-2 sm:grid-cols-4 lg:grid-cols-8">
                {[
                  { label: "Flashing", value: capabilities.flashing, active: capabilities.flashing > 0 },
                  { label: "Diagnostics", value: capabilities.diagnostics, active: capabilities.diagnostics > 0 },
                  { label: "Fault Mgmt", value: capabilities.fault_management, active: capabilities.fault_management > 0 },
                  { label: "Config", value: capabilities.configuration, active: capabilities.configuration > 0 },
                  { label: "Provision", value: capabilities.provisioning, active: capabilities.provisioning > 0 },
                  { label: "Monitoring", value: capabilities.monitoring, active: capabilities.monitoring > 0 },
                  { label: "Logging", value: capabilities.logging, active: capabilities.logging > 0 },
                  { label: "Bulk", value: capabilities.bulk, active: capabilities.bulk > 0 },
                ].map(({ label, value, active }) => (
                  <div
                    key={label}
                    className={`rounded-md p-2.5 text-center transition-colors ${
                      active ? "bg-primary/5 ring-1 ring-primary/20" : "bg-muted"
                    }`}
                  >
                    <p className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">{label}</p>
                    <p className={`mt-0.5 text-lg font-bold ${active ? "text-primary" : "text-muted-foreground"}`}>{value}</p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Jobs */}
          {jobs.length > 0 && (
            <div className="rounded-lg border bg-card p-4">
              <div className="mb-3 flex items-center justify-between">
                <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                  Recent Jobs
                </h2>
                <button
                  onClick={() => navigate("/jobs")}
                  className="inline-flex items-center gap-1 text-xs text-primary hover:underline"
                >
                  View all <ArrowRight className="h-3 w-3" />
                </button>
              </div>
              <div className="space-y-2">
                {jobs.slice(-5).reverse().map((job) => (
                  <div
                    key={job.id}
                    className="flex items-center justify-between rounded-md bg-muted/50 p-3 transition-colors hover:bg-muted"
                  >
                    <div className="flex items-center gap-3">
                      <span className="font-mono text-sm font-medium">{job.target_component}</span>
                      <span className="text-xs text-muted-foreground">{job.job_type}</span>
                    </div>
                    <div className="flex items-center gap-3">
                      <span className="text-xs text-muted-foreground">{job.phase}</span>
                      <span
                        className={`badge ${
                          job.state === "Completed" ? "badge-success"
                            : job.state === "Running" ? "badge-info"
                              : job.state === "Failed" ? "badge-error"
                                : "badge-neutral"
                        }`}
                      >
                        {job.state}
                      </span>
                      {job.progress_percent != null && (
                        <span className="min-w-[3ch] text-right text-xs font-medium tabular-nums">{job.progress_percent}%</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </>
      ) : (
        /* Empty / disconnected state */
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-16 text-center">
          <div className="rounded-full bg-muted p-4">
            <WifiOff className="h-8 w-8 text-muted-foreground" />
          </div>
          <h2 className="mt-4 text-lg font-semibold">No Server Connection</h2>
          <p className="mt-1 max-w-sm text-sm text-muted-foreground">
            Enter a SOVD server URL in the connection bar above and click
            <strong> Connect</strong> to start working with ECU components.
          </p>
          <div className="mt-6 flex gap-3 text-xs text-muted-foreground">
            <div className="flex items-center gap-1.5">
              <Cpu className="h-3.5 w-3.5" />
              <span>Browse Components</span>
            </div>
            <span>·</span>
            <div className="flex items-center gap-1.5">
              <Zap className="h-3.5 w-3.5" />
              <span>Flash ECUs</span>
            </div>
            <span>·</span>
            <div className="flex items-center gap-1.5">
              <Activity className="h-3.5 w-3.5" />
              <span>Monitor Live Data</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
