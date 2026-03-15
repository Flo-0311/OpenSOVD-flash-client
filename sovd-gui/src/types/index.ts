export type ComponentType = "NativeSovd" | "ClassicUds" | "Unknown";
export type ComponentStatus = "Available" | "Busy" | "Error" | "Offline" | "Unknown";

export interface SovdComponent {
  id: string;
  name: string;
  component_type: ComponentType;
  status: ComponentStatus;
  software_version?: string;
  hardware_version?: string;
}

export type JobType = "Flash" | "DiagnosticRead" | "DiagnosticWrite" | "DtcRead" | "DtcClear" | "SoftwareUpdate" | "BulkFlash";
export type JobState = "Pending" | "Running" | "Completed" | "Failed" | "Cancelled" | "Paused";
export type JobPhase = "PreCheck" | "Deployment" | "Monitoring" | "Verification" | "Reporting";

export interface Job {
  id: string;
  job_type: JobType;
  target_component: string;
  state: JobState;
  phase: JobPhase;
  progress_percent?: number;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export type DtcStatus = "Active" | "Pending" | "Confirmed" | "Cleared";
export type DtcSeverity = "Info" | "Warning" | "Error" | "Critical";

export interface DiagnosticTroubleCode {
  id: string;
  code: string;
  description?: string;
  status: DtcStatus;
  severity?: DtcSeverity;
  component_id?: string;
}

export interface DataValue {
  id: string;
  name?: string;
  value: unknown;
  unit?: string;
  timestamp?: string;
}

export interface SoftwarePackage {
  name: string;
  version: string;
  target_component: string;
}

export interface CapabilitySummary {
  total: number;
  flashing: number;
  diagnostics: number;
  fault_management: number;
  configuration: number;
  provisioning: number;
  monitoring: number;
  logging: number;
  bulk: number;
  other: number;
  sovd_version?: string;
}

export interface FlashProgress {
  job_id: string;
  phase: JobPhase;
  percent: number;
  message?: string;
}

export interface PluginInfo {
  name: string;
  version: string;
  plugin_type: string;
  description: string;
}

export interface LogEntry {
  timestamp: string;
  level: "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";
  message: string;
  target?: string;
}

