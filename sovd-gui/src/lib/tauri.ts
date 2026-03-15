import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  SovdComponent,
  CapabilitySummary,
  Job,
  DiagnosticTroubleCode,
  DataValue,
  SoftwarePackage,
  FlashProgress,
  PluginInfo,
  LogEntry,
} from "../types";

// --- Error Sanitization ---

export function sanitizeError(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  // Strip file system paths (Unix + Windows)
  let msg = raw.replace(/(?:[A-Z]:\\|\/)[^\s:]+\.\w+/gi, "[path]");
  // Strip stack traces (lines starting with "at ...")
  msg = msg.replace(/\n\s*at\s.+/g, "");
  // Strip internal IP addresses (e.g. 10.x, 172.16-31.x, 192.168.x)
  msg = msg.replace(/\b(?:10\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])|192\.168)\.\d{1,3}\.\d{1,3}\b/g, "[internal-ip]");
  // Truncate overly long messages
  if (msg.length > 300) msg = msg.slice(0, 297) + "…";
  return msg.trim() || "An unknown error occurred";
}

// --- Connection ---

export async function connectToServer(url: string, token?: string): Promise<CapabilitySummary> {
  return invoke<CapabilitySummary>("connect_to_server", { url, token: token || null });
}

export async function disconnect(): Promise<void> {
  return invoke("disconnect");
}

// --- Components ---

export async function listComponents(): Promise<SovdComponent[]> {
  return invoke<SovdComponent[]>("list_components");
}

// --- Flash ---

export async function startFlash(componentId: string, pkg: SoftwarePackage): Promise<string> {
  return invoke<string>("start_flash", { componentId, pkg });
}

// --- Jobs ---

export async function listJobs(): Promise<Job[]> {
  return invoke<Job[]>("list_jobs");
}

export async function getJob(jobId: string): Promise<Job> {
  return invoke<Job>("get_job", { jobId });
}

export async function cancelJob(jobId: string): Promise<void> {
  return invoke("cancel_job", { jobId });
}

// --- DTCs ---

export async function readDtcs(componentId: string): Promise<DiagnosticTroubleCode[]> {
  return invoke<DiagnosticTroubleCode[]>("read_dtcs", { componentId });
}

export async function clearDtcs(componentId: string): Promise<void> {
  return invoke("clear_dtcs", { componentId });
}

// --- Diagnostics ---

export async function readData(componentId: string, dataId: string): Promise<DataValue> {
  return invoke<DataValue>("read_data", { componentId, dataId });
}

export async function writeData(componentId: string, dataId: string, value: unknown): Promise<void> {
  return invoke("write_data", { componentId, dataId, value });
}

// --- Monitoring ---

export async function getLiveData(componentId: string): Promise<DataValue[]> {
  // Backend returns serde_json::Value; we cast to DataValue[] on the frontend
  return invoke<DataValue[]>("get_live_data", { componentId });
}

// --- Plugins ---

export async function listPlugins(): Promise<PluginInfo[]> {
  return invoke<PluginInfo[]>("list_plugins");
}

// --- Events ---

export function onFlashProgress(callback: (progress: FlashProgress) => void): Promise<UnlistenFn> {
  return listen<FlashProgress>("flash_progress", (event) => {
    callback(event.payload);
  });
}

export function onLogEvent(callback: (entry: LogEntry) => void): Promise<UnlistenFn> {
  return listen<LogEntry>("log_event", (event) => {
    callback(event.payload);
  });
}
