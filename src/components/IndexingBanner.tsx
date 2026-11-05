import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { Loader2 } from "lucide-react";
import type { IndexingStatus } from "../types";
import { formatComma } from "../lib/format";

/* Horizontal announcement bar for the settings page.
 *
 * Indexing can start from anywhere (launch, a config change, an OCR pack
 * finishing, the watcher noticing an overflow) and used to be invisible from
 * Settings: the panel covered the status line in the main window, so a scan
 * looked like the app had hung. This sits at the top of the panel for as long
 * as there is work, and says which phase and how far along it is. */

type Phase = IndexingStatus["phase"];

function describe(
  status: IndexingStatus,
  t: TFunction<"settings">,
): { label: string; detail: string } {
  const phase: Phase = status.phase ?? (status.is_indexing ? "extracting" : "idle");

  switch (phase) {
    case "walking": {
      const total = status.total_files ?? 0;
      return {
        label: t("indexingBanner.walking"),
        detail: total
          ? t("indexingBanner.foundFiles", { count: formatComma(total) })
          : t("indexingBanner.checking"),
      };
    }
    case "extracting": {
      const done = status.extract_done ?? status.files_indexed;
      const total = status.extract_total ?? status.total_files;
      return {
        label: t("indexingBanner.extracting"),
        detail: total
          ? t("indexingBanner.progress", {
              done: formatComma(done),
              total: formatComma(total),
            })
          : t("indexingBanner.progressUnknown", { done: formatComma(done) }),
      };
    }
    case "embedding":
      return {
        label: t("indexingBanner.embedding"),
        detail: t("indexingBanner.progressUnknown", {
          done: formatComma(status.units_indexed ?? status.files_indexed),
        }),
      };
    case "committing":
      return { label: t("indexingBanner.committing"), detail: t("indexingBanner.almostDone") };
    case "rebuilding":
      return {
        label: t("indexingBanner.rebuilding"),
        detail: status.current_status || t("indexingBanner.checking"),
      };
    case "paused":
    case "idle":
    case "error":
    default:
      return { label: t("indexingBanner.working"), detail: status.current_status ?? "" };
  }
}

function percentOf(status: IndexingStatus): number | null {
  const phase: Phase = status.phase ?? (status.is_indexing ? "extracting" : "idle");
  if (phase === "extracting") {
    const total = status.extract_total ?? status.total_files ?? 0;
    const done = status.extract_done ?? status.files_indexed;
    if (total > 0) return Math.min(100, Math.round((done / total) * 100));
    return null;
  }
  if (phase === "walking") {
    return null;
  }
  return null;
}

export function IndexingBanner() {
  const { t } = useTranslation("settings");
  const [status, setStatus] = useState<IndexingStatus | null>(null);

  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | undefined;

    invoke<IndexingStatus>("get_indexing_status")
      .then((initial) => {
        if (alive) setStatus(initial);
      })
      .catch(() => undefined);

    listen<IndexingStatus>("indexing-status", (event) => {
      if (alive) setStatus(event.payload);
    })
      .then((fn) => {
        if (alive) unlisten = fn;
        else fn();
      })
      .catch(() => undefined);

    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  if (!status) return null;

  const busy = status.is_indexing || (status.phase != null && status.phase !== "idle");
  if (!busy) return null;

  const { label, detail } = describe(status, t);
  const percent = percentOf(status);

  return (
    <div
      role="status"
      aria-live="polite"
      className="relative flex w-full shrink-0 items-center gap-3 border-t border-line bg-accent-soft px-4 py-2"
    >
      <Loader2 className="h-3.5 w-3.5 shrink-0 animate-spin text-muted" strokeWidth={2} />
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <p className="text-[12px] font-medium text-fg">{label}</p>
          <p className="truncate text-[11px] text-muted">{detail}</p>
        </div>
        {/* Indeterminate while walking: the total is not known yet. */}
        <div className="mt-1.5 h-0.5 w-full overflow-hidden rounded-full bg-[var(--border)]">
          {percent === null ? (
            <div className="progress-bar" />
          ) : (
            <div
              className="h-full rounded-full bg-fg transition-[width] duration-300"
              style={{ width: `${percent}%` }}
            />
          )}
        </div>
      </div>
      <p className="shrink-0 tabular-nums text-[11px] text-subtle">
        {percent === null ? t("indexingBanner.workingShort") : `${percent}%`}
      </p>
      <p className="sr-only">{t("indexingBanner.screenReader")}</p>
    </div>
  );
}
