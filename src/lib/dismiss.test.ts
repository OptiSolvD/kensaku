import { describe, expect, it } from "vitest";
import { INTERACTION_GRACE_MS, shouldHideOnBlur, type BlurState } from "./dismiss";

/* Regression tests for #95: the palette hid itself on right-click and when the
   sort control opened its native popup. Both blur the webview without the user
   leaving the app, so they must not dismiss the launcher. */

const launcher: BlurState = {
  setupComplete: true,
  appMode: "launcher",
  overlayOpen: false,
  documentFocused: false,
  msSinceInteraction: 10_000,
};

describe("shouldHideOnBlur", () => {
  it("hides when the user clicks away to another app", () => {
    expect(shouldHideOnBlur(launcher)).toBe(true);
  });

  it("does not hide when the document still reports focus", () => {
    expect(
      shouldHideOnBlur({ ...launcher, documentFocused: true }),
    ).toBe(false);
  });

  it("does not hide right after a pointer press (a popup is opening)", () => {
    expect(
      shouldHideOnBlur({ ...launcher, msSinceInteraction: 40 }),
    ).toBe(false);
    expect(
      shouldHideOnBlur({ ...launcher, msSinceInteraction: INTERACTION_GRACE_MS - 1 }),
    ).toBe(false);
  });

  it("hides again once the grace window has passed", () => {
    expect(
      shouldHideOnBlur({ ...launcher, msSinceInteraction: INTERACTION_GRACE_MS }),
    ).toBe(true);
  });

  it("never hides before the setup wizard has finished", () => {
    expect(
      shouldHideOnBlur({ ...launcher, setupComplete: false }),
    ).toBe(false);
  });

  it("never hides an app-mode window, or one whose config has not arrived", () => {
    expect(shouldHideOnBlur({ ...launcher, appMode: "app" })).toBe(false);
    expect(shouldHideOnBlur({ ...launcher, appMode: null })).toBe(false);
  });

  it("never hides while a modal, dialog or in-app menu is open", () => {
    expect(shouldHideOnBlur({ ...launcher, overlayOpen: true })).toBe(false);
  });
});
