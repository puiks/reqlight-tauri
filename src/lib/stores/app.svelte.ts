import { loadState, saveState } from "../commands";
import { MAX_HISTORY_ENTRIES, SAVE_DEBOUNCE_MS } from "../constants";
import {
  createEmptyBody,
  createEmptyPair,
  type AppState,
  type HttpMethod,
  type RequestCollection,
  type RequestEnvironment,
  type RequestHistoryEntry,
  type SavedRequest,
} from "../types";
import { handleError } from "../utils/errors";

class AppStore {
  collections = $state<RequestCollection[]>([]);
  environments = $state<RequestEnvironment[]>([]);
  activeEnvironmentId = $state<string | null>(null);
  selectedCollectionId = $state<string | null>(null);
  selectedRequestId = $state<string | null>(null);
  history = $state<RequestHistoryEntry[]>([]);
  sidebarVisible = $state(true);
  searchQuery = $state("");
  toastMessage = $state<string | null>(null);
  isLoading = $state(false);

  private saveTimer: ReturnType<typeof setTimeout> | null = null;

  // Derived
  get activeEnvironment(): RequestEnvironment | undefined {
    if (!this.activeEnvironmentId) return undefined;
    return this.environments.find((e) => e.id === this.activeEnvironmentId);
  }

  get selectedRequest(): SavedRequest | undefined {
    if (!this.selectedRequestId) return undefined;
    for (const c of this.collections) {
      const r = c.requests.find((r) => r.id === this.selectedRequestId);
      if (r) return r;
    }
    return undefined;
  }

  get selectedCollection(): RequestCollection | undefined {
    if (!this.selectedCollectionId) return undefined;
    return this.collections.find(
      (c) => c.id === this.selectedCollectionId,
    );
  }

  get filteredCollections(): RequestCollection[] {
    if (!this.searchQuery.trim()) return this.collections;
    const q = this.searchQuery.toLowerCase();
    return this.collections
      .map((c) => ({
        ...c,
        requests: c.requests.filter(
          (r) =>
            r.name.toLowerCase().includes(q) ||
            r.url.toLowerCase().includes(q),
        ),
      }))
      .filter(
        (c) =>
          c.name.toLowerCase().includes(q) || c.requests.length > 0,
      );
  }

  // Load
  async load() {
    this.isLoading = true;
    try {
      const state = await loadState();
      this.collections = state.collections;
      this.environments = state.environments;
      this.activeEnvironmentId = state.activeEnvironmentId;
      this.selectedCollectionId = state.lastSelectedCollectionId;
      this.selectedRequestId = state.lastSelectedRequestId;
      this.history = state.history;
    } catch (e) {
      handleError(e, "AppStore.load", { silent: true });
    } finally {
      this.isLoading = false;
    }
  }

  // Debounced save
  scheduleSave() {
    if (this.saveTimer) clearTimeout(this.saveTimer);
    this.saveTimer = setTimeout(() => this.save(), SAVE_DEBOUNCE_MS);
  }

