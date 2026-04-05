import { ArrowCircleDownIcon, XIcon, CircleNotchIcon } from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
import type { UpdateStatus } from "@/hooks/useUpdater";

interface UpdateBannerProps {
  status: UpdateStatus;
  onInstall: () => void;
  onDismiss: () => void;
}

export function UpdateBanner({ status, onInstall, onDismiss }: UpdateBannerProps) {
  if (status.state !== "available" && status.state !== "downloading") return null;

  return (
    <div className="flex items-center gap-2 px-3 py-1.5 bg-accent/80 text-xs text-foreground border-b border-border/60">
      {status.state === "available" && (
        <>
          <ArrowCircleDownIcon size={14} weight="duotone" className="text-foreground shrink-0" />
          <span className="flex-1 truncate">
            v{status.version} available
          </span>
          <Button variant="ghost" size="icon-xs" onClick={onInstall} title="Install update">
            <ArrowCircleDownIcon size={14} weight="fill" />
          </Button>
          <Button variant="ghost" size="icon-xs" onClick={onDismiss} title="Dismiss">
            <XIcon size={14} />
          </Button>
        </>
      )}
      {status.state === "downloading" && (
        <>
          <CircleNotchIcon size={14} className="animate-spin shrink-0" />
          <span className="flex-1">
            Downloading update...
            {status.total > 0 && ` ${Math.round((status.progress / status.total) * 100)}%`}
          </span>
        </>
      )}
    </div>
  );
}
