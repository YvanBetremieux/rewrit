<script>
  import { toastStore } from '../stores/toast.js';
  import { CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-svelte';
  import { fly } from 'svelte/transition';

  let toast;
  toastStore.subscribe(value => {
    toast = value;
  });

  function getIcon(type) {
    switch (type) {
      case 'success':
        return CheckCircle;
      case 'error':
        return AlertCircle;
      case 'warning':
        return AlertTriangle;
      default:
        return Info;
    }
  }

  function getIconColor(type) {
    switch (type) {
      case 'success':
        return 'text-green-500';
      case 'error':
        return 'text-red-500';
      case 'warning':
        return 'text-yellow-500';
      default:
        return 'text-blue-500';
    }
  }
</script>

{#if toast.visible}
  <div
    class="fixed top-5 right-5 z-50 bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4 flex items-center gap-3 min-w-[300px] max-w-[400px]"
    transition:fly={{ y: -50, duration: 300 }}
  >
    <svelte:component
      this={getIcon(toast.type)}
      class="{getIconColor(toast.type)} flex-shrink-0"
      size={20}
    />
    <p class="text-gray-900 dark:text-gray-100 text-sm flex-1">
      {toast.message}
    </p>
  </div>
{/if}

<style>
  /* Additional styles if needed */
</style>
