type KeyboardHandler = (e: KeyboardEvent) => void;

interface ShortcutEntry {
  key: string;
  meta?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: KeyboardHandler;
}

const shortcuts: ShortcutEntry[] = [];

export function registerShortcut(entry: ShortcutEntry): () => void {
  shortcuts.push(entry);
  return () => {
    const idx = shortcuts.indexOf(entry);
    if (idx >= 0) shortcuts.splice(idx, 1);
  };
}

export function initKeyboardShortcuts(): () => void {
  const listener = (e: KeyboardEvent) => {
    for (const s of shortcuts) {
      const metaMatch = s.meta ? e.metaKey || e.ctrlKey : true;
      const shiftMatch = s.shift ? e.shiftKey : !e.shiftKey;
      const altMatch = s.alt ? e.altKey : !e.altKey;
      if (
        e.key.toLowerCase() === s.key.toLowerCase() &&
        metaMatch &&
        shiftMatch &&
        altMatch
      ) {
        e.preventDefault();
        s.handler(e);
        return;
      }
    }
  };

  window.addEventListener("keydown", listener);
  return () => window.removeEventListener("keydown", listener);
}
