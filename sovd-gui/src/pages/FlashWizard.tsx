import { useState, useEffect } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { Zap, Loader2, CheckCircle, XCircle, AlertTriangle, ArrowLeft, ArrowRight, RotateCcw, Check } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useJobStore } from "../stores/jobStore";
import { useToast } from "../components/Toast";
import PhaseIndicator from "../components/PhaseIndicator";
import * as tauri from "../lib/tauri";
import { sanitizeError } from "../lib/tauri";
import type { JobPhase, SoftwarePackage } from "../types";

type WizardStep = "select" | "configure" | "confirm" | "progress" | "result";

const STEPS: { key: WizardStep; label: string }[] = [
  { key: "select", label: "Select" },
  { key: "configure", label: "Configure" },
  { key: "confirm", label: "Confirm" },
  { key: "progress", label: "Flash" },
  { key: "result", label: "Result" },
];

export default function FlashWizard() {
  const [searchParams] = useSearchParams();
  const { components, connected } = useConnectionStore();
  const { setActiveFlash } = useJobStore();
  const { toast } = useToast();
  const navigate = useNavigate();

  const [step, setStep] = useState<WizardStep>("select");
  const [selectedComponent, setSelectedComponent] = useState(searchParams.get("component") ?? "");
  const [pkgName, setPkgName] = useState("");
  const [pkgVersion, setPkgVersion] = useState("");
  const [flashing, setFlashing] = useState(false);
  const [phase, setPhase] = useState<JobPhase>("PreCheck");
  const [progress, setProgress] = useState(0);
  const [result, setResult] = useState<"success" | "error" | null>(null);
  const [errorMsg, setErrorMsg] = useState("");

  useEffect(() => {
    const unlisten = tauri.onFlashProgress((fp) => {
      setPhase(fp.phase);
      setProgress(fp.percent);
      setActiveFlash(fp);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [setActiveFlash]);

  const startFlash = async () => {
    setStep("progress");
    setFlashing(true);
    setResult(null);
    setErrorMsg("");
    try {
      const pkg: SoftwarePackage = {
        name: pkgName,
        version: pkgVersion,
        target_component: selectedComponent,
      };
      await tauri.startFlash(selectedComponent, pkg);
      setResult("success");
      setStep("result");
      toast("success", "Flash Successful", `${selectedComponent} updated to ${pkgName} v${pkgVersion}`);
    } catch (e) {
      setResult("error");
      const msg = sanitizeError(e);
      setErrorMsg(msg);
      setStep("result");
      toast("error", "Flash Failed", msg);
    } finally {
      setFlashing(false);
      setActiveFlash(null);
    }
  };

  const resetWizard = () => {
    setStep("select");
    setSelectedComponent("");
    setPkgName("");
    setPkgVersion("");
    setResult(null);
    setErrorMsg("");
    setProgress(0);
    setPhase("PreCheck");
  };

  const available = components.filter((c) => c.status === "Available");
  const stepIdx = STEPS.findIndex((s) => s.key === step);

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div>
        <h1 className="page-title">Flash Wizard</h1>
        <p className="page-description">
          Deploy software updates to components via SOVD
        </p>
      </div>

      {/* Stepper with proper done/current/upcoming states */}
      <nav className="flex items-center" aria-label="Flash wizard progress">
        {STEPS.map((s, i) => {
          const isDone = i < stepIdx || (step === "result" && result === "success");
          const isCurrent = i === stepIdx;
          return (
            <div key={s.key} className="flex flex-1 items-center">
              {i > 0 && (
                <div className={`h-0.5 flex-1 transition-colors duration-300 ${isDone ? "bg-green-500" : "bg-border"}`} />
              )}
              <div className="flex flex-col items-center gap-1">
                <div
                  className={`flex h-8 w-8 items-center justify-center rounded-full text-xs font-medium transition-all duration-300 ${
                    isDone
                      ? "bg-green-500 text-white"
                      : isCurrent
                        ? "bg-primary text-primary-foreground ring-4 ring-primary/20"
                        : "bg-muted text-muted-foreground"
                  }`}
                >
                  {isDone ? <Check className="h-4 w-4" /> : i + 1}
                </div>
                <span className={`text-[10px] font-medium ${isCurrent ? "text-primary" : isDone ? "text-green-600 dark:text-green-400" : "text-muted-foreground"}`}>
                  {s.label}
                </span>
              </div>
              {i < STEPS.length - 1 && i >= 0 && (
                <div className={`h-0.5 flex-1 transition-colors duration-300 ${isDone ? "bg-green-500" : "bg-border"}`} />
              )}
            </div>
          );
        })}
      </nav>

      {/* Not connected guard */}
      {!connected && step !== "result" && (
        <div className="flex items-center gap-2 rounded-lg border border-yellow-500/30 bg-yellow-50 p-3 text-sm text-yellow-800 dark:bg-yellow-950/30 dark:text-yellow-200">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          Connect to a SOVD server before starting a flash operation.
        </div>
      )}

      {/* Step 1: Select Component */}
      {step === "select" && (
        <div className="space-y-4 rounded-lg border p-6">
          <div>
            <h2 className="text-lg font-semibold">Select Component</h2>
            <p className="mt-0.5 text-sm text-muted-foreground">
              Choose the target component to update ({available.length} available)
            </p>
          </div>
          <select
            value={selectedComponent}
            onChange={(e) => setSelectedComponent(e.target.value)}
            className="input h-10 w-full"
            disabled={!connected}
          >
            <option value="">Select a component...</option>
            {available.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name} ({c.id}) — {c.software_version ?? "?"}
              </option>
            ))}
          </select>
          <div className="flex justify-end">
            <button
              onClick={() => setStep("configure")}
              disabled={!selectedComponent || !connected}
              className="btn-primary"
            >
              Next
              <ArrowRight className="h-4 w-4" />
            </button>
          </div>
        </div>
      )}

      {/* Step 2: Configure Package */}
      {step === "configure" && (
        <div className="space-y-4 rounded-lg border p-6">
          <div>
            <h2 className="text-lg font-semibold">Configure Package</h2>
            <p className="mt-0.5 text-sm text-muted-foreground">
              Specify the software package details for <span className="font-mono font-medium text-foreground">{selectedComponent}</span>
            </p>
          </div>
          <div className="space-y-3">
            <div>
              <label className="mb-1.5 block text-sm font-medium">Package Name</label>
              <input
                value={pkgName}
                onChange={(e) => setPkgName(e.target.value)}
                className="input w-full"
                placeholder="e.g. ecu-software-update"
                autoFocus
              />
            </div>
            <div>
              <label className="mb-1.5 block text-sm font-medium">Package Version</label>
              <input
                value={pkgVersion}
                onChange={(e) => setPkgVersion(e.target.value)}
                className="input w-full font-mono"
                placeholder="e.g. 2.1.0"
              />
            </div>
          </div>
          <div className="flex justify-between">
            <button onClick={() => setStep("select")} className="btn-secondary">
              <ArrowLeft className="h-4 w-4" />
              Back
            </button>
            <button
              onClick={() => setStep("confirm")}
              disabled={!pkgName || !pkgVersion}
              className="btn-primary"
            >
              Next
              <ArrowRight className="h-4 w-4" />
            </button>
          </div>
        </div>
      )}

      {/* Step 3: Confirm — with warning styling */}
      {step === "confirm" && (
        <div className="space-y-4 rounded-lg border border-yellow-500/30 p-6">
          <div className="flex items-start gap-3">
            <div className="rounded-lg bg-yellow-500/10 p-2">
              <AlertTriangle className="h-5 w-5 text-yellow-600 dark:text-yellow-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold">Confirm Flash Operation</h2>
              <p className="mt-0.5 text-sm text-muted-foreground">
                This will overwrite the current software. Please verify the details below.
              </p>
            </div>
          </div>
          <div className="space-y-2 rounded-md bg-muted p-4 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Target Component</span>
              <span className="font-mono font-medium">{selectedComponent}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Package Name</span>
              <span className="font-medium">{pkgName}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Package Version</span>
              <span className="font-mono font-medium">{pkgVersion}</span>
            </div>
          </div>
          <div className="flex justify-between">
            <button onClick={() => setStep("configure")} className="btn-secondary">
              <ArrowLeft className="h-4 w-4" />
              Back
            </button>
            <button
              onClick={startFlash}
              disabled={!connected}
              className="btn-primary bg-yellow-600 hover:bg-yellow-700 dark:bg-yellow-600 dark:hover:bg-yellow-700"
            >
              <Zap className="h-4 w-4" />
              Start Flash
            </button>
          </div>
        </div>
      )}

      {/* Step 4: Progress */}
      {step === "progress" && (
        <div className="space-y-6 rounded-lg border p-6">
          <h2 className="text-lg font-semibold">Flash in Progress</h2>
          <PhaseIndicator currentPhase={phase} />
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">{phase}</span>
              <span className="font-medium tabular-nums">{progress}%</span>
            </div>
            <div className="h-3 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-primary transition-all duration-500 ease-out"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
          {flashing && (
            <div className="flex flex-col items-center gap-3">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Loader2 className="h-4 w-4 animate-spin" />
                Flashing <span className="font-mono font-medium text-foreground">{selectedComponent}</span>...
              </div>
              <p className="text-xs text-muted-foreground">
                Do not disconnect from the server or close the application during this process.
              </p>
            </div>
          )}
        </div>
      )}

      {/* Step 5: Result */}
      {step === "result" && (
        <div className="space-y-5 rounded-lg border p-8 text-center">
          {result === "success" ? (
            <>
              <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-green-100 dark:bg-green-900/30">
                <CheckCircle className="h-8 w-8 text-green-600 dark:text-green-400" />
              </div>
              <div>
                <h2 className="text-xl font-bold text-green-700 dark:text-green-400">Flash Successful</h2>
                <p className="mt-1 text-sm text-muted-foreground">
                  <span className="font-mono">{selectedComponent}</span> updated to <span className="font-medium">{pkgName}</span> v{pkgVersion}
                </p>
              </div>
            </>
          ) : (
            <>
              <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-100 dark:bg-red-900/30">
                <XCircle className="h-8 w-8 text-destructive" />
              </div>
              <div>
                <h2 className="text-xl font-bold text-destructive">Flash Failed</h2>
                <p className="mt-1 text-sm text-muted-foreground">{errorMsg}</p>
              </div>
            </>
          )}
          <div className="flex items-center justify-center gap-3 pt-2">
            <button onClick={resetWizard} className="btn-secondary">
              <RotateCcw className="h-4 w-4" />
              Start New Flash
            </button>
            <button onClick={() => navigate("/jobs")} className="btn-ghost text-sm">
              View Jobs
              <ArrowRight className="h-4 w-4" />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
