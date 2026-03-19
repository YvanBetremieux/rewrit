import { writable } from 'svelte/store';

export const quotaStore = writable({
  used: 0,
  limit: 5,
  remaining: 5,
  resetsAt: null,
  lastSync: null
});

export function updateQuota(quota) {
  quotaStore.set({
    used: quota.used,
    limit: quota.limit,
    remaining: quota.remaining,
    resetsAt: quota.resets_at,
    lastSync: new Date().toISOString()
  });
}

export function incrementQuota() {
  quotaStore.update(q => ({
    ...q,
    used: q.used + 1,
    remaining: Math.max(0, q.remaining - 1)
  }));
}
