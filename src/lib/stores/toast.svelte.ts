class ToastStore {
  message = $state<string | null>(null);
  private timer: ReturnType<typeof setTimeout> | null = null;

  show(message: string, duration = 2000) {
    this.message = message;
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.message = null;
    }, duration);
  }
}

export const toastStore = new ToastStore();
