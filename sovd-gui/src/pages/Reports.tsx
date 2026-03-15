import { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { FileBarChart, Download, RefreshCw } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useJobStore } from "../stores/jobStore";
import { useToast } from "../components/Toast";

type ReportType = "flash-summary" | "dtc-overview" | "job-history" | "capability-audit";

const reportTypes: { key: ReportType; label: string; description: string }[] = [
  { key: "flash-summary", label: "Flash Summary", description: "Overview of all flash operations with success/failure statistics" },
  { key: "dtc-overview", label: "DTC Overview", description: "Summary of diagnostic trouble codes across all components" },
  { key: "job-history", label: "Job History", description: "Complete job execution history with timing and outcomes" },
  { key: "capability-audit", label: "Capability Audit", description: "Server capabilities and component inventory" },
];

function downloadJson(data: unknown, filename: string) {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export default function Reports() {
  const { connected, capabilities, components } = useConnectionStore();
  const jobs = useJobStore((s) => s.jobs);
  const { toast } = useToast();
  const navigate = useNavigate();
  const [selectedReport, setSelectedReport] = useState<ReportType>("flash-summary");
  const [generating, setGenerating] = useState(false);
  const completedJobs = jobs.filter((j) => j.state === "Completed").length;
  const failedJobs = jobs.filter((j) => j.state === "Failed").length;
  const flashJobs = jobs.filter((j) => j.job_type === "Flash" || j.job_type === "SoftwareUpdate");

  const buildReportData = useCallback(() => {
    switch (selectedReport) {
      case "flash-summary":
        return {
          type: "flash-summary",
          generated_at: new Date().toISOString(),
          total: flashJobs.length,
          successful: flashJobs.filter((j) => j.state === "Completed").length,
          failed: flashJobs.filter((j) => j.state === "Failed").length,
          jobs: flashJobs.map((j) => ({ id: j.id, component: j.target_component, state: j.state, created: j.created_at })),
        };
      case "job-history":
        return {
          type: "job-history",
          generated_at: new Date().toISOString(),
          total: jobs.length,
          completed: completedJobs,
          failed: failedJobs,
          running: jobs.filter((j) => j.state === "Running").length,
          jobs: jobs.map((j) => ({ id: j.id, type: j.job_type, component: j.target_component, state: j.state, phase: j.phase, created: j.created_at, updated: j.updated_at })),
        };
      case "capability-audit":
        return {
          type: "capability-audit",
          generated_at: new Date().toISOString(),
          capabilities,
          components: components.map((c) => ({ id: c.id, name: c.name, type: c.component_type, status: c.status, sw: c.software_version })),
        };
      case "dtc-overview":
        return {
          type: "dtc-overview",
          generated_at: new Date().toISOString(),
          note: "DTC data must be read per-component via the DTC Viewer first.",
        };
    }
  }, [selectedReport, flashJobs, jobs, completedJobs, failedJobs, capabilities, components]);

  const handleGenerate = () => {
    setGenerating(true);
    const data = buildReportData();
    setTimeout(() => {
      setGenerating(false);
      const label = reportTypes.find((r) => r.key === selectedReport)?.label ?? selectedReport;
      toast("success", "Report Generated", `${label} report ready (${JSON.stringify(data).length} bytes)`);
    }, 300);
  };

  const handleExport = () => {
    const data = buildReportData();
    const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, "-");
    downloadJson(data, `sovd-${selectedReport}-${timestamp}.json`);
    toast("success", "Exported", `${selectedReport} report saved as JSON`);
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="page-title">Reports</h1>
        <p className="page-description">
          Generate and export diagnostic reports from collected data
        </p>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        {/* Report type selection */}
        <div className="space-y-2 lg:col-span-1">
          <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            Report Type
          </h3>
          {reportTypes.map(({ key, label, description }) => (
            <button
              key={key}
              onClick={() => setSelectedReport(key)}
              className={`flex w-full flex-col rounded-lg border p-3 text-left transition-all ${
                selectedReport === key
                  ? "border-primary bg-primary/5 ring-1 ring-primary/20"
                  : "hover:bg-accent"
              }`}
            >
              <span className="text-sm font-medium">{label}</span>
              <span className="mt-0.5 text-xs text-muted-foreground">{description}</span>
            </button>
          ))}
        </div>

        {/* Report preview */}
        <div className="rounded-lg border lg:col-span-2">
          <div className="flex items-center justify-between border-b px-5 py-3">
            <h3 className="font-semibold">
              {reportTypes.find((r) => r.key === selectedReport)?.label}
            </h3>
            <div className="flex items-center gap-2">
              <button
                onClick={handleGenerate}
                disabled={!connected || generating}
                className="btn-secondary btn-sm"
              >
                <RefreshCw className={`h-3.5 w-3.5 ${generating ? "animate-spin" : ""}`} />
                {generating ? "Generating..." : "Generate"}
              </button>
              <button
                onClick={handleExport}
                disabled={!connected && jobs.length === 0}
                className="btn-primary btn-sm"
              >
                <Download className="h-3.5 w-3.5" />
                Export JSON
              </button>
            </div>
          </div>

          <div className="p-5">
            {selectedReport === "flash-summary" && (
              <div className="space-y-4">
                <div className="grid grid-cols-3 gap-3">
                  <div className="rounded-md bg-muted/50 p-3 text-center">
                    <p className="text-2xl font-bold tabular-nums">{flashJobs.length}</p>
                    <p className="text-xs text-muted-foreground">Total Operations</p>
                  </div>
                  <div className="rounded-md bg-green-50 p-3 text-center dark:bg-green-950/30">
                    <p className="text-2xl font-bold tabular-nums text-green-600">{flashJobs.filter((j) => j.state === "Completed").length}</p>
                    <p className="text-xs text-muted-foreground">Successful</p>
                  </div>
                  <div className="rounded-md bg-red-50 p-3 text-center dark:bg-red-950/30">
                    <p className="text-2xl font-bold tabular-nums text-red-600">{flashJobs.filter((j) => j.state === "Failed").length}</p>
                    <p className="text-xs text-muted-foreground">Failed</p>
                  </div>
                </div>
                {flashJobs.length === 0 && (
                  <p className="py-4 text-center text-sm text-muted-foreground">No flash operations recorded yet</p>
                )}
              </div>
            )}

            {selectedReport === "job-history" && (
              <div className="space-y-4">
                <div className="grid grid-cols-4 gap-3">
                  <div className="rounded-md bg-muted/50 p-3 text-center">
                    <p className="text-2xl font-bold tabular-nums">{jobs.length}</p>
                    <p className="text-xs text-muted-foreground">Total</p>
                  </div>
                  <div className="rounded-md bg-green-50 p-3 text-center dark:bg-green-950/30">
                    <p className="text-2xl font-bold tabular-nums text-green-600">{completedJobs}</p>
                    <p className="text-xs text-muted-foreground">Completed</p>
                  </div>
                  <div className="rounded-md bg-red-50 p-3 text-center dark:bg-red-950/30">
                    <p className="text-2xl font-bold tabular-nums text-red-600">{failedJobs}</p>
                    <p className="text-xs text-muted-foreground">Failed</p>
                  </div>
                  <div className="rounded-md bg-blue-50 p-3 text-center dark:bg-blue-950/30">
                    <p className="text-2xl font-bold tabular-nums text-blue-600">{jobs.filter((j) => j.state === "Running").length}</p>
                    <p className="text-xs text-muted-foreground">Running</p>
                  </div>
                </div>
                {jobs.length > 0 && (
                  <div className="max-h-60 overflow-y-auto rounded border text-xs">
                    <table className="w-full">
                      <thead className="sticky top-0 bg-muted/80 backdrop-blur">
                        <tr>
                          <th className="px-3 py-1.5 text-left font-medium">Component</th>
                          <th className="px-3 py-1.5 text-left font-medium">Type</th>
                          <th className="px-3 py-1.5 text-left font-medium">State</th>
                          <th className="px-3 py-1.5 text-left font-medium">Created</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y">
                        {jobs.map((j) => (
                          <tr key={j.id} className="hover:bg-muted/30">
                            <td className="px-3 py-1.5 font-mono">{j.target_component}</td>
                            <td className="px-3 py-1.5">{j.job_type}</td>
                            <td className="px-3 py-1.5">
                              <span className={`badge ${
                                j.state === "Completed" ? "badge-success" : j.state === "Failed" ? "badge-error" : j.state === "Running" ? "badge-info" : "badge-neutral"
                              }`}>{j.state}</span>
                            </td>
                            <td className="px-3 py-1.5 tabular-nums text-muted-foreground">{new Date(j.created_at).toLocaleString()}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
              </div>
            )}

            {selectedReport === "capability-audit" && (
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                  {capabilities ? (
                    <>
                      {[
                        { label: "Flashing", value: capabilities.flashing },
                        { label: "Diagnostics", value: capabilities.diagnostics },
                        { label: "Fault Mgmt", value: capabilities.fault_management },
                        { label: "Config", value: capabilities.configuration },
                        { label: "Provision", value: capabilities.provisioning },
                        { label: "Monitoring", value: capabilities.monitoring },
                        { label: "Logging", value: capabilities.logging },
                        { label: "Bulk", value: capabilities.bulk },
                      ].map(({ label, value }) => (
                        <div key={label} className={`rounded-md p-2.5 text-center ${value > 0 ? "bg-primary/5 ring-1 ring-primary/20" : "bg-muted"}`}>
                          <p className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">{label}</p>
                          <p className={`mt-0.5 text-lg font-bold ${value > 0 ? "text-primary" : "text-muted-foreground"}`}>{value}</p>
                        </div>
                      ))}
                    </>
                  ) : (
                    <p className="col-span-full py-4 text-center text-sm text-muted-foreground">Connect to a server to view capabilities</p>
                  )}
                </div>
                <div>
                  <h4 className="text-xs font-medium text-muted-foreground">Components ({components.length})</h4>
                  <div className="mt-2 space-y-1">
                    {components.map((c) => (
                      <div key={c.id} className="flex items-center justify-between rounded bg-muted/30 px-3 py-1.5 text-xs">
                        <span className="font-medium">{c.name}</span>
                        <div className="flex items-center gap-2">
                          <span className="font-mono text-muted-foreground">{c.component_type}</span>
                          <span className={`badge ${c.status === "Available" ? "badge-success" : "badge-neutral"}`}>{c.status}</span>
                        </div>
                      </div>
                    ))}
                    {components.length === 0 && (
                      <p className="py-2 text-center text-xs text-muted-foreground">No components discovered</p>
                    )}
                  </div>
                </div>
              </div>
            )}

            {selectedReport === "dtc-overview" && (
              <div className="flex flex-col items-center justify-center py-8 text-center">
                <div className="rounded-full bg-muted p-3">
                  <FileBarChart className="h-8 w-8 text-muted-foreground" />
                </div>
                <h3 className="mt-3 font-semibold">DTC Report</h3>
                <p className="mt-1 max-w-xs text-sm text-muted-foreground">
                  Read DTCs from individual components first, then generate a cross-component report here.
                </p>
                <button onClick={() => navigate("/dtcs")} className="btn-secondary btn-sm mt-4">
                  Go to DTC Viewer
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
