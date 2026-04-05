import { useRef, useEffect } from "react";
import { HeartIcon } from "@phosphor-icons/react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { HrEvent } from "@/types";

const PULSE_ATTACK = 0.15;
const PULSE_DECAY = 0.45;
const PULSE_LERP_SPEED = 0.08;
const GLOW_BASE_OPACITY = 0.15;
const GLOW_PEAK_OPACITY = 0.45;
const GLOW_BASE_SPREAD = 24;
const GLOW_PEAK_SPREAD = 16;
const GLOW_BASE_SHADOW = 0.2;
const GLOW_PEAK_SHADOW = 0.3;
const RING_PEAK_SCALE = 0.35;
const RING_PEAK_OPACITY = 0.5;

interface HeartRateDisplayProps {
  data: HrEvent | null;
  connected: boolean;
}

function usePulse(
  bpm: number | null,
  active: boolean,
  glowRef: React.RefObject<HTMLSpanElement | null>,
  ringRef: React.RefObject<HTMLSpanElement | null>,
) {
  const targetIntervalRef = useRef(1000);
  const currentIntervalRef = useRef(1000);
  const lastBeatRef = useRef(0);
  const rafRef = useRef(0);

  useEffect(() => {
    if (bpm && bpm > 0) {
      targetIntervalRef.current = (60 / bpm) * 1000;
    }
  }, [bpm]);

  useEffect(() => {
    if (!active) {
      cancelAnimationFrame(rafRef.current);
      if (glowRef.current) {
        glowRef.current.style.opacity = "";
        glowRef.current.style.filter = "";
      }
      if (ringRef.current) {
        ringRef.current.style.transform = "scale(1)";
        ringRef.current.style.opacity = "0";
      }
      return;
    }

    lastBeatRef.current = performance.now();
    currentIntervalRef.current = targetIntervalRef.current;

    function tick(now: number) {
     
      const cur = currentIntervalRef.current;
      const target = targetIntervalRef.current;
      currentIntervalRef.current = cur + (target - cur) * PULSE_LERP_SPEED;

      const interval = currentIntervalRef.current;
      const elapsed = now - lastBeatRef.current;

      if (elapsed >= interval) {
        lastBeatRef.current = now - (elapsed % interval);
      }

      const t = (now - lastBeatRef.current) / interval;
      const phase = t < PULSE_ATTACK ? t / PULSE_ATTACK : Math.max(0, 1 - (t - PULSE_ATTACK) / PULSE_DECAY);

      if (glowRef.current) {
        const glowOpacity = GLOW_BASE_OPACITY + phase * GLOW_PEAK_OPACITY;
        const shadowSpread = GLOW_BASE_SPREAD + phase * GLOW_PEAK_SPREAD;
        const shadowOpacity = (GLOW_BASE_SHADOW + phase * GLOW_PEAK_SHADOW).toFixed(2);
        glowRef.current.style.opacity = String(glowOpacity);
        glowRef.current.style.filter = `drop-shadow(0 0 ${shadowSpread}px oklch(0.65 0.22 25 / ${shadowOpacity}))`;
      }
      if (ringRef.current) {
        const ringScale = 1 + phase * RING_PEAK_SCALE;
        const ringOpacity = phase * RING_PEAK_OPACITY;
        ringRef.current.style.transform = `scale(${ringScale})`;
        ringRef.current.style.opacity = String(ringOpacity);
      }

      rafRef.current = requestAnimationFrame(tick);
    }

    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, [active, glowRef, ringRef]);
}

export function HeartRateDisplay({ data, connected }: HeartRateDisplayProps) {
  const bpm = data?.heart_rate ?? null;
  const rrIntervals = data?.rr_intervals ?? [];

  const lastBpmRef = useRef<number | null>(null);
  if (bpm !== null) lastBpmRef.current = bpm;

  const isActive = connected && bpm !== null;
  const displayBpm = bpm ?? lastBpmRef.current;
  const isStale = displayBpm !== null && !isActive;
  const isEmpty = displayBpm === null && !connected;

  const glowRef = useRef<HTMLSpanElement>(null);
  const ringRef = useRef<HTMLSpanElement>(null);

  usePulse(bpm, isActive, glowRef, ringRef);

  return (
    <div className="flex flex-col items-center justify-center flex-1 gap-4 select-none">
      <div
        className={cn(
          "relative flex items-center justify-center w-52 h-52 rounded-full",
          "ring-1 ring-inset ring-white/5"
        )}
      >
        <span
          ref={glowRef}
          aria-hidden="true"
          className="absolute inset-0 rounded-full bg-[radial-gradient(ellipse_at_center,_oklch(0.22_0.04_10)_0%,_oklch(0.14_0.02_10)_70%)] will-change-[opacity,filter]"
        />

        <span
          ref={ringRef}
          aria-hidden="true"
          className="absolute inset-0 rounded-full border border-rose-500/40 will-change-transform"
          style={{ transform: "scale(1)", opacity: 0 }}
        />

        <span
          className={cn(
            "relative z-10 font-mono tabular-nums leading-none tracking-tight transition-opacity duration-500",
            displayBpm !== null && displayBpm >= 100 ? "text-7xl" : "text-8xl",
            isActive && "text-foreground",
            isStale && "text-foreground opacity-50",
            isEmpty && "text-muted-foreground/70"
          )}
          role="status"
          aria-live="polite"
          aria-label={displayBpm !== null ? `Heart rate: ${displayBpm} beats per minute` : "No heart rate data available"}
        >
          {displayBpm !== null ? displayBpm : "--"}
        </span>
      </div>

      <div className="flex items-center gap-1.5" aria-hidden="true">
        <HeartIcon
          size={14}
          weight={isActive ? "fill" : "regular"}
          className={cn(
            "transition-colors duration-300",
            isActive ? "text-rose-500" : "text-muted-foreground/70"
          )}
        />
        <span
          className={cn(
            "text-sm font-medium uppercase tracking-widest transition-colors duration-300",
            isActive ? "text-muted-foreground" : "text-muted-foreground/70"
          )}
        >
          bpm
        </span>
        {data?.signal_quality != null && (
          <Badge
            variant="outline"
            className="ml-1 px-1.5 py-0 text-[10px] font-mono text-muted-foreground/60 border-muted-foreground/20"
          >
            {data.signal_quality}
          </Badge>
        )}
      </div>

      {rrIntervals.length > 0 && (
        <Tooltip>
          <TooltipTrigger
            render={<button type="button" className="text-xs text-muted-foreground/80 font-mono mt-1 cursor-help border-b border-dotted border-muted-foreground/50 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring focus-visible:ring-offset-1 rounded-sm" />}
            aria-label="View RR Intervals info"
          >
            <span className="text-muted-foreground/60 mr-1.5" aria-hidden="true">RR</span>
            <span className="sr-only">RR Intervals: </span>
            {rrIntervals
              .slice(0, 6)
              .map((v) => `${v}ms`)
              .join(" ")}
          </TooltipTrigger>
          <TooltipContent side="bottom" className="max-w-xs text-center">
            <p className="text-xs text-muted-foreground">
              Time between heartbeats (ms)
            </p>
          </TooltipContent>
        </Tooltip>
      )}

      {isEmpty && (
        <p className="text-xs text-muted-foreground/70 tracking-wide mt-1">
          No device connected
        </p>
      )}
    </div>
  );
}
