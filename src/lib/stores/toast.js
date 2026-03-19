import { writable } from 'svelte/store';

export const toastStore = writable({
  visible: false,
  message: '',
  type: 'info', // 'info', 'success', 'error', 'warning'
  duration: 3000
});

export function showToast(message, type = 'info', duration = 3000) {
  toastStore.set({
    visible: true,
    message,
    type,
    duration
  });

  setTimeout(() => {
    hideToast();
  }, duration);
}

export function hideToast() {
  toastStore.update(t => ({ ...t, visible: false }));
}
