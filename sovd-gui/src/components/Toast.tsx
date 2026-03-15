import { useState, useEffect, useCallback, createContext, useContext } from "react";
import { CheckCircle, XCircle, AlertTriangle, Info, X } from "lucide-react";

type ToastType = "success" | "error" | "warning" | "info";

interface ToastItem {
  id: number;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
}

interface ToastContextValue {
  toast: (type: ToastType, title: string, message?: string, duration?: number) => void;
}

const ToastContext = createContext<ToastContextValue>({
  toast: () => {},
});

export const useToast = () => useContext(ToastContext);

let nextId = 0;

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const toast = useCallback(
    (type: ToastType, title: string, message?: string, duration = 4000) => {
      const id = ++nextId;
      setToasts((prev) => [...prev, { id, type, title, message, duration }]);
    },
    [],
  );

  const dismiss = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ toast }}>
      {children}
      <div className="fixed right-4 top-4 z-50 flex flex-col gap-2" role="log" aria-live="polite">
        {toasts.map((t) => (
          <ToastCard key={t.id} item={t} onDismiss={dismiss} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

const icons: Record<ToastType, typeof CheckCircle> = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
};

const styles: Record<ToastType, string> = {
  success: "border-green-500/30 bg-green-50 dark:bg-green-950/50 text-green-800 dark:text-green-200",
  error: "border-red-500/30 bg-red-50 dark:bg-red-950/50 text-red-800 dark:text-red-200",
  warning: "border-yellow-500/30 bg-yellow-50 dark:bg-yellow-950/50 text-yellow-800 dark:text-yellow-200",
  info: "border-blue-500/30 bg-blue-50 dark:bg-blue-950/50 text-blue-800 dark:text-blue-200",
};

const iconStyles: Record<ToastType, string> = {
  success: "text-green-600 dark:text-green-400",
  error: "text-red-600 dark:text-red-400",
  warning: "text-yellow-600 dark:text-yellow-400",
  info: "text-blue-600 dark:text-blue-400",
};

function ToastCard({ item, onDismiss }: { item: ToastItem; onDismiss: (id: number) => void }) {
  const [exiting, setExiting] = useState(false);
  const Icon = icons[item.type];

  useEffect(() => {
    if (!item.duration) return;
    const timer = setTimeout(() => setExiting(true), item.duration);
    return () => clearTimeout(timer);
  }, [item.duration]);

  useEffect(() => {
    if (!exiting) return;
    const timer = setTimeout(() => onDismiss(item.id), 200);
    return () => clearTimeout(timer);
  }, [exiting, item.id, onDismiss]);

  return (
    <div
      className={`animate-slide-down flex w-80 items-start gap-3 rounded-lg border p-3 shadow-lg backdrop-blur transition-all duration-200 ${
        styles[item.type]
      } ${exiting ? "translate-x-4 opacity-0" : ""}`}
      role="alert"
    >
      <Icon className={`mt-0.5 h-4 w-4 shrink-0 ${iconStyles[item.type]}`} />
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium">{item.title}</p>
        {item.message && (
          <p className="mt-0.5 text-xs opacity-80">{item.message}</p>
        )}
      </div>
      <button
        onClick={() => setExiting(true)}
        className="shrink-0 rounded p-0.5 opacity-60 transition-opacity hover:opacity-100"
        aria-label="Dismiss"
      >
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
