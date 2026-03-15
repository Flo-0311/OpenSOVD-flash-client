import { useState, useRef, useEffect } from "react";
import { Plug, Unplug, Loader2, Eye, EyeOff, XCircle, AlertTriangle } from "lucide-react";
import { useConnectionStore } from "../stores/connectionStore";
import { useToast } from "./Toast";
import * as tauri from "../lib/tauri";
import { sanitizeError } from "../lib/tauri";

export default function ConnectionBar() {
  const {
    url, token, connected, connecting, error,
    setUrl, setToken, setConnected, setConnecting,
    setError, setCapabilities, setSovdVersion, setComponents, reset,
  } = useConnectionStore();
  const [showToken, setShowToken] = useState(false);
  const { toast } = useToast();
  const urlRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!connected && urlRef.current) urlRef.current.focus();
  }, [connected]);

  const handleConnect = async () => {
    if (!url || connecting) return;
    try {
      const parsed = new URL(url);
      if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
        setError("Only http:// and https:// URLs are allowed");
        return;
      }
    } catch {
      setError("Invalid URL format");
      return;
    }
    setConnecting(true);
    setError(null);
    try {
      const caps = await tauri.connectToServer(url, token || undefined);
      setCapabilities(caps);
      setSovdVersion(caps.sovd_version ?? null);
      setConnected(true);
      const components = await tauri.listComponents();
      setComponents(components);
      toast("success", "Connected", `Successfully connected to ${url}`);
    } catch (e) {
      const msg = sanitizeError(e);
      setError(msg);
      setConnected(false);
      toast("error", "Connection Failed", msg);
    } finally {
      setConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    try { await tauri.disconnect(); } catch { /* ignore */ }
    reset();
    toast("info", "Disconnected", "Server connection closed");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !connected) handleConnect();
  };

  return (
    <div className="flex items-center gap-2 border-b bg-card px-4 py-2">
      <div className="flex flex-1 items-center gap-2">
        <input
          ref={urlRef}
          type="url"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={connected || connecting}
          placeholder="http://sovd-server:8080"
          className="input h-8 min-w-0 flex-1 max-w-sm font-mono text-xs"
          aria-label="SOVD Server URL"
          autoComplete="off"
        />
        <div className="relative">
          <input
            type={showToken ? "text" : "password"}
            value={token}
            onChange={(e) => setToken(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={connected || connecting}
            placeholder="Auth Token (optional)"
            className="input h-8 w-48 pr-8 text-xs"
            aria-label="Authentication token"
          />
          <button
            type="button"
            onClick={() => setShowToken(!showToken)}
            className="absolute right-1.5 top-1/2 -translate-y-1/2 rounded p-1 text-muted-foreground/60 hover:text-muted-foreground"
            tabIndex={-1}
            aria-label={showToken ? "Hide token" : "Show token"}
          >
            {showToken ? <EyeOff className="h-3.5 w-3.5" /> : <Eye className="h-3.5 w-3.5" />}
          </button>
        </div>
      </div>

      {!connected ? (
        <button
          onClick={handleConnect}
          disabled={connecting || !url}
          className="btn-primary btn-sm shrink-0"
        >
          {connecting ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <Plug className="h-3.5 w-3.5" />
          )}
          {connecting ? "Connecting..." : "Connect"}
        </button>
      ) : (
        <button
          onClick={handleDisconnect}
          className="btn-secondary btn-sm shrink-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
        >
          <Unplug className="h-3.5 w-3.5" />
          Disconnect
        </button>
      )}

      {!connected && token && url && url.startsWith("http://") && (
        <div className="flex items-center gap-1.5 rounded-md bg-yellow-500/10 px-2.5 py-1 text-xs text-yellow-700 dark:text-yellow-400">
          <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
          <span>Token sent in cleartext over HTTP</span>
        </div>
      )}

      {error && (
        <div className="flex items-center gap-1.5 rounded-md bg-destructive/10 px-2.5 py-1 text-xs text-destructive">
          <XCircle className="h-3.5 w-3.5 shrink-0" />
          <span className="max-w-xs truncate">{error}</span>
          <button
            onClick={() => setError(null)}
            className="ml-1 shrink-0 rounded p-0.5 hover:bg-destructive/20"
            aria-label="Dismiss error"
          >
            <XCircle className="h-3 w-3" />
          </button>
        </div>
      )}
    </div>
  );
}
