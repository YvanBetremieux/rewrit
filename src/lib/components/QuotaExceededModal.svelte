<script>
  import { quotaStore } from '../stores/quota.js';
  import { AlertTriangle, Clock, Sparkles, X } from 'lucide-svelte';
  import { fade, scale } from 'svelte/transition';

  export let visible = false;

  let quota;
  quotaStore.subscribe(value => {
    quota = value;
  });

  function getTimeUntilReset() {
    if (!quota.resetsAt) return '';

    const now = new Date();
    const reset = new Date(quota.resetsAt);
    const diff = reset - now;

    if (diff <= 0) return '0h 0min';

    const hours = Math.floor(diff / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

    return `${hours}h ${minutes}min`;
  }

  function close() {
    visible = false;
  }

  function handleKeydown(e) {
    if (e.key === 'Escape') {
      close();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible}
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
    transition:fade={{ duration: 200 }}
    on:click={close}
  >
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl max-w-md w-full p-6 relative"
      transition:scale={{ duration: 200 }}
      on:click|stopPropagation
    >
      <!-- Close Button -->
      <button
        on:click={close}
        class="absolute top-4 right-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
      >
        <X size={20} />
      </button>

      <!-- Warning Icon -->
      <div class="flex justify-center mb-4">
        <div class="bg-yellow-100 dark:bg-yellow-900 rounded-full p-3">
          <AlertTriangle class="text-yellow-600 dark:text-yellow-400" size={32} />
        </div>
      </div>

      <!-- Title -->
      <h2 class="text-2xl font-bold text-center text-gray-900 dark:text-white mb-2">
        Daily Limit Reached
      </h2>

      <!-- Message -->
      <p class="text-center text-gray-600 dark:text-gray-300 mb-4">
        You've used your {quota.limit} free actions today. Your quota resets in:
      </p>

      <!-- Reset Timer -->
      <div class="bg-gray-100 dark:bg-gray-700 rounded-lg p-4 mb-6">
        <div class="flex items-center justify-center gap-2 text-2xl font-bold text-gray-900 dark:text-white">
          <Clock size={24} class="text-blue-600" />
          {getTimeUntilReset()}
        </div>
      </div>

      <!-- Upgrade CTA -->
      <div class="bg-gradient-to-r from-purple-50 to-blue-50 dark:from-purple-900 dark:to-blue-900 rounded-lg p-4 mb-4">
        <p class="text-center text-gray-700 dark:text-gray-200 mb-3 font-medium">
          Want unlimited actions?
        </p>
        <button
          class="w-full bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg px-4 py-3 flex items-center justify-center gap-2 hover:from-purple-700 hover:to-blue-700 transition-colors font-semibold"
        >
          <Sparkles size={18} />
          Upgrade to Pro (coming soon)
        </button>
      </div>

      <!-- Close Button -->
      <button
        on:click={close}
        class="w-full bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg px-4 py-2 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
      >
        Close
      </button>
    </div>
  </div>
{/if}
