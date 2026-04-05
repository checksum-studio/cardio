import { useState, useEffect, useCallback } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "available"; version: string; body: string | undefined }
  | { state: "downloading"; progress: number; total: number }
  | { state: "ready" }
  | { state: "error"; message: string };

export function useUpdater() {
  const [status, setStatus] = useState<UpdateStatus>({ state: "idle" });
  const [update, setUpdate] = useState<Update | null>(null);

  useEffect(() => {
    checkForUpdate();
    const id = setInterval(checkForUpdate, 4 * 60 * 60 * 1000);
    return () => clearInterval(id);
  }, []);

  const checkForUpdate = useCallback(async () => {
    try {
      setStatus({ state: "checking" });
      const found = await check();
      if (found) {
        setUpdate(found);
        setStatus({
          state: "available",
          version: found.version,
          body: found.body ?? undefined,
        });
      } else {
        setStatus({ state: "idle" });
      }
    } catch (err) {
      setStatus({
        state: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    if (!update) return;
    try {
      let downloaded = 0;
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            setStatus({
              state: "downloading",
              progress: 0,
              total: event.data.contentLength ?? 0,
            });
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            setStatus((prev) => ({
              state: "downloading",
              progress: downloaded,
              total: prev.state === "downloading" ? prev.total : 0,
            }));
            break;
          case "Finished":
            setStatus({ state: "ready" });
            break;
        }
      });
      await relaunch();
    } catch (err) {
      setStatus({
        state: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }, [update]);

  const dismissUpdate = useCallback(() => {
    setStatus({ state: "idle" });
    setUpdate(null);
  }, []);

  return { status, checkForUpdate, downloadAndInstall, dismissUpdate };
}
