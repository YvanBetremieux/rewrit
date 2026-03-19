<script>
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  let status = 'working'; // 'working' | 'streaming' | 'done' | 'error'
  let elapsed = '0.0';
  let percent = 0;
  let errorMsg = '';
  let pollTimer = null;

  onMount(() => {
    // Poll progress state from Rust every 100ms
    pollTimer = setInterval(async () => {
      try {
        const ps = await invoke('get_progress_state');

        if (!ps.active) return;

        // Calculate elapsed from start_ms
        const now = Date.now();
        elapsed = ((now - ps.start_ms) / 1000).toFixed(1);

        if (ps.error) {
          status = 'error';
          errorMsg = ps.error;
        } else if (ps.done) {
          status = 'done';
          percent = 100;
        } else if (ps.output_chars > 0) {
          status = 'streaming';
          const estimated = ps.input_chars * 1.2;
          percent = Math.min(Math.round((ps.output_chars / estimated) * 100), 99);
        } else {
          // Waiting for first token — simulate slow progress (0-5%)
          status = 'working';
          const waitSec = (now - ps.start_ms) / 1000;
          percent = Math.min(Math.round(waitSec * 2), 5);
        }
      } catch (e) {
        // ignore poll errors
      }
    }, 100);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });
</script>

<div class="progress-widget">
  <div class="top-row">
    {#if status === 'error'}
      <span class="icon error-icon">!</span>
      <span class="label error">{errorMsg.slice(0, 35)}</span>
    {:else if status === 'done'}
      <span class="icon done-icon">✓</span>
      <span class="label">Done</span>
      <span class="time">{elapsed}s</span>
    {:else}
      <div class="spinner-small"></div>
      <span class="label">Translating...</span>
      <span class="time">{elapsed}s</span>
    {/if}
  </div>

  <div class="bar-container">
    <div class="bar-fill" style="width: {percent}%"></div>
  </div>

  {#if status === 'streaming' || status === 'done'}
    <div class="percent">{percent}%</div>
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: rgb(20, 20, 30);
    overflow: hidden;
  }

  .progress-widget {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 8px 14px;
    background: rgba(20, 20, 30, 0.95);
    border: 1px solid rgba(99, 102, 241, 0.4);
    border-radius: 12px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
  }

  .top-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .spinner-small {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.15);
    border-top-color: #6366f1;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .icon {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .done-icon {
    background: #22c55e;
    color: white;
  }

  .error-icon {
    background: #ef4444;
    color: white;
  }

  .label {
    color: rgba(255, 255, 255, 0.85);
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
    flex: 1;
  }

  .label.error {
    color: #fca5a5;
  }

  .time {
    color: rgba(255, 255, 255, 0.4);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .bar-container {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: linear-gradient(90deg, #6366f1, #8b5cf6);
    border-radius: 2px;
    transition: width 0.1s ease-out;
    min-width: 0;
  }

  .percent {
    color: rgba(255, 255, 255, 0.5);
    font-size: 10px;
    font-weight: 600;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
</style>
