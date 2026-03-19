<script>
  import { invoke } from '@tauri-apps/api/core';

  const languages = [
    { key: '1', code: 'fr', label: 'Français', flag: '🇫🇷' },
    { key: '2', code: 'en', label: 'English', flag: '🇬🇧' },
    { key: '3', code: 'es', label: 'Español', flag: '🇪🇸' },
  ];

  const tones = [
    { key: 'a', code: 'professional', label: 'Pro', icon: '💼' },
    { key: 'b', code: 'casual', label: 'Casual', icon: '😊' },
    { key: 'c', code: 'concise', label: 'Concis', icon: '✂️' },
  ];

  let selectedLanguage = languages[0];
  let selectedTone = tones[0];

  function selectLanguage(lang) {
    selectedLanguage = lang;
  }

  function selectTone(tone) {
    selectedTone = tone;
  }

  async function execute() {
    if (selectedLanguage && selectedTone) {
      try {
        await invoke('execute_quick_select', {
          language: selectedLanguage.code,
          tone: selectedTone.code,
        });
      } catch (error) {
        console.error('Quick select failed:', error);
      }
    }
  }

  async function handleCancel() {
    try {
      await invoke('cancel_quick_select');
    } catch (error) {
      console.error('Cancel failed:', error);
    }
  }

  function handleKeyPress(event) {
    const key = event.key.toLowerCase();

    if (key === 'escape') {
      event.preventDefault();
      handleCancel();
      return;
    }

    if (event.key === 'Enter') {
      event.preventDefault();
      execute();
      return;
    }

    // Language keys
    const lang = languages.find(l => l.key === key);
    if (lang) {
      event.preventDefault();
      selectLanguage(lang);
      return;
    }

    // Tone keys
    const tone = tones.find(t => t.key === key);
    if (tone) {
      event.preventDefault();
      selectTone(tone);
    }
  }
</script>

<svelte:window on:keydown={handleKeyPress} />

<div class="qs-container">
  <div class="qs-card">
    <div class="qs-header">Quick Select</div>

    <div class="qs-columns">
      <!-- Language column -->
      <div class="qs-column">
        <div class="qs-column-label">Langue</div>
        {#each languages as lang}
          <button
            class="qs-option"
            class:selected={selectedLanguage === lang}
            on:click={() => selectLanguage(lang)}
          >
            <span class="qs-key">{lang.key}</span>
            <span class="qs-icon">{lang.flag}</span>
            <span class="qs-label">{lang.label}</span>
          </button>
        {/each}
      </div>

      <!-- Tone column -->
      <div class="qs-column">
        <div class="qs-column-label">Ton</div>
        {#each tones as tone}
          <button
            class="qs-option"
            class:selected={selectedTone === tone}
            on:click={() => selectTone(tone)}
          >
            <span class="qs-key">{tone.key.toUpperCase()}</span>
            <span class="qs-icon">{tone.icon}</span>
            <span class="qs-label">{tone.label}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="qs-footer">
      <span class="qs-hint">Entrée pour valider · Échap pour annuler</span>
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .qs-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px;
    box-sizing: border-box;
  }

  .qs-card {
    background: rgba(20, 20, 30, 0.98);
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    padding: 20px;
    width: 100%;
    max-width: 320px;
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .qs-header {
    text-align: center;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: rgba(255, 255, 255, 0.5);
    margin-bottom: 16px;
  }

  .qs-columns {
    display: flex;
    gap: 12px;
  }

  .qs-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .qs-column-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: rgba(255, 255, 255, 0.4);
    margin-bottom: 4px;
    padding-left: 4px;
  }

  .qs-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.05);
    color: white;
    cursor: pointer;
    transition: all 0.15s ease;
    font-size: 14px;
  }

  .qs-option:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
    transform: translateY(-1px);
  }

  .qs-option.selected {
    background: rgba(99, 102, 241, 0.25);
    border-color: rgba(99, 102, 241, 0.5);
    box-shadow: 0 0 12px rgba(99, 102, 241, 0.2);
  }

  .qs-key {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    font-size: 11px;
    font-weight: 700;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 5px;
    flex-shrink: 0;
    font-family: monospace;
  }

  .qs-icon {
    font-size: 16px;
    flex-shrink: 0;
  }

  .qs-label {
    font-size: 13px;
    font-weight: 500;
  }

  .qs-footer {
    margin-top: 14px;
    text-align: center;
  }

  .qs-hint {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.3);
  }
</style>
