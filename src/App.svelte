<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import Toast from './lib/components/Toast.svelte';
  import Settings from './lib/components/Settings.svelte';
  import QuotaExceededModal from './lib/components/QuotaExceededModal.svelte';
  import { quotaStore, updateQuota } from './lib/stores/quota.js';
  import { setDeviceId } from './lib/stores/settings.js';
  import { showToast } from './lib/stores/toast.js';

  let showQuotaModal = false;
  let isLoading = true;

  onMount(async () => {
    try {
      // Initialize device ID
      const deviceId = await invoke('get_device_id');
      setDeviceId(deviceId);

      // Fetch initial quota
      const quota = await invoke('fetch_quota');
      if (quota) {
        updateQuota(quota);
      }

      // Listen for quota updates
      await listen('quota-updated', (event) => {
        updateQuota(event.payload);
      });

      // Listen for quota exceeded events
      await listen('quota-exceeded', () => {
        showQuotaModal = true;
      });

      // Listen for translation started
      await listen('translation-started', () => {
        let currentQuota;
        quotaStore.subscribe(q => { currentQuota = q; })();
        showToast(`🔄 rewrit is working... ${currentQuota.used}/${currentQuota.limit} actions used`, 'info', 2000);
      });

      // Listen for translation completed
      await listen('translation-completed', (event) => {
        const { quota } = event.payload;
        updateQuota(quota);

        if (quota.used === quota.limit) {
          showToast(`✅ Translation complete\n⚠️ ${quota.used}/${quota.limit} actions used today\nResets in ${getTimeUntilReset(quota.resets_at)}`, 'warning', 4000);
        } else {
          showToast(`✅ Translation complete`, 'success', 2000);
        }
      });

      // Listen for errors
      await listen('translation-error', (event) => {
        const { error } = event.payload;
        showToast(error, 'error', 3000);
      });
    } catch (error) {
      console.error('Initialization error:', error);
      showToast('Failed to initialize app', 'error');
    } finally {
      isLoading = false;
    }
  });

  function getTimeUntilReset(resetsAt) {
    if (!resetsAt) return '';

    const now = new Date();
    const reset = new Date(resetsAt);
    const diff = reset - now;

    if (diff <= 0) return '0h 0min';

    const hours = Math.floor(diff / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

    return `${hours}h ${minutes}min`;
  }
</script>

<main class="min-h-screen">
  {#if isLoading}
    <div class="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
      <div class="text-center">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
        <p class="text-gray-600 dark:text-gray-400">Loading rewrit...</p>
      </div>
    </div>
  {:else}
    <Settings />
  {/if}

  <Toast />
  <QuotaExceededModal bind:visible={showQuotaModal} />
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
  }
</style>
