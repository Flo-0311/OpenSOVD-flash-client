import { Check, Loader2, Circle } from "lucide-react";
import type { JobPhase } from "../types";

const phases: JobPhase[] = ["PreCheck", "Deployment", "Monitoring", "Verification", "Reporting"];
const phaseLabels: Record<JobPhase, string> = {
  PreCheck: "Pre-Check",
  Deployment: "Deployment",
  Monitoring: "Monitoring",
  Verification: "Verification",
  Reporting: "Reporting",
};

interface Props {
  currentPhase: JobPhase;
  completed?: boolean;
  failed?: boolean;
}

export default function PhaseIndicator({ currentPhase, completed, failed }: Props) {
  const currentIdx = phases.indexOf(currentPhase);

  return (
    <div className="flex items-center gap-1">
      {phases.map((phase, idx) => {
        const isDone = completed || idx < currentIdx;
        const isCurrent = !completed && idx === currentIdx;
        const isFailed = failed && isCurrent;

        return (
          <div key={phase} className="flex items-center gap-1">
            {idx > 0 && (
              <div className={`h-0.5 w-6 ${isDone ? "bg-green-500" : "bg-border"}`} />
            )}
            <div className="flex flex-col items-center gap-1">
              <div
                className={`flex h-7 w-7 items-center justify-center rounded-full text-xs font-medium ${
                  isFailed
                    ? "bg-destructive text-destructive-foreground"
                    : isDone
                      ? "bg-green-500 text-white"
                      : isCurrent
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted text-muted-foreground"
                }`}
              >
                {isDone ? (
                  <Check className="h-3.5 w-3.5" />
                ) : isCurrent ? (
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                ) : (
                  <Circle className="h-3 w-3" />
                )}
              </div>
              <span className={`text-[10px] ${isCurrent ? "font-semibold text-foreground" : "text-muted-foreground"}`}>
                {phaseLabels[phase]}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
