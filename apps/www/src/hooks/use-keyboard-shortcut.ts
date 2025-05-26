"use client";

import { useEffect, useCallback } from "react";

type KeyboardShortcutOptions = {
  metaKey?: boolean;
  ctrlKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
};

export function useKeyboardShortcut(
  key: string,
  callback: () => void,
  options: KeyboardShortcutOptions = {}
) {
  const {
    metaKey = false,
    ctrlKey = false,
    altKey = false,
    shiftKey = false,
  } = options;

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (
        event.key.toLowerCase() === key.toLowerCase() &&
        event.metaKey === metaKey &&
        event.ctrlKey === ctrlKey &&
        event.altKey === altKey &&
        event.shiftKey === shiftKey
      ) {
        event.preventDefault();
        callback();
      }
    },
    [key, callback, metaKey, ctrlKey, altKey, shiftKey]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);
}
