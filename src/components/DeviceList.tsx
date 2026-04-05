import { WifiLowIcon, SpinnerGapIcon, XIcon, WifiHighIcon, WifiMediumIcon, WifiNoneIcon } from "@phosphor-icons/react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { DEVICE_TYPE_LABELS } from "@/constants";
import type { ScanResult } from "@/types";

interface DeviceListProps {
  devices: ScanResult[];
  scanning: boolean;
  onConnect: (deviceId: string, deviceName: string) => void;
  onDismiss: () => void;
  connectingId: string | null;
}

function RssiIcon({ rssi }: { rssi: number | null }) {
  if (rssi === null) return <WifiNoneIcon size={14} weight="duotone" className="text-muted-foreground/70" />;
  if (rssi >= -60) return <WifiHighIcon size={14} weight="duotone" className="text-emerald-400" />;
  if (rssi >= -75) return <WifiMediumIcon size={14} weight="duotone" className="text-yellow-400" />;
  return <WifiLowIcon size={14} weight="duotone" className="text-orange-400" />;
}

function rssiLabel(rssi: number | null): string {
  if (rssi === null) return "-";
  return `${rssi} dBm`;
}

function deviceTypeLabel(deviceType: string): string {
  return DEVICE_TYPE_LABELS[deviceType] ?? deviceType;
}

export function DeviceList({
  devices,
  scanning,
  onConnect,
  onDismiss,
  connectingId,
}: DeviceListProps) {
  const showHeader = scanning || devices.length > 0;

  return (
    <div className="flex flex-col gap-1" role="region" aria-label="Device List">
      {showHeader && (
        <div className="flex items-center justify-between px-1">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/90">
            Devices
          </h2>
          <Button
            variant="ghost"
            size="icon"
            className="h-5 w-5 rounded-sm text-muted-foreground/70 hover:text-foreground hover:bg-accent/60 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            onClick={onDismiss}
            aria-label="Close device list"
            title="Close device list"
          >
            <XIcon size={12} aria-hidden="true" />
          </Button>
        </div>
      )}

      {scanning && (
        <div className="flex flex-col items-center justify-center gap-2 py-6 text-muted-foreground" role="status" aria-live="polite">
          <SpinnerGapIcon size={20} weight="bold" className="animate-spin text-muted-foreground/80" aria-hidden="true" />
          <span className="text-xs font-medium">Scanning for devices…</span>
        </div>
      )}

      {!scanning && devices.length === 0 && (
        <div className="flex flex-col items-center justify-center gap-1 py-6" role="status" aria-live="polite">
          <p className="text-xs font-medium text-muted-foreground/90">No devices found</p>
          <p className="text-xs text-muted-foreground/70 text-center">Make sure your device is nearby and try again</p>
        </div>
      )}

      {!scanning && devices.length > 0 && (
        <ScrollArea className="max-h-48">
          <ul className="flex flex-col gap-0.5 pr-3" aria-label="Available devices">
            {devices.map((result) => {
              const { id, name, device_type } = result.device_info;
              const isConnecting = connectingId === id;
              const deviceLabel = name || "Unknown Device";

              return (
                <li key={id}>
                  <button
                    onClick={() => onConnect(id, name)}
                    disabled={!!connectingId}
                    aria-label={`Connect to ${deviceLabel}. Device type: ${deviceTypeLabel(device_type)}. Signal strength: ${rssiLabel(result.rssi)}${isConnecting ? ". Connecting..." : ""}`}
                    className={cn(
                      "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left",
                      "border border-transparent transition-all duration-150",
                      "hover:bg-accent/60 hover:border-border/50",
                      "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                      "disabled:opacity-50 disabled:cursor-not-allowed",
                      isConnecting && "bg-accent/40 border-border/30"
                    )}
                  >
                    <RssiIcon rssi={result.rssi} />

                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-semibold truncate leading-tight">
                        {name || "Unknown Device"}
                      </p>
                      <p className="text-xs text-muted-foreground/90 font-mono leading-tight mt-0.5 break-all">
                        {id}
                      </p>
                    </div>

                    <div className="flex items-center gap-1.5 flex-shrink-0">
                      <Badge
                        variant="secondary"
                        className="text-xs px-1.5 py-0 h-auto font-normal"
                      >
                        {deviceTypeLabel(device_type)}
                      </Badge>
                      <span className="text-xs text-muted-foreground/80 font-mono w-16 text-right tabular-nums">
                        {rssiLabel(result.rssi)}
                      </span>
                      {isConnecting ? (
                        <SpinnerGapIcon size={14} weight="bold" className="animate-spin text-muted-foreground/80" aria-hidden="true" />
                      ) : (
                        <div className="h-3.5 w-3.5" aria-hidden="true" />
                      )}
                    </div>
                  </button>
                </li>
              );
            })}
          </ul>
        </ScrollArea>
      )}
    </div>
  );
}
