<script>
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';

  let originalText = '';
  let translatedText = '';
  let isLoading = false;
  let isStreaming = false;

  const window = getCurrentWindow();

  onMount(async () => {
    // Listen for translation started event
    await listen('translation-started', () => {
      isLoading = true;
      isStreaming = false;
      originalText = '';
      translatedText = '';
    });

    // Listen for preview data from Rust (non-streaming: full result)
    await listen('show-preview', (event) => {
      originalText = event.payload.original;
      translatedText = event.payload.translated;
      isLoading = false;
      isStreaming = false;
    });

    // Listen for streaming chunks
    await listen('stream-chunk', (event) => {
      if (event.payload.original && !originalText) {
        originalText = event.payload.original;
      }
      translatedText = event.payload.text;
      if (isLoading) {
        isLoading = false;
        isStreaming = true;
      }
    });

    // Listen for stream completion
    await listen('stream-done', () => {
      isStreaming = false;
    });

    // Don't close window on blur - let user explicitly choose action
  });

  async function handlePaste() {
    try {
      await invoke('execute_paste');
    } catch (error) {
      console.error('Paste failed:', error);
    }
  }

  async function handleRetry() {
    isLoading = true;
    try {
      await invoke('retry_translation');
    } catch (error) {
      console.error('Retry failed:', error);
      isLoading = false;
    }
  }

  async function handleCancel() {
    try {
      // Clear pending translation so nothing gets pasted accidentally
      await invoke('cancel_preview');
    } catch (error) {
      console.error('Cancel failed:', error);
    }
    await window.hide();
  }

  // Keyboard event handler
  function handleKeyPress(event) {
    if (isLoading) return;
    if (isStreaming && event.key !== 'Escape') return;

    if (event.key === 'Enter') {
      event.preventDefault();
      handlePaste();
    } else if (event.key === ' ') {
      event.preventDefault();
      handleRetry();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      handleCancel();
    }
  }
</script>

<svelte:window on:keydown={handleKeyPress} />

<div class="preview-container">
  <div class="preview-card">
    {#if isLoading}
      <div class="loading">
        <div class="spinner"></div>
        <p>Translation en cours...</p>
      </div>
    {:else}
      <div class="content">
        <div class="section">
          <div class="label">Original</div>
          <div class="text original">{originalText}</div>
        </div>

        <div class="arrow">→</div>

        <div class="section">
          <div class="label">
            Result
            {#if isStreaming}
              <span class="streaming-dot"></span>
            {/if}
          </div>
          <div class="text translated">{translatedText}{#if isStreaming}<span class="cursor">▌</span>{/if}</div>
        </div>
      </div>

      <div class="actions">
        <button class="action-btn paste" on:click={handlePaste} disabled={isStreaming}>
          <span class="key">ENTRÉE</span>
          Coller
        </button>
        <button class="action-btn retry" on:click={handleRetry} disabled={isStreaming}>
          <span class="key">ESPACE</span>
          Recommencer
        </button>
        <button class="action-btn cancel" on:click={handleCancel}>
          <span class="key">ÉCHAP</span>
          Annuler
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: rgb(20, 20, 30);
  }

  .preview-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: stretch;
    justify-content: center;
    padding: 16px;
    box-sizing: border-box;
  }

  .preview-card {
    background: rgba(20, 20, 30, 0.98);
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    padding: 20px;
    max-width: 620px;
    width: 100%;
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .loading {
    text-align: center;
    padding: 40px 20px;
  }

  .spinner {
    width: 40px;
    height: 40px;
    margin: 0 auto 16px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: #6366f1;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading p {
    color: rgba(255, 255, 255, 0.7);
    margin: 0;
    font-size: 14px;
  }

  .content {
    display: flex;
    gap: 16px;
    align-items: stretch;
    margin-bottom: 16px;
    flex: 1;
    min-height: 0;
  }

  .section {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: rgba(255, 255, 255, 0.5);
    margin-bottom: 8px;
  }

  .text {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 12px;
    color: white;
    font-size: 14px;
    line-height: 1.6;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .text.translated {
    background: rgba(99, 102, 241, 0.1);
    border: 1px solid rgba(99, 102, 241, 0.3);
  }

  .arrow {
    font-size: 24px;
    color: rgba(255, 255, 255, 0.3);
    flex-shrink: 0;
    align-self: center;
  }

  .actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .action-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 12px 16px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
    color: white;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
    transform: translateY(-1px);
  }

  .action-btn.paste {
    background: rgba(99, 102, 241, 0.2);
    border-color: rgba(99, 102, 241, 0.4);
  }

  .action-btn.paste:hover {
    background: rgba(99, 102, 241, 0.3);
    border-color: rgba(99, 102, 241, 0.5);
  }

  .action-btn.retry {
    background: rgba(245, 158, 11, 0.2);
    border-color: rgba(245, 158, 11, 0.4);
  }

  .action-btn.retry:hover {
    background: rgba(245, 158, 11, 0.3);
    border-color: rgba(245, 158, 11, 0.5);
  }

  .action-btn.cancel {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.4);
  }

  .action-btn.cancel:hover {
    background: rgba(239, 68, 68, 0.3);
    border-color: rgba(239, 68, 68, 0.5);
  }

  .key {
    display: inline-block;
    font-size: 10px;
    font-weight: 600;
    padding: 4px 8px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    letter-spacing: 0.5px;
  }

  /* Custom scrollbar for text areas */
  .text::-webkit-scrollbar {
    width: 6px;
  }

  .text::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 3px;
  }

  .text::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
  }

  .text::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .streaming-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    background: #6366f1;
    border-radius: 50%;
    margin-left: 4px;
    animation: pulse 1s ease-in-out infinite;
    vertical-align: middle;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .cursor {
    color: #6366f1;
    animation: blink 0.8s step-end infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }

  .action-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
    transform: none;
  }

  .action-btn:disabled:hover {
    transform: none;
    background: inherit;
  }
</style>
