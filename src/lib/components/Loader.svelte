<div class="loader-container">
  <div class="spinner"></div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent;
  }

  .loader-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(99, 102, 241, 0.2);
    border-top-color: #6366f1;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>

