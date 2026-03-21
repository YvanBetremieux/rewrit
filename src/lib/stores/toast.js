import { writable } from 'svelte/store';

export const toastStore = writable({
  visible: false,
  message: '',
  type: 'info', // 'info', 'success', 'error', 'warning'
  duration: 3000
});

let toastTimer = null;

export function showToast(message, type = 'info', duration = 3000) {
  if (toastTimer) clearTimeout(toastTimer);

  toastStore.set({
    visible: true,
    message,
    type,
    duration
  });

  toastTimer = setTimeout(() => {
    hideToast();
    toastTimer = null;
  }, duration);
}

export function hideToast() {
  toastStore.update(t => ({ ...t, visible: false }));
}
