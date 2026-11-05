/**
 * When a window blur should hide the launcher.
 *
 * A blur is only a dismissal when the user moved focus to another
 * application. Several things blur the webview without the user leaving:
 *
 * - A right-click opens a native context menu on platforms where the webview
 *   hands context menus to the toolkit.
 * - A control that opens an OS-rendered popup (the sort dropdown used to be a
 *   native `<select>`) takes focus for as long as the popup is open.
 *
 * Hiding on those makes the window disappear under the user's cursor, which
 * is the bug reported in #95. The rules below keep the escape hatch honest:
 * clicking another window still dismisses, because by the time the event is
 * delivered `document.hasFocus()` is false and no picker is mid-open.
 */

export interface BlurState {
  /** The setup wizard owns the window; it must never be hidden from under it. */
  setupComplete: boolean;
  /** App mode is a normal window, not a launcher; blur never hides it. */
  appMode: "launcher" | "app" | null;
  /** A modal or in-app menu is open and owns the interaction. */
  overlayOpen: boolean;
  /** `document.hasFocus()` at the moment the blur fires. */
  documentFocused: boolean;
  /** Milliseconds since the last pointerdown or contextmenu on the window. */
  msSinceInteraction: number;
}

/**
 * A popup opened by a press steals focus within a frame or two of that press.
 * Waiting this long after the press distinguishes "a popup just opened" from
 * "the user clicked away", which happens well after any press.
 */
export const INTERACTION_GRACE_MS = 350;

export function shouldHideOnBlur(state: BlurState): boolean {
  if (!state.setupComplete) return false;
  // The config may not have arrived yet on a fresh window; never treat an
  // app-mode window as a launcher in that gap.
  if (state.appMode === null || state.appMode === "app") return false;
  if (state.overlayOpen) return false;
  if (state.documentFocused) return false;
  if (state.msSinceInteraction < INTERACTION_GRACE_MS) return false;
  return true;
}
