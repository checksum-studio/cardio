import { useState, useEffect } from "react";
import { BatteryLowIcon, BatteryMediumIcon, BatteryFullIcon, XIcon } from "@phosphor-icons/react";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/types";

interface StatusBarProps {
  connection: ConnectionStatus;
  error: string | null;
  children?: React.ReactNode;
}

function BatteryIcon({ level }: { level: number }) {
  if (level <= 20) return <BatteryLowIcon size={14} weight="duotone" className="text-rose-400" aria-hidden="true" />;
  if (level <= 50) return <BatteryMediumIcon size={14} weight="duotone" className="text-yellow-400" aria-hidden="true" />;
  if (level <= 80) return <BatteryFullIcon size={14} weight="duotone" className="text-emerald-400" aria-hidden="true" />;
  return <BatteryFullIcon size={14} weight="fill" className="text-emerald-300 drop-shadow-[0_0_4px_var(--color-emerald-300)]" aria-hidden="true" />;
}

function StatusDot({ status }: { status: ConnectionStatus["status"] }) {
  return (
    <span
      className={cn(
        "inline-block h-2 w-2 rounded-full flex-shrink-0 transition-colors duration-300",
        status === "connected" &&
          "bg-emerald-400 shadow-[0_0_6px_var(--color-emerald-400)]",
        (status === "scanning" || status === "connecting") &&
          "bg-yellow-400 shadow-[0_0_6px_var(--color-yellow-400)] animate-pulse",
        status === "disconnected" && "bg-zinc-600"
      )}
      aria-hidden="true"
    />
  );
}

const STATUS_LABEL: Record<ConnectionStatus["status"], string> = {
  disconnected: "Disconnected",
  scanning: "Scanning…",
  connecting: "Connecting…",
  connected: "Connected",
};

export function StatusBar({ connection, error, children }: StatusBarProps) {
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    if (error !== null) {
      setDismissed(false);
    }
  }, [error]);

  const deviceName =
    connection.status === "connected" ? connection.device_name : null;
  const batteryLevel =
    connection.status === "connected" ? connection.battery_level : null;

  const showError = error !== null && !dismissed;

  return (
    <header className="flex flex-col flex-shrink-0" role="banner">
      <div
        data-tauri-drag-region
        className="flex items-center gap-2 px-3 py-2.5 border-b border-border/60 bg-card/80 backdrop-blur-sm select-none"
      >
        <StatusDot status={connection.status} />

        <span className="text-xs font-semibold tracking-wide text-muted-foreground uppercase" role="status" aria-live="polite">
          <span className="sr-only">Connection status: </span>
          {STATUS_LABEL[connection.status]}
        </span>

        {deviceName && (
          <>
            <span className="text-muted-foreground/50 text-xs" aria-hidden="true">·</span>
            <span className="text-xs font-medium text-foreground/90 truncate max-w-[200px]" title={`Connected to ${deviceName}`}>
              {deviceName}
            </span>
          </>
        )}

        {batteryLevel != null && (
          <Badge
            variant="secondary"
            className="gap-1 px-1.5 py-0.5 text-xs font-mono h-auto"
            title={`Battery level: ${batteryLevel}%`}
            aria-label={`Battery level: ${batteryLevel}%`}
          >
            <BatteryIcon level={batteryLevel} />
            {batteryLevel}%
          </Badge>
        )}

        <div className="flex-1" data-tauri-drag-region />

        {children && (
          <nav className="flex items-center gap-0.5" aria-label="Application controls">
            {children}
          </nav>
        )}
      </div>

      {showError && (
        <div className="flex items-center gap-2 px-3 py-1.5 bg-rose-600/90 backdrop-blur-sm text-white text-xs font-medium" role="alert" aria-live="assertive">
          <span className="flex-1 truncate">{error}</span>
          <button
            onClick={() => setDismissed(true)}
            className="flex-shrink-0 rounded p-0.5 hover:bg-white/20 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white"
            aria-label="Dismiss error"
            title="Dismiss error"
          >
            <XIcon size={14} aria-hidden="true" />
          </button>
        </div>
      )}
    </header>
  );
}
