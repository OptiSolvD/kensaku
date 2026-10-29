import React from "react";
import type { IndexingStatus, DoctorReport } from "../types";
import { formatComma } from "../lib/format";
import { Stethoscope, ArrowUpCircle } from "lucide-react";

/* Status line. Index health on the left, the few keys worth knowing on the
   right. Quiet by default: it only takes the ink while indexing. */

const HINTS: { keys: string; label: string }[] = [
  { keys: "↵", label: "Open" },
  { keys: "Space", label: "Preview" },
  { keys: "⇧↵", label: "Reveal" },
  { keys: "⌘I", label: "Details" },
];

function statusText(s: IndexingStatus | null | undefined): { text: string; busy: boolean } {
  if (!s) return { text: "", busy: false };
  if (s.paused) return { text: "Indexing paused", busy: false };
  if (s.is_indexing || (s.phase && s.phase !== "idle")) {
    if (s.phase === "rebuilding") return { text: s.current_status || "Rebuilding index…", busy: true };
    /* Background semantic embedding runs after the crawl. Showing it as
       "Indexing N of N" (the finished crawl's counts) looked stuck. */
    if (s.phase === "embedding" && !s.is_indexing)
      return { text: `${formatComma(s.files_indexed)} files · building semantic index`, busy: true };
    const done = s.extract_done ?? s.files_indexed;
    const total = s.extract_total ?? s.total_files;
    return {
      text: total ? `Indexing ${formatComma(done)} of ${formatComma(total)}` : `Indexing ${formatComma(done)} files`,
      busy: true,
    };
  }
  return { text: `${formatComma(s.files_indexed)} files`, busy: false };
}

export const KeyHintStrip: React.FC<{
  status?: IndexingStatus | null;
  doctor?: DoctorReport | null;
  updateAvailable?: string | null;
  onDoctor?: () => void;
  onUpdate?: () => void;
}> = ({ status, doctor, updateAvailable, onDoctor, onUpdate }) => {
  const { text, busy } = statusText(status);
  const issues = doctor ? doctor.summary.warn + doctor.summary.fail : 0;
  return (
    <div className="relative z-10 flex h-7 shrink-0 items-center justify-between gap-4 px-3.5 text-[11px] text-subtle">
      <span className="flex items-center gap-2">
        {text && (
          <>
            <span className={`h-1.5 w-1.5 rounded-full ${busy ? "dot-pulse bg-ink" : "bg-success/70"}`} />
            <span className="t-meta">{text}</span>
          </>
        )}
        {doctor && (
          <button
            type="button"
            onClick={onDoctor}
            title={issues ? `${issues} health issue${issues === 1 ? "" : "s"} — open Health` : "Index health OK"}
            className={`flex items-center gap-1 rounded-full px-1.5 py-0.5 transition-colors hover:bg-surface-2 ${issues ? "text-warning" : "text-subtle"}`}
          >
            <Stethoscope className="h-3.5 w-3.5" />
            {issues > 0 && <span className="t-meta">{issues} issue{issues === 1 ? "" : "s"}</span>}
          </button>
        )}
        {updateAvailable && (
          <button
            type="button"
            onClick={onUpdate}
            title={`Update available: ${updateAvailable}`}
            className="flex items-center gap-1 rounded-full px-1.5 py-0.5 text-accent transition-colors hover:bg-surface-2"
          >
            <ArrowUpCircle className="h-3.5 w-3.5" />
            <span className="t-meta">Update</span>
          </button>
        )}
      </span>
      <span className="hidden items-center gap-4 sm:flex">
        {HINTS.map((h) => (
          <span key={h.label} className="flex items-center gap-1.5">
            <span className="kbd">{h.keys}</span>
            <span>{h.label}</span>
          </span>
        ))}
      </span>
    </div>
  );
};
