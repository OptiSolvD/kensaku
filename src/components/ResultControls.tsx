import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Bookmark, Check, ChevronDown, LayoutGrid, Rows3 } from "lucide-react";

/* Right side of the toolbar: sort, layout, save. Same capsule material as
   the category segments.

   The sort control is an in-app listbox rather than a native <select>.
   A native select renders its option list in a separate OS window, which
   blurs the webview and makes the launcher hide itself on Linux and
   GTK-based desktops. Keeping the list inside the webview means choosing a
   sort never moves focus out of the window, and the control looks the same
   on every platform. */

export function ResultControls({
  sort,
  onSort,
  view,
  onView,
  onSave,
  canSave,
}: {
  sort: string;
  onSort: (value: string) => void;
  view: "grid" | "list";
  onView: (v: "grid" | "list") => void;
  onSave: () => void;
  canSave: boolean;
}) {
  const { t } = useTranslation("search");
  const SORT_OPTIONS = [
    { value: "best", label: t("sort.best") },
    { value: "newest", label: t("sort.newest") },
    { value: "oldest", label: t("sort.oldest") },
    { value: "largest", label: t("sort.largest") },
    { value: "name", label: t("sort.name") },
  ];

  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(() =>
    Math.max(0, SORT_OPTIONS.findIndex((o) => o.value === sort)),
  );
  const wrapRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const selected =
    SORT_OPTIONS.find((o) => o.value === sort) ?? SORT_OPTIONS[0];

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!wrapRef.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    return () => document.removeEventListener("mousedown", onDown);
  }, [open]);

  // Keep the highlight on the current value each time the list opens, and
  // return focus to the trigger when it closes so a keyboard user is not
  // dropped onto the page body.
  //
  // Only on a close, never on mount. This component mounts when the first
  // results arrive, and an unconditional `.focus()` on mount yanks focus out
  // of the search box mid-typing: the first character landed, then the caret
  // was gone. `wasOpen` makes the effect run only on an actual transition.
  const wasOpen = useRef(false);
  useEffect(() => {
    const justClosed = wasOpen.current && !open;
    wasOpen.current = open;

    if (open) {
      setActiveIndex(Math.max(0, SORT_OPTIONS.findIndex((o) => o.value === sort)));
      listRef.current?.focus({ preventScroll: true });
    } else if (justClosed) {
      buttonRef.current?.focus({ preventScroll: true });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const choose = (value: string) => {
    onSort(value);
    setOpen(false);
  };

  const onListKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      setOpen(false);
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIndex((i) => (i + 1) % SORT_OPTIONS.length);
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIndex(
        (i) => (i - 1 + SORT_OPTIONS.length) % SORT_OPTIONS.length,
      );
      return;
    }
    if (e.key === "Home") {
      e.preventDefault();
      setActiveIndex(0);
      return;
    }
    if (e.key === "End") {
      e.preventDefault();
      setActiveIndex(SORT_OPTIONS.length - 1);
      return;
    }
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      choose(SORT_OPTIONS[activeIndex].value);
      return;
    }
    if (e.key === "Tab") {
      setOpen(false);
    }
  };

  return (
    <div className="flex shrink-0 items-center gap-2">
      <div ref={wrapRef} className="relative">
        <button
          ref={buttonRef}
          type="button"
          aria-haspopup="listbox"
          aria-expanded={open}
          aria-label={t("sort.label")}
          title={t("sort.label")}
          onClick={() => setOpen((v) => !v)}
          className="control flex h-[38px] cursor-pointer items-center gap-1.5 rounded-control pl-3.5 pr-3 text-[13px] font-medium text-fg outline-none"
        >
          <span className="max-w-[110px] truncate">{selected.label}</span>
          <ChevronDown
            className={`h-3.5 w-3.5 shrink-0 text-subtle transition-transform ${open ? "rotate-180" : ""}`}
            strokeWidth={2}
          />
        </button>

        {open && (
          <div
            role="listbox"
            aria-label={t("sort.label")}
            ref={listRef}
            tabIndex={-1}
            onKeyDown={onListKeyDown}
            className="menu-pop elevated absolute right-0 z-[70] mt-1.5 w-[180px] overflow-hidden p-1"
          >
            {SORT_OPTIONS.map((option, index) => {
              const isSelected = option.value === sort;
              const isActive = index === activeIndex;
              return (
                <button
                  key={option.value}
                  type="button"
                  role="option"
                  aria-selected={isSelected}
                  onMouseEnter={() => setActiveIndex(index)}
                  onClick={() => choose(option.value)}
                  data-active={isActive ? "true" : undefined}
                  className="menu-row w-full justify-between"
                  style={{ textAlign: "left" }}
                >
                  <span className="truncate">{option.label}</span>
                  {isSelected && (
                    <Check className="h-3.5 w-3.5 shrink-0 text-subtle" strokeWidth={2} />
                  )}
                </button>
              );
            })}
          </div>
        )}
      </div>

      <div role="radiogroup" aria-label={t("view.layout")} className="control flex items-center gap-0.5 rounded-control p-1">
        <button type="button" role="radio" aria-checked={view === "grid"} aria-label={t("view.gridShort")} title={t("view.gridShort")} onClick={() => onView("grid")} className="seg !px-2.5">
          <LayoutGrid className="h-4 w-4" strokeWidth={1.75} />
        </button>
        <button type="button" role="radio" aria-checked={view === "list"} aria-label={t("view.listShort")} title={t("view.listShort")} onClick={() => onView("list")} className="seg !px-2.5">
          <Rows3 className="h-4 w-4" strokeWidth={1.75} />
        </button>
      </div>

      {canSave && (
        <button type="button" title={t("saveSearch")} aria-label={t("saveSearch")} onClick={onSave} className="control flex h-[38px] w-[38px] items-center justify-center rounded-control text-muted transition-colors hover:text-fg">
          <Bookmark className="h-4 w-4" strokeWidth={1.75} />
        </button>
      )}
    </div>
  );
}
