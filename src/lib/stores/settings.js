import { writable } from 'svelte/store';

export const settingsStore = writable({
  enabled: true,
  showNotifications: true,
  deviceId: null,
  shortcuts: {
    translate: 'CommandOrControl+Shift+T'
  }
});

export function updateSettings(newSettings) {
  settingsStore.update(s => ({ ...s, ...newSettings }));
}

export function setDeviceId(deviceId) {
  settingsStore.update(s => ({ ...s, deviceId }));
}
