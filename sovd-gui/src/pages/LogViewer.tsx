import { useState, useEffect, useRef, useCallback } from "react";
import { FileText, Search, Trash2, ArrowDownToLine } from "lucide-react";
import { useSettingsStore } from "../stores/settingsStore";
import * as tauri from "../lib/tauri";
import type { LogEntry } from "../types";

const levelColor: Record<string, string> = {
  ERROR: "text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950",
  WARN: "text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-950",
  INFO: "text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-950",
  DEBUG: "text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-900",
  TRACE: "text-gray-400 dark:text-gray-500 bg-gray-50 dark:bg-gray-900",
};

let logCounter = 0;

export default function LogViewer() {
  const maxLogEntries = useSettingsStore((s) => s.maxLogEntries);
  const defaultLogLevel = useSettingsStore((s) => s.logLevel);
  const [logs, setLogs] = useState<{ entry: LogEntry; key: number }[]>([]);
  const [filter, setFilter] = useState("");
  const [levelFilter, setLevelFilter] = useState<string>(defaultLogLevel);
  const [pinToBottom, setPinToBottom] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlisten = tauri.onLogEvent((entry) => {
      setLogs((prev) => [...prev.slice(-(maxLogEntries - 1)), { entry, key: ++logCounter }]);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [maxLogEntries]);

  useEffect(() => {
    if (pinToBottom && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs, pinToBottom]);

  const handleScroll = useCallback(() => {
    if (!scrollRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 40;
    if (pinToBottom !== isAtBottom) setPinToBottom(isAtBottom);
  }, [pinToBottom]);

  const filtered = logs.filter(({ entry: l }) => {
    if (levelFilter !== "ALL" && l.level !== levelFilter) return false;
    if (filter && !l.message.toLowerCase().includes(filter.toLowerCase())) return false;
    return true;
  });

  const errorCount = logs.filter(({ entry: l }) => l.level === "ERROR").length;
  const warnCount = logs.filter(({ entry: l }) => l.level === "WARN").length;

  return (
    <div className="flex h-full flex-col space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="page-title">
            Logs
            {logs.length > 0 && (
              <span className="ml-2 align-middle text-base font-normal text-muted-foreground">
                ({filtered.length}{filtered.length !== logs.length ? ` of ${logs.length}` : ""})
              </span>
            )}
          </h1>
          <p className="page-description">
            {logs.length === 0
              ? "Waiting for log events..."
              : `${logs.length} entries${errorCount > 0 ? ` · ${errorCount} errors` : ""}${warnCount > 0 ? ` · ${warnCount} warnings` : ""}`}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={levelFilter}
            onChange={(e) => setLevelFilter(e.target.value)}
            className="input h-8 w-24 text-xs"
          >
            <option value="ALL">ALL</option>
            <option value="ERROR">ERROR</option>
            <option value="WARN">WARN</option>
            <option value="INFO">INFO</option>
            <option value="DEBUG">DEBUG</option>
            <option value="TRACE">TRACE</option>
          </select>
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Filter messages..."
              className="input h-8 w-48 pl-8 text-xs"
            />
          </div>
          <button
            onClick={() => {
              if (scrollRef.current) {
                scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
              }
              setPinToBottom(true);
            }}
            className={`btn-secondary btn-sm ${pinToBottom ? "text-primary" : ""}`}
            title="Scroll to bottom"
          >
            <ArrowDownToLine className="h-3.5 w-3.5" />
          </button>
          <button
            onClick={() => setLogs([])}
            disabled={logs.length === 0}
            className="btn-secondary btn-sm"
          >
            <Trash2 className="h-3.5 w-3.5" />
            Clear
          </button>
        </div>
      </div>

      {filtered.length === 0 ? (
        <div className="flex flex-1 flex-col items-center justify-center rounded-lg border border-dashed text-center">
          <div className="rounded-full bg-muted p-3">
            <FileText className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="mt-3 font-semibold">No Log Entries</h3>
          <p className="mt-1 max-w-xs text-sm text-muted-foreground">
            {logs.length > 0
              ? "No entries match the current filters. Try adjusting the level or search."
              : "Log entries will appear here in real-time as they are emitted."}
          </p>
        </div>
      ) : (
        <div
          ref={scrollRef}
          onScroll={handleScroll}
          className="flex-1 overflow-y-auto rounded-lg border bg-card font-mono text-xs"
        >
          {filtered.map(({ entry: log, key }) => (
            <div
              key={key}
              className="flex items-start gap-3 border-b px-3 py-1.5 last:border-0 hover:bg-muted/50"
            >
              <span className="w-20 shrink-0 tabular-nums text-muted-foreground">
                {new Date(log.timestamp).toLocaleTimeString()}
              </span>
              <span className={`w-14 shrink-0 rounded px-1.5 py-0.5 text-center text-[10px] font-bold ${levelColor[log.level] ?? ""}`}>
                {log.level}
              </span>
              {log.target && (
                <span className="w-32 shrink-0 truncate text-muted-foreground">{log.target}</span>
              )}
              <span className="flex-1 break-all">{log.message}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
