<script>
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { quotaStore } from '../stores/quota.js';
  import { settingsStore } from '../stores/settings.js';
  import { showToast } from '../stores/toast.js';
  import { getTimeUntilReset } from '../utils/time.js';
  import { Copy, Book, HelpCircle, Sparkles, RotateCcw, Key, Eye, EyeOff, Download, FolderOpen, Stethoscope } from 'lucide-svelte';

  let quota = { used: 0, limit: 100, remaining: 100, resetsAt: null };
  let settings = { deviceId: '', enabled: true, showNotifications: true };
  let isResetting = false;
  let selectedProvider = 'groq';
  let apiKeyInput = '';
  let savedApiKey = '';
  let showApiKey = false;
  let isSavingKey = false;

  const providers = [
    { id: 'groq', name: 'Groq', description: 'GPT-OSS 120B — Ultra-fast', keyPrefix: 'gsk_', keyUrl: 'https://console.groq.com/keys', local: false },
    { id: 'mistral', name: 'Mistral', description: 'Mistral Small — EU/RGPD', keyPrefix: '', keyUrl: 'https://console.mistral.ai/api-keys', local: false },
    { id: 'gemini', name: 'Gemini', description: 'Gemini 2.5 Flash — Google', keyPrefix: '', keyUrl: 'https://aistudio.google.com/apikey', local: false },
    { id: 'ollama', name: 'Ollama', description: 'LLaMA 3.2 3B — Local/Offline', keyPrefix: '', keyUrl: '', local: true },
  ];
  let ollamaStatus = { installed: false, ollama_running: false };
  let isOllamaPulling = false;
  let isOllamaDeleting = false;
  let showPreview = true;
  let streamResponse = false;
  let quickSelectEnabled = true;
  let quickSelectShortcut = '';
  let quickSelectShortcutInput = '';
  let isSavingQsShortcut = false;
  let shortcuts = [];
  let editingShortcut = null;
  let showShortcutHelp = false;
  let updateSettings = { current_version: '', auto_check: true, auto_install: true, last_check: null };
  // idle | checking | up-to-date | available | installing | installed | error
  let updateState = 'idle';
  let availableVersion = null;
  let updateError = '';
  let unlistenUpdate = null;
  let diagnostics = null;
  let connectionTest = null;
  let isTestingConnection = false;

  // Check if we're in development mode
  const isDevelopment = import.meta.env.DEV;

  quotaStore.subscribe(value => {
    quota = value;
  });

  settingsStore.subscribe(value => {
    settings = value;
  });

  // Load provider and API key
  async function loadProviderSettings() {
    try {
      selectedProvider = await invoke('get_llm_provider');
      const p = providers.find(p => p.id === selectedProvider);
      if (p?.local) {
        await checkOllamaStatus();
      } else {
        await loadApiKeyForProvider(selectedProvider);
      }
    } catch (error) {
      console.error('Failed to load provider settings:', error);
    }
  }

  async function loadApiKeyForProvider(provider) {
    try {
      savedApiKey = await invoke('get_api_key', { provider });
      apiKeyInput = savedApiKey;
    } catch (error) {
      console.error('Failed to load API key:', error);
    }
  }

  async function changeProvider(newProvider) {
    selectedProvider = newProvider;
    showApiKey = false;
    const p = providers.find(p => p.id === newProvider);
    if (p?.local) {
      await checkOllamaStatus();
    } else {
      await loadApiKeyForProvider(newProvider);
    }
    try {
      await invoke('set_llm_provider', { provider: newProvider });
      showToast(`Switched to ${p?.name || newProvider}`, 'success', 2000);
    } catch (error) {
      showToast('Failed to switch provider: ' + error, 'error', 3000);
    }
  }

  async function saveApiKey() {
    if (isSavingKey) return;
    isSavingKey = true;
    try {
      await invoke('set_api_key', { provider: selectedProvider, key: apiKeyInput.trim() });
      savedApiKey = apiKeyInput.trim();
      showToast(savedApiKey ? 'API key saved' : 'API key removed', 'success', 2000);
    } catch (error) {
      showToast('Failed to save API key: ' + error, 'error', 3000);
    } finally {
      isSavingKey = false;
    }
  }

  // Ollama model management
  async function checkOllamaStatus() {
    try {
      ollamaStatus = await invoke('ollama_status');
    } catch (error) {
      ollamaStatus = { installed: false, ollama_running: false };
    }
  }

  async function pullOllamaModel() {
    if (isOllamaPulling) return;
    isOllamaPulling = true;
    showToast('Downloading LLaMA 3.2 3B (~2 GB)...', 'success', 5000);
    try {
      await invoke('ollama_pull');
      ollamaStatus.installed = true;
      showToast('Model downloaded!', 'success', 3000);
    } catch (error) {
      showToast('Download failed: ' + error, 'error', 5000);
    } finally {
      isOllamaPulling = false;
    }
  }

  async function deleteOllamaModel() {
    if (isOllamaDeleting) return;
    isOllamaDeleting = true;
    try {
      await invoke('ollama_delete');
      ollamaStatus.installed = false;
      showToast('Model deleted (~2 GB freed)', 'success', 3000);
    } catch (error) {
      showToast('Delete failed: ' + error, 'error', 3000);
    } finally {
      isOllamaDeleting = false;
    }
  }

  // Load show preview setting
  async function loadShowPreview() {
    try {
      showPreview = await invoke('get_show_preview');
    } catch (error) {
      console.error('Failed to load show preview setting:', error);
    }
  }

  // Load stream response setting
  async function loadStreamResponse() {
    try {
      streamResponse = await invoke('get_stream_response');
    } catch (error) {
      console.error('Failed to load stream response setting:', error);
    }
  }

  // Toggle stream response
  async function toggleStreamResponse() {
    streamResponse = !streamResponse;
    try {
      await invoke('set_stream_response', { enabled: streamResponse });
      showToast(streamResponse ? 'Streaming enabled' : 'Streaming disabled', 'success', 2000);
    } catch (error) {
      console.error('Failed to update stream response setting:', error);
      streamResponse = !streamResponse;
    }
  }

  // Load quick select shortcut
  async function loadQuickSelectShortcut() {
    try {
      quickSelectShortcut = await invoke('get_quick_select_shortcut');
      quickSelectShortcutInput = quickSelectShortcut;
    } catch (error) {
      console.error('Failed to load quick select shortcut:', error);
    }
  }

  // Save quick select shortcut
  async function saveQuickSelectShortcut() {
    if (isSavingQsShortcut || !quickSelectShortcutInput.trim()) return;
    isSavingQsShortcut = true;
    try {
      await invoke('set_quick_select_shortcut', { shortcut: quickSelectShortcutInput.trim() });
      quickSelectShortcut = quickSelectShortcutInput.trim();
      showToast('Quick Select shortcut updated', 'success', 2000);
    } catch (error) {
      showToast('Invalid shortcut: ' + error, 'error', 3000);
    } finally {
      isSavingQsShortcut = false;
    }
  }

  // Load quick select setting
  async function loadQuickSelectEnabled() {
    try {
      quickSelectEnabled = await invoke('get_quick_select_enabled');
    } catch (error) {
      console.error('Failed to load quick select setting:', error);
    }
  }

  // Toggle quick select setting
  async function toggleQuickSelect() {
    quickSelectEnabled = !quickSelectEnabled;
    try {
      await invoke('set_quick_select_enabled', { enabled: quickSelectEnabled });
      showToast(quickSelectEnabled ? 'Quick Select enabled' : 'Quick Select disabled', 'success', 2000);
    } catch (error) {
      console.error('Failed to update quick select setting:', error);
      quickSelectEnabled = !quickSelectEnabled; // Revert on error
    }
  }

  // Toggle show preview setting
  async function toggleShowPreview() {
    showPreview = !showPreview;
    try {
      await invoke('set_show_preview', { show: showPreview });
      showToast(showPreview ? 'Preview enabled' : 'Direct paste enabled', 'success', 2000);
    } catch (error) {
      console.error('Failed to update show preview setting:', error);
      showPreview = !showPreview; // Revert on error
    }
  }

  async function loadShortcuts() {
    try {
      const result = await invoke('get_shortcuts');
      shortcuts = Array.isArray(result) ? result : [];
      console.log('Loaded shortcuts:', shortcuts);
    } catch (error) {
      console.error('Failed to load shortcuts:', error);
      shortcuts = [];
    }
  }

  // Load data on mount
  onMount(async () => {
    try {
      await Promise.all([
        loadProviderSettings(),
        loadShowPreview(),
        loadStreamResponse(),
        loadQuickSelectShortcut(),
        loadQuickSelectEnabled(),
        loadShortcuts(),
        loadUpdateSettings(),
        loadDiagnostics()
      ]);
      unlistenUpdate = await listen('update-status', (event) => {
        updateState = event.payload;
        loadUpdateSettings();
      });
    } catch (error) {
      console.error('Error loading settings:', error);
    }
  });

  onDestroy(() => unlistenUpdate?.());

  // --- Updates ---

  async function loadUpdateSettings() {
    try {
      updateSettings = await invoke('get_update_settings');
    } catch (error) {
      console.error('Failed to load update settings:', error);
    }
  }

  async function toggleUpdateAutoCheck() {
    const enabled = !updateSettings.auto_check;
    try {
      await invoke('set_update_auto_check', { enabled });
      updateSettings = { ...updateSettings, auto_check: enabled };
    } catch (error) {
      showToast('Failed to save setting: ' + error, 'error', 3000);
    }
  }

  async function toggleUpdateAutoInstall() {
    const enabled = !updateSettings.auto_install;
    try {
      await invoke('set_update_auto_install', { enabled });
      updateSettings = { ...updateSettings, auto_install: enabled };
    } catch (error) {
      showToast('Failed to save setting: ' + error, 'error', 3000);
    }
  }

  async function checkForUpdate() {
    updateState = 'checking';
    updateError = '';
    try {
      const result = await invoke('check_for_update');
      availableVersion = result.version;
      updateState = result.available ? 'available' : 'up-to-date';
    } catch (error) {
      updateError = String(error);
      updateState = 'error';
    }
    loadUpdateSettings();
  }

  async function installUpdate() {
    updateState = 'installing';
    updateError = '';
    try {
      // Restarts the app on success
      await invoke('install_update');
    } catch (error) {
      updateError = String(error);
      updateState = 'error';
    }
  }

  function formatLastCheck(timestamp) {
    if (!timestamp) return 'Never';
    return new Date(timestamp).toLocaleString();
  }

  // --- Diagnostics ---

  async function loadDiagnostics() {
    try {
      diagnostics = await invoke('get_diagnostics');
    } catch (error) {
      console.error('Failed to load diagnostics:', error);
    }
  }

  async function testConnection() {
    isTestingConnection = true;
    connectionTest = null;
    try {
      connectionTest = await invoke('test_llm_connection');
    } catch (error) {
      connectionTest = { ok: false, duration_ms: 0, error: String(error) };
    } finally {
      isTestingConnection = false;
      loadDiagnostics();
    }
  }

  async function openLogs() {
    try {
      await invoke('open_log_dir');
    } catch (error) {
      showToast('Failed to open logs folder: ' + error, 'error', 3000);
    }
  }

  function copyDiagnostics() {
    const lines = [
      ...Object.entries(diagnostics || {}).map(([k, v]) => `${k}: ${v ?? '-'}`),
      `last_update_check: ${updateSettings.last_check ?? 'never'}`,
    ];
    if (connectionTest) {
      lines.push(`connection_test: ${connectionTest.ok ? 'ok' : 'failed'} (${connectionTest.duration_ms} ms)`);
      if (connectionTest.error) lines.push(`connection_error: ${connectionTest.error}`);
    }
    navigator.clipboard.writeText(lines.join('\n'));
    showToast('Diagnostics copied to clipboard', 'success', 2000);
  }

  const PROMPT_MAX_LENGTH = 500;

  async function addShortcut() {
    editingShortcut = { shortcut: '', prompt: '' };
  }

  async function removeShortcut(id) {
    try {
      await invoke('remove_shortcut', { id });
      await loadShortcuts();
      showToast('Shortcut removed', 'success', 2000);
    } catch (error) {
      showToast('Failed to remove shortcut: ' + error, 'error', 3000);
    }
  }

  async function editShortcut(shortcut) {
    editingShortcut = { ...shortcut };
  }

  async function saveShortcut() {
    if (!editingShortcut) return;

    if (!editingShortcut.prompt || !editingShortcut.prompt.trim()) {
      showToast('Please enter a prompt', 'error', 2000);
      return;
    }
    if (!editingShortcut.shortcut || !editingShortcut.shortcut.trim()) {
      showToast('Please enter a shortcut', 'error', 2000);
      return;
    }

    try {
      if (editingShortcut.id) {
        await invoke('update_shortcut', {
          id: editingShortcut.id,
          shortcut: editingShortcut.shortcut || null,
          prompt: editingShortcut.prompt || null
        });
        showToast('Shortcut updated', 'success', 2000);
      } else {
        await invoke('add_shortcut', {
          shortcut: editingShortcut.shortcut,
          prompt: editingShortcut.prompt
        });
        showToast('Shortcut added', 'success', 2000);
      }
      editingShortcut = null;
      await loadShortcuts();
    } catch (error) {
      showToast('Failed to save shortcut: ' + error, 'error', 3000);
    }
  }

  function copyDeviceId() {
    if (settings.deviceId) {
      navigator.clipboard.writeText(settings.deviceId);
      showToast('Device ID copied to clipboard', 'success', 2000);
    }
  }

  async function resetQuota() {
    if (isResetting) return;

    isResetting = true;
    try {
      const newQuota = await invoke('reset_quota');
      quotaStore.set({
        used: newQuota.used,
        limit: newQuota.limit,
        remaining: newQuota.remaining,
        resetsAt: newQuota.resets_at
      });
      showToast('Quota reset to 0 (exhausted)', 'success', 3000);
    } catch (error) {
      showToast('Failed to reset quota: ' + error, 'error', 3000);
    } finally {
      isResetting = false;
    }
  }

  function getTimeUntilResetLocal() {
    return getTimeUntilReset(quota?.resetsAt);
  }

  function getQuotaPercentage() {
    if (!quota || !quota.limit) return 0;
    return (quota.used / quota.limit) * 100;
  }

  function toggleEnabled() {
    settingsStore.update(s => ({ ...s, enabled: !s.enabled }));
  }

  function toggleNotifications() {
    settingsStore.update(s => ({ ...s, showNotifications: !s.showNotifications }));
  }
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900 p-5">
  <!-- Header -->
  <div class="mb-5">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">rewrit</h1>
    <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Settings</p>
  </div>

  <!-- Two-column layout -->
  <div class="grid grid-cols-2 gap-5">
    <!-- LEFT COLUMN -->
    <div class="space-y-5">
      <!-- Usage Section -->
      <section class="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
          <span>📊</span> Usage
        </h2>

        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
          <div class="flex items-center justify-between mb-2">
            <p class="text-sm text-gray-600 dark:text-gray-300">
              {quota?.used || 0} / {quota?.limit || 100} actions
            </p>
            <p class="text-sm font-medium text-gray-700 dark:text-gray-200">
              {Math.round(getQuotaPercentage())}%
            </p>
          </div>

          <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2.5">
            <div
              class="h-2.5 rounded-full transition-all duration-300"
              class:bg-blue-600={getQuotaPercentage() < 80}
              class:bg-yellow-500={getQuotaPercentage() >= 80 && getQuotaPercentage() < 100}
              class:bg-red-500={getQuotaPercentage() >= 100}
              style="width: {getQuotaPercentage()}%"
            ></div>
          </div>
        </div>

        {#if quota?.resetsAt}
          <p class="mt-3 text-xs text-gray-500 dark:text-gray-400">
            Resets in {getTimeUntilResetLocal()}
          </p>
        {/if}

        <button class="mt-4 w-full bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg px-4 py-2 text-sm flex items-center justify-center gap-2 hover:from-purple-700 hover:to-blue-700 transition-colors">
          <Sparkles size={16} />
          Upgrade to Pro (coming soon)
        </button>

        {#if isDevelopment}
          <button
            on:click={resetQuota}
            disabled={isResetting}
            class="mt-2 w-full bg-red-600/80 text-white rounded-lg px-4 py-1.5 text-xs flex items-center justify-center gap-2 hover:bg-red-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <RotateCcw size={14} class={isResetting ? 'animate-spin' : ''} />
            {isResetting ? 'Resetting...' : 'Reset Quota (Debug)'}
          </button>
        {/if}
      </section>

      <!-- Preferences Section -->
      <section class="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
          <span>🔧</span> Preferences
        </h2>

        <div class="space-y-3">
          <label class="flex items-center justify-between cursor-pointer py-1">
            <span class="text-sm text-gray-700 dark:text-gray-300">Enable rewrit</span>
            <input
              type="checkbox"
              checked={settings.enabled}
              on:change={toggleEnabled}
              class="w-5 h-5 text-blue-600 rounded focus:ring-blue-500"
            />
          </label>

          <label class="flex items-center justify-between cursor-pointer py-1">
            <span class="text-sm text-gray-700 dark:text-gray-300">Show notifications</span>
            <input
              type="checkbox"
              checked={settings.showNotifications}
              on:change={toggleNotifications}
              class="w-5 h-5 text-blue-600 rounded focus:ring-blue-500"
            />
          </label>

          <label class="flex items-center justify-between cursor-pointer py-1">
            <span class="text-sm text-gray-700 dark:text-gray-300">Show preview before paste</span>
            <input
              type="checkbox"
              checked={showPreview}
              on:change={toggleShowPreview}
              class="w-5 h-5 text-blue-600 rounded focus:ring-blue-500"
            />
          </label>

          <label class="flex items-center justify-between cursor-pointer py-1">
            <div>
              <span class="text-sm text-gray-700 dark:text-gray-300">Stream response</span>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{showPreview ? 'Show text as it\'s generated' : 'Show progress indicator'}</p>
            </div>
            <input
              type="checkbox"
              checked={streamResponse}
              on:change={toggleStreamResponse}
              class="w-5 h-5 text-blue-600 rounded focus:ring-blue-500"
            />
          </label>

          <label class="flex items-center justify-between cursor-pointer py-1">
            <div>
              <span class="text-sm text-gray-700 dark:text-gray-300">Quick Select</span>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Choisir langue et ton au clavier</p>
            </div>
            <input
              type="checkbox"
              checked={quickSelectEnabled}
              on:change={toggleQuickSelect}
              class="w-5 h-5 text-blue-600 rounded focus:ring-blue-500"
            />
          </label>

          {#if quickSelectEnabled}
            <div class="flex gap-2 items-center pl-1">
              <span class="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">Shortcut:</span>
              <input
                type="text"
                bind:value={quickSelectShortcutInput}
                placeholder="CommandOrControl+Shift+Space"
                class="flex-1 px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono"
              />
              <button
                on:click={saveQuickSelectShortcut}
                disabled={isSavingQsShortcut || quickSelectShortcutInput.trim() === quickSelectShortcut}
                class="px-2 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Save
              </button>
            </div>
          {/if}
        </div>
      </section>

      <!-- AI Provider -->
      <section class="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-3 flex items-center gap-2">
          <Key size={16} /> AI Provider
        </h2>

        <!-- Provider selector -->
        <div class="flex gap-2 mb-4">
          {#each providers as provider}
            <button
              on:click={() => changeProvider(provider.id)}
              class="flex-1 px-3 py-2 text-sm rounded-lg border-2 transition-colors text-left {selectedProvider === provider.id ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-gray-200 dark:border-gray-600'}"
            >
              <span class="font-medium text-gray-900 dark:text-white">{provider.name}</span>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{provider.description}</p>
            </button>
          {/each}
        </div>

        <!-- Cloud: API Key input -->
        {#if !providers.find(p => p.id === selectedProvider)?.local}
          {#each providers as provider}
            {#if provider.id === selectedProvider && !provider.local}
              <p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
                Enter your <a href={provider.keyUrl} target="_blank" class="text-blue-500 hover:underline">{provider.name} API key</a>
              </p>
            {/if}
          {/each}
          <div class="flex gap-2">
            <div class="relative flex-1">
              {#if showApiKey}
                <input
                  type="text"
                  bind:value={apiKeyInput}
                  placeholder={providers.find(p => p.id === selectedProvider)?.keyPrefix + '...'}
                  class="w-full px-3 py-2 pr-9 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono"
                />
              {:else}
                <input
                  type="password"
                  bind:value={apiKeyInput}
                  placeholder={providers.find(p => p.id === selectedProvider)?.keyPrefix + '...'}
                  class="w-full px-3 py-2 pr-9 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono"
                />
              {/if}
              <button
                on:click={() => showApiKey = !showApiKey}
                class="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                title={showApiKey ? 'Hide' : 'Show'}
              >
                {#if showApiKey}
                  <EyeOff size={14} />
                {:else}
                  <Eye size={14} />
                {/if}
              </button>
            </div>
            <button
              on:click={saveApiKey}
              disabled={isSavingKey || apiKeyInput.trim() === savedApiKey}
              class="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSavingKey ? 'Saving...' : 'Save'}
            </button>
          </div>
          {#if savedApiKey}
            <p class="mt-2 text-xs text-green-600 dark:text-green-400">Key configured</p>
          {:else}
            <p class="mt-2 text-xs text-yellow-600 dark:text-yellow-400">No key set — enter your API key to start</p>
          {/if}

        <!-- Local: Ollama model management -->
        {:else}
          {#if !ollamaStatus.ollama_running}
            <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3">
              <p class="text-sm text-yellow-800 dark:text-yellow-200">Ollama is not running</p>
              <p class="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                Install from <a href="https://ollama.com" target="_blank" class="underline">ollama.com</a> then run <code class="bg-yellow-100 dark:bg-yellow-800 px-1 rounded">ollama serve</code>
              </p>
            </div>
          {:else if !ollamaStatus.installed}
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
              <p class="text-sm text-gray-700 dark:text-gray-300">LLaMA 3.2 3B (~2 GB)</p>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Runs locally, no internet needed. Slower than cloud APIs.</p>
              <button
                on:click={pullOllamaModel}
                disabled={isOllamaPulling}
                class="mt-3 w-full px-4 py-2 text-sm bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isOllamaPulling ? 'Downloading...' : 'Download model (~2 GB)'}
              </button>
            </div>
          {:else}
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm text-green-600 dark:text-green-400">LLaMA 3.2 3B installed</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Ready to use offline</p>
                </div>
                <button
                  on:click={deleteOllamaModel}
                  disabled={isOllamaDeleting}
                  class="px-3 py-1.5 text-xs bg-red-600 text-white rounded hover:bg-red-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isOllamaDeleting ? 'Deleting...' : 'Delete (~2 GB)'}
                </button>
              </div>
            </div>
          {/if}
        {/if}
      </section>

      <!-- Device ID -->
      <section class="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-3 flex items-center gap-2">
          <span>🆔</span> Device ID
        </h2>
        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3 flex items-center justify-between gap-2">
          <code class="text-xs text-gray-600 dark:text-gray-300 font-mono break-all flex-1">
            {settings?.deviceId || 'Loading...'}
          </code>
          <button
            on:click={copyDeviceId}
            class="p-1.5 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors flex-shrink-0"
            title="Copy to clipboard"
          >
            <Copy size={14} class="text-gray-600 dark:text-gray-300" />
          </button>
        </div>
      </section>
    </div>

    <!-- RIGHT COLUMN -->
    <div class="space-y-5">
      <!-- Shortcuts Section -->
      <section class="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
          <span>⌨️</span> Keyboard Shortcuts
        </h2>

        <div class="space-y-2">
          {#if shortcuts.length === 0}
            <p class="text-sm text-gray-500 dark:text-gray-400 text-center py-4">
              No shortcuts configured.
            </p>
          {:else}
            {#each shortcuts as shortcut (shortcut.id)}
              {#if shortcut && shortcut.prompt}
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
                  <div class="flex items-start justify-between gap-2">
                    <div class="flex-1 min-w-0">
                      <kbd class="inline-block px-2 py-0.5 text-xs font-semibold text-gray-800 bg-gray-100 border border-gray-200 rounded dark:bg-gray-600 dark:text-gray-100 dark:border-gray-500">
                        {shortcut.shortcut ? shortcut.shortcut.replace('CommandOrControl', '⌘/Ctrl') : 'N/A'}
                      </kbd>
                    </div>
                    <div class="flex gap-1.5 flex-shrink-0">
                      <button
                        on:click={() => editShortcut(shortcut)}
                        class="px-2.5 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                      >
                        Edit
                      </button>
                      <button
                        on:click={() => removeShortcut(shortcut.id)}
                        class="px-2.5 py-1 text-xs bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                      >
                        Remove
                      </button>
                    </div>
                  </div>
                  <p class="mt-2 text-xs text-gray-600 dark:text-gray-300 line-clamp-2">
                    {shortcut.prompt}
                  </p>
                </div>
              {/if}
            {/each}
          {/if}

          <button
            on:click={addShortcut}
            class="w-full mt-3 px-4 py-2 bg-green-600 text-white text-sm rounded-lg hover:bg-green-700 transition-colors flex items-center justify-center gap-2"
          >
            <span>+</span> Add Shortcut
          </button>
        </div>

        {#if editingShortcut}
          <div class="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
            <h3 class="text-sm font-semibold text-blue-900 dark:text-blue-100 mb-3">
              {editingShortcut.id ? 'Edit Shortcut' : 'New Shortcut'}
            </h3>

            <div class="space-y-3">
              <div>
                <div class="flex items-center justify-between mb-1">
                  <label class="block text-xs text-gray-700 dark:text-gray-300">
                    Keyboard shortcut
                  </label>
                  <button
                    on:click={() => showShortcutHelp = !showShortcutHelp}
                    class="p-0.5 text-gray-400 hover:text-blue-500 dark:hover:text-blue-400 transition-colors"
                    title="Shortcut format help"
                  >
                    <HelpCircle size={14} />
                  </button>
                </div>
                {#if showShortcutHelp}
                  <div class="mb-2 p-3 bg-gray-100 dark:bg-gray-700/50 rounded-lg text-xs text-gray-600 dark:text-gray-300 space-y-2">
                    <p class="font-semibold text-gray-700 dark:text-gray-200">Format: [Modifier+...+]Key</p>
                    <p class="text-gray-500 dark:text-gray-400">Une touche obligatoire, modifiers optionnels (avant la touche).</p>
                    <div>
                      <span class="font-medium">Modifiers:</span>
                      <span class="ml-1">CommandOrControl, Shift, Alt, Super</span>
                    </div>
                    <div>
                      <span class="font-medium">Lettres / chiffres:</span>
                      <span class="ml-1">A-Z, 0-9</span>
                    </div>
                    <div>
                      <span class="font-medium">Fonctions:</span>
                      <span class="ml-1">F1-F24</span>
                    </div>
                    <div>
                      <span class="font-medium">Spéciales:</span>
                      <span class="ml-1">Space, Tab, Enter, Escape, Backspace, Delete</span>
                    </div>
                    <div>
                      <span class="font-medium">Navigation:</span>
                      <span class="ml-1">ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Home, End, PageUp, PageDown</span>
                    </div>
                    <div class="pt-1.5 border-t border-gray-200 dark:border-gray-600 space-y-1">
                      <p class="text-gray-500 dark:text-gray-400">Exemples :</p>
                      <div class="flex flex-wrap gap-1.5">
                        <kbd class="px-1.5 py-0.5 bg-white dark:bg-gray-600 border border-gray-200 dark:border-gray-500 rounded text-[10px]">CommandOrControl+Shift+T</kbd>
                        <kbd class="px-1.5 py-0.5 bg-white dark:bg-gray-600 border border-gray-200 dark:border-gray-500 rounded text-[10px]">Alt+K</kbd>
                        <kbd class="px-1.5 py-0.5 bg-white dark:bg-gray-600 border border-gray-200 dark:border-gray-500 rounded text-[10px]">F1</kbd>
                      </div>
                    </div>
                  </div>
                {/if}
                <input
                  type="text"
                  bind:value={editingShortcut.shortcut}
                  placeholder="e.g., CommandOrControl+Shift+T"
                  class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                />
              </div>

              <div>
                <label class="block text-xs text-gray-700 dark:text-gray-300 mb-1">
                  Prompt
                </label>
                <textarea
                  bind:value={editingShortcut.prompt}
                  placeholder="e.g., Translate this text to English"
                  maxlength={PROMPT_MAX_LENGTH}
                  rows="3"
                  class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white resize-none"
                ></textarea>
                <div class="mt-1 flex items-center justify-between">
                  <p class="text-xs text-gray-500 dark:text-gray-400">
                    Describe the action to perform on the selected text.
                  </p>
                  <span class="text-xs text-gray-400 dark:text-gray-500">
                    {editingShortcut.prompt?.length || 0}/{PROMPT_MAX_LENGTH}
                  </span>
                </div>
              </div>

              <div class="flex gap-2">
                <button
                  on:click={saveShortcut}
                  class="flex-1 px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                >
                  Save
                </button>
                <button
                  on:click={() => { editingShortcut = null; }}
                  class="flex-1 px-4 py-2 text-sm bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        {/if}
      </section>

      <!-- Updates Section -->
      <section class="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
          <span>⬆️</span> Updates
        </h2>

        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3 mb-3">
          <p class="text-sm text-gray-700 dark:text-gray-200">
            Installed version: <span class="font-mono font-medium">v{updateSettings.current_version || '…'}</span>
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Last check: {formatLastCheck(updateSettings.last_check)}
          </p>
          {#if updateState === 'up-to-date'}
            <p class="text-xs text-green-600 dark:text-green-400 mt-2">✓ rewrit is up to date</p>
          {:else if updateState === 'available'}
            <p class="text-xs text-blue-600 dark:text-blue-400 mt-2">
              v{availableVersion || 'new'} is available
            </p>
          {:else if updateState === 'installed'}
            <p class="text-xs text-green-600 dark:text-green-400 mt-2">Update installed — restart rewrit to use it</p>
          {:else if updateState === 'error'}
            <p class="text-xs text-red-600 dark:text-red-400 mt-2 break-words">{updateError}</p>
          {/if}
        </div>

        <div class="flex gap-2">
          <button
            on:click={checkForUpdate}
            disabled={updateState === 'checking' || updateState === 'installing'}
            class="flex-1 px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {updateState === 'checking' ? 'Checking…' : 'Check for updates…'}
          </button>
          {#if updateState === 'available' || updateState === 'installing'}
            <button
              on:click={installUpdate}
              disabled={updateState === 'installing'}
              class="flex-1 px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors flex items-center justify-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Download size={14} />
              {updateState === 'installing' ? 'Installing…' : 'Install & restart'}
            </button>
          {/if}
        </div>

        <div class="space-y-2 mt-4">
          <label class="flex items-center justify-between cursor-pointer py-1">
            <span class="text-sm text-gray-700 dark:text-gray-300">Check automatically</span>
            <input
              type="checkbox"
              checked={updateSettings.auto_check}
              on:change={toggleUpdateAutoCheck}
              class="w-5 h-5 text-blue-600 rounded focus:ring-blue-500"
            />
          </label>
          <label class="flex items-center justify-between cursor-pointer py-1">
            <span class="text-sm text-gray-700 dark:text-gray-300">Install automatically</span>
            <input
              type="checkbox"
              checked={updateSettings.auto_install}
              on:change={toggleUpdateAutoInstall}
              disabled={!updateSettings.auto_check}
              class="w-5 h-5 text-blue-600 rounded focus:ring-blue-500 disabled:opacity-50"
            />
          </label>
        </div>
      </section>

      <!-- Diagnostics Section -->
      <section class="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
          <span>🩺</span> Diagnostics
        </h2>

        {#if diagnostics}
          <dl class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
            <dt class="text-gray-500 dark:text-gray-400">Provider</dt>
            <dd class="text-gray-700 dark:text-gray-200">{diagnostics.provider}</dd>
            <dt class="text-gray-500 dark:text-gray-400">Model</dt>
            <dd class="text-gray-700 dark:text-gray-200 font-mono break-all">
              {diagnostics.model ?? 'No API key configured'}
            </dd>
            <dt class="text-gray-500 dark:text-gray-400">System</dt>
            <dd class="text-gray-700 dark:text-gray-200">{diagnostics.os} {diagnostics.arch}</dd>
            <dt class="text-gray-500 dark:text-gray-400">Logs</dt>
            <dd class="text-gray-700 dark:text-gray-200 font-mono break-all">{diagnostics.log_dir}</dd>
          </dl>
        {/if}

        {#if connectionTest}
          <div
            class="mt-3 rounded-lg p-3 text-xs break-words"
            class:bg-green-50={connectionTest.ok}
            class:text-green-800={connectionTest.ok}
            class:bg-red-50={!connectionTest.ok}
            class:text-red-800={!connectionTest.ok}
          >
            {#if connectionTest.ok}
              ✓ Connection OK ({connectionTest.duration_ms} ms) — “{connectionTest.output}”
            {:else}
              ✗ {connectionTest.error}
            {/if}
          </div>
        {/if}

        <div class="flex flex-wrap gap-2 mt-3">
          <button
            on:click={testConnection}
            disabled={isTestingConnection}
            class="px-3 py-1.5 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors flex items-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Stethoscope size={14} />
            {isTestingConnection ? 'Testing…' : 'Test AI connection'}
          </button>
          <button
            on:click={openLogs}
            class="px-3 py-1.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors flex items-center gap-1.5"
          >
            <FolderOpen size={14} />
            Open logs
          </button>
          <button
            on:click={copyDiagnostics}
            class="px-3 py-1.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors flex items-center gap-1.5"
          >
            <Copy size={14} />
            Copy diagnostics
          </button>
        </div>
      </section>

      <!-- About Section -->
      <section class="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-3 flex items-center gap-2">
          <span>ℹ️</span> About
        </h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">
          rewrit v{updateSettings.current_version} — AI-powered instant text transformation.
        </p>
        <div class="flex items-center gap-4 text-sm">
          <a
            href="https://github.com/YvanBetremieux/rewrit"
            target="_blank"
            class="flex items-center gap-1 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
          >
            <Book size={14} />
            GitHub
          </a>
          <a
            href="https://github.com/YvanBetremieux/rewrit/issues"
            target="_blank"
            class="flex items-center gap-1 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
          >
            <HelpCircle size={14} />
            Support
          </a>
        </div>
      </section>
    </div>
  </div>
</div>
