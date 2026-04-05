import { useState, useCallback } from "react";
import { ScanIcon, BluetoothSlashIcon, BluetoothConnectedIcon, HeartIcon, GearSixIcon } from "@phosphor-icons/react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { StatusBar } from "@/components/StatusBar";
import { HeartRateDisplay } from "@/components/HeartRateDisplay";
import { DeviceList } from "@/components/DeviceList";
import { Settings } from "@/components/Settings";
import { UpdateBanner } from "@/components/UpdateBanner";
import {
  useStatus,
  useScanDevices,
  useConnectDevice,
  useDisconnectDevice,
  useConfig,
  usePlatform,
} from "@/hooks/useTauri";
import { useUpdater } from "@/hooks/useUpdater";

export default function App() {
  const { status, error: statusError } = useStatus();
  const { devices, scanning, error: scanError, scan, clearDevices } = useScanDevices();
  const { connecting, error: connectError, connect } = useConnectDevice();
  const { disconnecting, error: disconnectError, disconnect } = useDisconnectDevice();
  const { config, saving, error: configError, updateConfig, reload: reloadConfig } = useConfig();
  const { isDesktop } = usePlatform();
  const { status: updateStatus, downloadAndInstall, dismissUpdate } = useUpdater();

  const [connectingId, setConnectingId] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState(false);

  const connection = status?.connection ?? { status: "disconnected" as const };
  const currentData = status?.current_data ?? null;
  const isConnected = connection.status === "connected";
  const isBusy =
    connection.status === "scanning" ||
    connection.status === "connecting" ||
    scanning ||
    connecting ||
    disconnecting;

  const error = statusError || scanError || connectError || disconnectError || configError;
  const showDeviceList = scanning || devices.length > 0;

  const handleConnect = useCallback(async (deviceId: string, deviceName: string) => {
    setConnectingId(deviceId);
    const ok = await connect(deviceId, deviceName);
    setConnectingId(null);
    if (ok) {
      clearDevices();
      reloadConfig();
    }
  }, [connect, clearDevices, reloadConfig]);

  return (
    <div className="absolute inset-0 flex flex-col bg-background text-foreground overflow-hidden">
      <UpdateBanner status={updateStatus} onInstall={downloadAndInstall} onDismiss={dismissUpdate} />
      <StatusBar connection={connection} error={error}>
        <Button
          variant="ghost"
          size="icon-xs"
          onClick={() => { setShowSettings(false); scan(); }}
          disabled={isBusy}
          className="text-muted-foreground hover:text-foreground"
          title="Scan for devices"
          aria-label="Scan for devices"
        >
          <ScanIcon weight="duotone" className={scanning ? "animate-pulse" : ""} />
        </Button>

        {isConnected ? (
          <Button
            variant="ghost"
            size="icon-xs"
            onClick={disconnect}
            disabled={isBusy}
            className="text-muted-foreground hover:text-destructive"
            title="Disconnect"
            aria-label="Disconnect device"
          >
            <BluetoothSlashIcon weight="duotone" />
          </Button>
        ) : config?.last_device_id ? (
          <Button
            variant="ghost"
            size="icon-xs"
            disabled={isBusy}
            onClick={() => {
              if (config?.last_device_id) handleConnect(config.last_device_id, config.last_device_name ?? "Unknown");
            }}
            className="text-muted-foreground hover:text-foreground"
            title={`Reconnect to ${config.last_device_name ?? "last device"}`}
            aria-label={`Reconnect to ${config.last_device_name ?? "last device"}`}
          >
            <BluetoothConnectedIcon weight="duotone" />
          </Button>
        ) : null}

        <Button
          variant="ghost"
          size="icon-xs"
          onClick={() => {
            if (!showSettings) clearDevices();
            setShowSettings((v) => !v);
          }}
          className={cn(
            "text-muted-foreground hover:text-foreground transition-colors",
            showSettings && "text-foreground bg-accent/60"
          )}
          title="Settings"
          aria-label="Settings"
          aria-expanded={showSettings}
        >
          <GearSixIcon weight="duotone" />
        </Button>
      </StatusBar>

      <main className="flex-1 relative flex flex-col overflow-hidden">
        <HeartRateDisplay data={currentData} connected={isConnected} />

        {!isConnected && !showDeviceList && !showSettings && !isBusy && (
          <div className="absolute bottom-8 left-0 right-0 flex justify-center pointer-events-none">
            <p className="text-xs text-muted-foreground/70 text-center max-w-56 leading-relaxed">
              {config?.last_device_id
                ? "Click reconnect or scan for devices"
                : "Scan for nearby heart rate monitors to get started"}
            </p>
          </div>
        )}

        {showDeviceList && (
          <div className="absolute inset-x-0 bottom-0 z-20 bg-background/95 backdrop-blur-sm border-t border-border/60 shadow-[0_-4px_24px_rgba(0,0,0,0.4)]">
            <div className="px-4 py-3">
              <DeviceList
                devices={devices}
                scanning={scanning}
                onConnect={handleConnect}
                onDismiss={clearDevices}
                connectingId={connectingId}
              />
            </div>
          </div>
        )}

        {showSettings && (
          <div className="absolute inset-x-0 bottom-0 z-20 bg-background/95 backdrop-blur-sm border-t border-border/60 shadow-[0_-4px_24px_rgba(0,0,0,0.4)]">
            <div className="px-4 py-3">
              <Settings config={config} saving={saving} onSave={updateConfig} isDesktop={isDesktop} onDismiss={() => setShowSettings(false)} />
            </div>
          </div>
        )}
      </main>

      <footer className="flex items-center justify-center gap-1.5 px-4 py-1 select-none" role="contentinfo" aria-label="Credits">
        <span className="text-[10px] text-muted-foreground/50">made with</span>
        <HeartIcon size={10} weight="fill" className="text-rose-500/60" aria-hidden="true" />
        <span className="text-[10px] text-muted-foreground/50">by</span>
        <a
          href="https://bsky.app/profile/checksum.bsky.social"
          target="_blank"
          rel="noopener noreferrer"
          className="text-[10px] text-muted-foreground/70 hover:text-foreground transition-colors"
          aria-label="Made with love by @checksum on Bluesky, opens in new tab"
        >
          @checksum
        </a>
      </footer>
    </div>
  );
}
