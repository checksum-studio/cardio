import { useState, useEffect, useRef } from "react";
import { FloppyDiskIcon, CheckIcon, ArrowSquareOutIcon, XIcon } from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import type { AppConfig } from "@/types";

interface SettingsProps {
  config: AppConfig | null;
  saving: boolean;
  onSave: (config: AppConfig) => void;
  isDesktop: boolean;
  onDismiss: () => void;
}

export function Settings({ config, saving, onSave, isDesktop, onDismiss }: SettingsProps) {
  const [saved, setSaved] = useState(false);
  const savedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [host, setHost] = useState<string>(config?.host ?? "0.0.0.0");
  const [port, setPort] = useState<number>(config?.port ?? 2328);
  const [autoConnect, setAutoConnect] = useState<boolean>(
    config?.auto_connect ?? false
  );
  const [autoLaunch, setAutoLaunch] = useState<boolean>(
    config?.auto_launch ?? false
  );
  const [startMinimized, setStartMinimized] = useState<boolean>(
    config?.start_minimized ?? false
  );

  useEffect(() => {
    if (config) {
      setHost(config.host);
      setPort(config.port);
      setAutoConnect(config.auto_connect);
      setAutoLaunch(config.auto_launch);
      setStartMinimized(config.start_minimized);
    }
  }, [config]);

  const prevSavingRef = useRef(saving);
  useEffect(() => {
    if (prevSavingRef.current && !saving) {
      setSaved(true);
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
      savedTimerRef.current = setTimeout(() => setSaved(false), 1500);
    }
    prevSavingRef.current = saving;
    return () => {
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
    };
  }, [saving]);

  function handleSave() {
    if (!config) return;
    onSave({
      ...config,
      host,
      port,
      auto_connect: autoConnect,
      auto_launch: autoLaunch,
      start_minimized: startMinimized,
    });
  }

  const isDirty =
    config !== null &&
    (host !== config.host ||
      port !== config.port ||
      autoConnect !== config.auto_connect ||
      autoLaunch !== config.auto_launch ||
      startMinimized !== config.start_minimized);

  const activeHost = config?.host === "0.0.0.0" ? "localhost" : (config?.host ?? "localhost");
  const activePort = config?.port ?? 2328;

  return (
    <div className="flex flex-col gap-3" role="region" aria-label="Settings">
      <div className="flex items-center justify-between px-1 mb-1">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/90">
          Settings
        </h2>
        <Button
          variant="ghost"
          size="icon"
          className="h-5 w-5 rounded-sm text-muted-foreground/70 hover:text-foreground hover:bg-accent/60 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          onClick={onDismiss}
          aria-label="Close settings"
          title="Close settings"
        >
          <XIcon size={12} aria-hidden="true" />
        </Button>
      </div>

      <div className="flex flex-col gap-2.5">
        <div className="flex items-center justify-between gap-3">
          <label htmlFor="host-input" className="text-xs text-muted-foreground select-none font-medium">
            Listen Host
          </label>
          <Input
            id="host-input"
            type="text"
            value={host}
            onChange={(e) => setHost(e.target.value)}
            placeholder="0.0.0.0"
            className="w-32 h-7 px-2 text-xs font-mono text-right"
            aria-label="Listen Host"
          />
        </div>

        <div className="flex items-center justify-between gap-3">
          <label htmlFor="port-input" className="text-xs text-muted-foreground select-none font-medium">
            API Port
          </label>
          <Input
            id="port-input"
            type="number"
            min={1024}
            max={65535}
            value={port}
            onChange={(e) => setPort(Number(e.target.value))}
            className="w-24 h-7 px-2 text-xs font-mono text-right"
            aria-label="API Port"
          />
        </div>

        <div className="flex items-center justify-between gap-3">
          <label htmlFor="auto-connect" className="text-xs text-muted-foreground select-none cursor-pointer font-medium">
            Auto-connect
          </label>
          <Switch
            id="auto-connect"
            checked={autoConnect}
            onCheckedChange={setAutoConnect}
            className="scale-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
            aria-label="Toggle Auto-connect"
          />
        </div>

        {isDesktop && (
          <>
            <div className="flex items-center justify-between gap-3">
              <label htmlFor="auto-launch" className="text-xs text-muted-foreground select-none cursor-pointer font-medium">
                Start at login
              </label>
              <Switch
                id="auto-launch"
                checked={autoLaunch}
                onCheckedChange={setAutoLaunch}
                className="scale-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                aria-label="Toggle Start at login"
              />
            </div>

            <div className="flex items-center justify-between gap-3">
              <label htmlFor="start-minimized" className="text-xs text-muted-foreground select-none cursor-pointer font-medium">
                Start minimized
              </label>
              <Switch
                id="start-minimized"
                checked={startMinimized}
                onCheckedChange={setStartMinimized}
                className="scale-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                aria-label="Toggle Start minimized"
              />
            </div>
          </>
        )}
      </div>

      <Separator className="opacity-40" />

      <div className="rounded-md border border-border/40 bg-muted/20 px-3 py-2 flex flex-col gap-1.5" role="group" aria-label="API Endpoints">
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-medium text-muted-foreground/70 w-7 uppercase tracking-wider" aria-hidden="true">API</span>
          <a
            href={`http://${activeHost}:${activePort}/api/docs`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring rounded-sm"
            aria-label="Open API Documentation in a new tab"
          >
            <span aria-hidden="true">{activeHost}:{activePort}/api/docs</span>
            <ArrowSquareOutIcon size={12} weight="duotone" className="flex-shrink-0" aria-hidden="true" />
          </a>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-medium text-muted-foreground/70 w-7 uppercase tracking-wider" aria-hidden="true">WS</span>
          <span className="text-xs font-mono text-muted-foreground" aria-label={`WebSocket endpoint: ws://${activeHost}:${activePort}/api/ws`}>
            ws://{activeHost}:{activePort}/api/ws
          </span>
        </div>
        <div className="flex items-start gap-2">
          <span className="text-[10px] font-medium text-muted-foreground/70 w-7 uppercase tracking-wider mt-0.5" aria-hidden="true">OBS</span>
          <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
            <a
              href={`http://${activeHost}:${activePort}/overlay`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring rounded-sm"
              aria-label="Open standard OBS overlay in a new tab"
            >
              <span aria-hidden="true">/overlay</span>
              <ArrowSquareOutIcon size={12} weight="duotone" className="flex-shrink-0" aria-hidden="true" />
            </a>
            <span className="text-muted-foreground/50 text-xs" aria-hidden="true">|</span>
            <a
              href={`http://${activeHost}:${activePort}/overlay/chart`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring rounded-sm"
              aria-label="Open chart OBS overlay in a new tab"
            >
              <span aria-hidden="true">/chart</span>
              <ArrowSquareOutIcon size={12} weight="duotone" className="flex-shrink-0" aria-hidden="true" />
            </a>
            <span className="text-muted-foreground/50 text-xs" aria-hidden="true">|</span>
            <a
              href={`http://${activeHost}:${activePort}/overlay/ring`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring rounded-sm"
              aria-label="Open ring OBS overlay in a new tab"
            >
              <span aria-hidden="true">/ring</span>
              <ArrowSquareOutIcon size={12} weight="duotone" className="flex-shrink-0" aria-hidden="true" />
            </a>
            <span className="text-muted-foreground/50 text-xs" aria-hidden="true">|</span>
            <a
              href={`http://${activeHost}:${activePort}/overlay/ekg`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring rounded-sm"
              aria-label="Open EKG OBS overlay in a new tab"
            >
              <span aria-hidden="true">/ekg</span>
              <ArrowSquareOutIcon size={12} weight="duotone" className="flex-shrink-0" aria-hidden="true" />
            </a>
          </div>
        </div>
        <p className="text-[10px] text-muted-foreground/70 px-0.5 mt-0.5">
          Overlays accept <span className="font-mono">?color=HEX</span> for custom colors
        </p>
      </div>

      <div className="flex justify-end">
        <Button
          size="sm"
          onClick={handleSave}
          disabled={!isDirty || saving || config === null}
          className="gap-1.5 h-7 text-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
          aria-live="polite"
        >
          {saved ? (
            <CheckIcon size={14} weight="bold" className="text-green-400" aria-hidden="true" />
          ) : (
            <FloppyDiskIcon size={14} weight="duotone" aria-hidden="true" />
          )}
          {saving ? "Saving…" : saved ? "Saved" : "Save"}
        </Button>
      </div>
    </div>
  );
}
