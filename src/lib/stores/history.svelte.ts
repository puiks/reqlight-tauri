import { MAX_HISTORY_ENTRIES } from "../constants";
import type { RequestHistoryEntry } from "../types";

class HistoryStore {
  history = $state<RequestHistoryEntry[]>([]);

  private onChanged: (() => void) | null = null;

  onStateChange(cb: () => void) {
    this.onChanged = cb;
  }

  private notify() {
    this.onChanged?.();
  }

  addEntry(entry: Omit<RequestHistoryEntry, "id" | "timestamp">) {
    const newEntry: RequestHistoryEntry = {
      id: crypto.randomUUID(),
      ...entry,
      timestamp: new Date().toISOString(),
    };
    this.history = [newEntry, ...this.history].slice(0, MAX_HISTORY_ENTRIES);
    this.notify();
  }

  clear() {
    this.history = [];
    this.notify();
  }
}

export const historyStore = new HistoryStore();
