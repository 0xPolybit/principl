"use client";

import { useSyncExternalStore } from "react";

type ThemePreference = "system" | "light" | "dark";

const storageKey = "princi-theme";
let fallbackTheme: ThemePreference = "system";

function isThemePreference(value: string | null): value is ThemePreference {
  return value === "system" || value === "light" || value === "dark";
}

function subscribeToTheme(onChange: () => void) {
  window.addEventListener("storage", onChange);
  window.addEventListener("princi-theme-change", onChange);
  return () => {
    window.removeEventListener("storage", onChange);
    window.removeEventListener("princi-theme-change", onChange);
  };
}

function readThemePreference(): ThemePreference {
  try {
    const savedTheme = window.localStorage.getItem(storageKey);
    if (isThemePreference(savedTheme)) return savedTheme;
  } catch {
    // Use the in-memory value when storage is disabled.
  }
  return fallbackTheme;
}

export function ThemeSwitcher() {
  const theme = useSyncExternalStore(
    subscribeToTheme,
    readThemePreference,
    () => "system",
  );

  function changeTheme(nextTheme: ThemePreference) {
    fallbackTheme = nextTheme;
    document.documentElement.dataset.theme = nextTheme;

    try {
      window.localStorage.setItem(storageKey, nextTheme);
    } catch {
      // Keep the current page usable when storage is disabled.
    }

    window.dispatchEvent(new Event("princi-theme-change"));
  }

  return (
    <label className="theme-control">
      <span className="theme-control-label">Theme</span>
      <select
        aria-label="Color theme"
        className="theme-select"
        onChange={(event) => changeTheme(event.target.value as ThemePreference)}
        value={theme}
      >
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </label>
  );
}