  // Immediately flush any pending debounced save (for window close)
  flushSave() {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
      this.save();
    }
  }

  private async save() {
    const state: AppState = {
      collections: this.collections,
      environments: this.environments,
      activeEnvironmentId: this.activeEnvironmentId,
      lastSelectedCollectionId: this.selectedCollectionId,
      lastSelectedRequestId: this.selectedRequestId,
      history: this.history,
    };
    try {
      await saveState(state);
    } catch (e) {
      handleError(e, "AppStore.save");
    }
  }

  // Collection CRUD
  addCollection(name = "New Collection") {
    const collection: RequestCollection = {
      id: crypto.randomUUID(),
      name,
      requests: [],
      sortOrder: this.collections.length,
      createdAt: new Date().toISOString(),
    };
    this.collections = [...this.collections, collection];
    this.selectedCollectionId = collection.id;
    this.scheduleSave();
    this.showToast(`Collection "${name}" created`);
    return collection;
  }

  renameCollection(id: string, name: string) {
    this.collections = this.collections.map((c) =>
      c.id === id ? { ...c, name } : c,
    );
    this.scheduleSave();
  }

  deleteCollection(id: string) {
    const name = this.collections.find((c) => c.id === id)?.name;
    this.collections = this.collections.filter((c) => c.id !== id);
    if (this.selectedCollectionId === id) {
      this.selectedCollectionId = null;
      this.selectedRequestId = null;
    }
    this.scheduleSave();
    if (name) this.showToast(`Collection "${name}" deleted`);
  }

  // Request CRUD
  addRequest(collectionId: string, name = "New Request") {
    const request: SavedRequest = {
      id: crypto.randomUUID(),
      name,
      method: "GET",
      url: "",
      queryParams: [createEmptyPair()],
      headers: [createEmptyPair()],
      body: createEmptyBody(),
      sortOrder: 0,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    this.collections = this.collections.map((c) => {
      if (c.id === collectionId) {
        request.sortOrder = c.requests.length;
        return { ...c, requests: [...c.requests, request] };
      }
      return c;
    });
    this.selectedCollectionId = collectionId;
    this.selectedRequestId = request.id;
    this.scheduleSave();
    return request;
  }

  updateRequest(request: SavedRequest) {
    this.collections = this.collections.map((c) => ({
      ...c,
      requests: c.requests.map((r) =>
        r.id === request.id ? { ...request, updatedAt: new Date().toISOString() } : r,
      ),
    }));
    this.scheduleSave();
  }

  deleteRequest(requestId: string) {
    let name: string | undefined;
    this.collections = this.collections.map((c) => {
      const r = c.requests.find((r) => r.id === requestId);
      if (r) name = r.name;
      return {
        ...c,
        requests: c.requests.filter((r) => r.id !== requestId),
      };
    });
    if (this.selectedRequestId === requestId) {
      this.selectedRequestId = null;
    }
    this.scheduleSave();
    if (name) this.showToast(`Request "${name}" deleted`);
  }

  duplicateRequest(requestId: string) {
    for (const c of this.collections) {
      const original = c.requests.find((r) => r.id === requestId);
      if (original) {
        const duplicate: SavedRequest = {
          ...structuredClone(original),
          id: crypto.randomUUID(),
          name: `${original.name} (Copy)`,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        };
        this.collections = this.collections.map((col) =>
          col.id === c.id
            ? { ...col, requests: [...col.requests, duplicate] }
            : col,
        );
        this.selectedRequestId = duplicate.id;
        this.scheduleSave();
        this.showToast(`Request duplicated`);
        return duplicate;
      }
    }
    return null;
  }

  selectRequest(collectionId: string, requestId: string) {
    this.selectedCollectionId = collectionId;
    this.selectedRequestId = requestId;
    this.scheduleSave();
  }

  reorderRequest(collectionId: string, fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    this.collections = this.collections.map((c) => {
      if (c.id !== collectionId) return c;
      const requests = [...c.requests];
      const [moved] = requests.splice(fromIndex, 1);
      requests.splice(toIndex, 0, moved);
      return { ...c, requests };
    });
    this.scheduleSave();
  }

  // History
  addHistoryEntry(entry: Omit<RequestHistoryEntry, "id" | "timestamp">) {
    const newEntry: RequestHistoryEntry = {
      id: crypto.randomUUID(),
      ...entry,
      timestamp: new Date().toISOString(),
    };
    this.history = [newEntry, ...this.history].slice(
      0,
      MAX_HISTORY_ENTRIES,
    );
    this.scheduleSave();
  }

  clearHistory() {
    this.history = [];
    this.scheduleSave();
    this.showToast("History cleared");
  }

  // Environment
  addEnvironment(name = "New Environment") {
    const env: RequestEnvironment = {
      id: crypto.randomUUID(),
      name,
      variables: [createEmptyPair()],
    };
    this.environments = [...this.environments, env];
    this.scheduleSave();
    return env;
  }

  updateEnvironment(env: RequestEnvironment) {
    this.environments = this.environments.map((e) =>
      e.id === env.id ? env : e,
    );
    this.scheduleSave();
  }

  deleteEnvironment(id: string) {
    this.environments = this.environments.filter((e) => e.id !== id);
    if (this.activeEnvironmentId === id) {
      this.activeEnvironmentId = null;
    }
    this.scheduleSave();
  }

  setActiveEnvironment(id: string | null) {
    this.activeEnvironmentId = id;
    this.scheduleSave();
  }

  // Toast
  private toastTimer: ReturnType<typeof setTimeout> | null = null;
  showToast(message: string) {
    this.toastMessage = message;
    if (this.toastTimer) clearTimeout(this.toastTimer);
    this.toastTimer = setTimeout(() => {
      this.toastMessage = null;
    }, 2000);
  }
}

export const appStore = new AppStore();
