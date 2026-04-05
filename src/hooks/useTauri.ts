import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppStatus, ScanResult, AppConfig, Platform } from "@/types";

export function usePlatform() {
  const [platform, setPlatform] = useState<Platform>("unknown");

  useEffect(() => {
    invoke<Platform>("get_platform").then(setPlatform).catch(() => {});
  }, []);

  const isDesktop = platform === "windows" || platform === "macos" || platform === "linux";
  const isMobile = platform === "android" || platform === "ios";

  return { platform, isDesktop, isMobile };
}

export function useStatus() {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function poll() {
      try {
        const result = await invoke<AppStatus>("get_status");
        if (!cancelled) {
          setStatus(result);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    }

    poll();
    const id = setInterval(poll, 1000);

    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  return { status, error };
}

export function useScanDevices() {
  const [devices, setDevices] = useState<ScanResult[]>([]);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const scanIdRef = useRef<number>(0);

  const scan = useCallback(async () => {
    const scanId = ++scanIdRef.current;
    setScanning(true);
    setError(null);
    setDevices([]);
    try {
      const results = await invoke<ScanResult[]>("scan_devices");
      if (scanId === scanIdRef.current) {
        setDevices(results);
      }
    } catch (err) {
      if (scanId === scanIdRef.current) {
        setError(err instanceof Error ? err.message : String(err));
      }
    } finally {
      if (scanId === scanIdRef.current) {
        setScanning(false);
      }
    }
  }, []);

  const clearDevices = useCallback(() => {
    scanIdRef.current++;
    setScanning(false);
    setDevices([]);
  }, []);

  return { devices, scanning, error, scan, clearDevices };
}

export function useConnectDevice() {
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(async (deviceId: string, deviceName: string): Promise<boolean> => {
    setConnecting(true);
    setError(null);
    try {
      await invoke("connect_device", { deviceId, deviceName });
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    } finally {
      setConnecting(false);
    }
  }, []);

  return { connecting, error, connect };
}

export function useDisconnectDevice() {
  const [disconnecting, setDisconnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const disconnect = useCallback(async () => {
    setDisconnecting(true);
    setError(null);
    try {
      await invoke("disconnect_device");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setDisconnecting(false);
    }
  }, []);

  return { disconnecting, error, disconnect };
}

export function useConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const result = await invoke<AppConfig>("get_config");
      setConfig(result);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const updateConfig = useCallback(async (next: AppConfig) => {
    setSaving(true);
    setError(null);
    try {
      await invoke("update_config", { config: next });
      setConfig(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  }, []);

  return { config, saving, error, updateConfig, reload: load };
}
