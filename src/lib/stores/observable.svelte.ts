/**
 * Base class providing the observer pattern for stores that need
 * to notify the persistence layer when state changes.
 */
export class ObservableStore {
  private onChanged: (() => void) | null = null;

  /** Register a callback to be called when state changes (for save scheduling) */
  onStateChange(cb: () => void) {
    this.onChanged = cb;
  }

  protected notify() {
    this.onChanged?.();
  }
}
