import { createEmptyPair, type RequestEnvironment } from "../types";
import { ObservableStore } from "./observable.svelte";

const SECRET_PLACEHOLDER = "\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022";

class EnvironmentStore extends ObservableStore {
  environments = $state<RequestEnvironment[]>([]);
  activeEnvironmentId = $state<string | null>(null);

  /**
   * Mask secret values in environments so they don't sit in reactive state.
   * Call after loading from Rust backend.
   */
  maskSecrets() {
    this.environments = this.environments.map((env) => ({
      ...env,
      variables: env.variables.map((v) =>
        v.isSecret && v.value ? { ...v, value: SECRET_PLACEHOLDER } : v,
      ),
    }));
  }

  get activeEnvironment(): RequestEnvironment | undefined {
    if (!this.activeEnvironmentId) return undefined;
    return this.environments.find((e) => e.id === this.activeEnvironmentId);
  }

  addEnvironment(name = "New Environment") {
    const env: RequestEnvironment = {
      id: crypto.randomUUID(),
      name,
      variables: [createEmptyPair()],
    };
    this.environments = [...this.environments, env];
    this.notify();
    return env;
  }

  updateEnvironment(env: RequestEnvironment) {
    this.environments = this.environments.map((e) =>
      e.id === env.id ? env : e,
    );
    this.notify();
  }

  deleteEnvironment(id: string) {
    this.environments = this.environments.filter((e) => e.id !== id);
    if (this.activeEnvironmentId === id) {
      this.activeEnvironmentId = null;
    }
    this.notify();
  }

  setActiveEnvironment(id: string | null) {
    this.activeEnvironmentId = id;
    this.notify();
  }
}

export const environmentStore = new EnvironmentStore();
