import { loadState, saveState } from "../commands";
import { SAVE_DEBOUNCE_MS } from "../constants";
import {
  createEmptyBody,
  createEmptyPair,
  type AppState,
  type RequestCollection,
  type SavedRequest,
} from "../types";
import { handleError } from "../utils/errors";
import { environmentStore } from "./environment.svelte";
import { historyStore } from "./history.svelte";
import { toastStore } from "./toast.svelte";

class AppStore {
  collections = $state<RequestCollection[]>([]);
  selectedCollectionId = $state<string | null>(null);
  selectedRequestId = $state<string | null>(null);
  sidebarVisible = $state(true);
  searchQuery = $state("");
  isLoading = $state(false);

  private saveTimer: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    // Wire sub-stores to trigger save on change
    environmentStore.onStateChange(() => this.scheduleSave());
    historyStore.onStateChange(() => this.scheduleSave());
  }

  // Derived
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
    return this.collections.find((c) => c.id === this.selectedCollectionId);
  }

  get filteredCollections(): RequestCollection[] {
    if (!this.searchQuery.trim()) return this.collections;
    const q = this.searchQuery.toLowerCase();
    return this.collections
      .map((c) => {
        const collectionMatches = c.name.toLowerCase().includes(q);
        return {
          ...c,
          requests: collectionMatches
            ? c.requests
            : c.requests.filter(
                (r) =>
                  r.name.toLowerCase().includes(q) ||
                  r.url.toLowerCase().includes(q),
              ),
        };
      })
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
      // Sort collections and their requests by sortOrder on load
      this.collections = state.collections
        .sort((a, b) => a.sortOrder - b.sortOrder)
        .map((c) => ({
          ...c,
          requests: [...c.requests].sort((a, b) => a.sortOrder - b.sortOrder),
        }));
      this.selectedCollectionId = state.lastSelectedCollectionId;
      this.selectedRequestId = state.lastSelectedRequestId;
      environmentStore.environments = state.environments;
      environmentStore.activeEnvironmentId = state.activeEnvironmentId;
      // Mask secret values so they don't linger in reactive state
      environmentStore.maskSecrets();
      historyStore.history = state.history;
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
      environments: environmentStore.environments,
      activeEnvironmentId: environmentStore.activeEnvironmentId,
      lastSelectedCollectionId: this.selectedCollectionId,
      lastSelectedRequestId: this.selectedRequestId,
      history: historyStore.history,
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
    toastStore.show(`Collection "${name}" created`);
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
    if (name) toastStore.show(`Collection "${name}" deleted`);
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
    if (name) toastStore.show(`Request "${name}" deleted`);
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
        toastStore.show(`Request duplicated`);
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

  reorderCollection(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    const cols = [...this.collections];
    const [moved] = cols.splice(fromIndex, 1);
    cols.splice(toIndex, 0, moved);
    this.collections = cols.map((c, i) => ({ ...c, sortOrder: i }));
    this.scheduleSave();
  }

  reorderRequest(collectionId: string, fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    this.collections = this.collections.map((c) => {
      if (c.id !== collectionId) return c;
      const requests = [...c.requests];
      const [moved] = requests.splice(fromIndex, 1);
      requests.splice(toIndex, 0, moved);
      // Sync sortOrder to match new array positions
      return { ...c, requests: requests.map((r, i) => ({ ...r, sortOrder: i })) };
    });
    this.scheduleSave();
  }
}

export const appStore = new AppStore();
