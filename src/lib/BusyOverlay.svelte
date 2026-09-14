<script lang="ts">
  interface Props {
    /** What is happening, in the user's language. */
    what: string;
  }

  let { what }: Props = $props();
</script>

<!--
  Modal on purpose. An attach takes seconds, and during it the device is being
  handed between Windows and WSL — a second operation started in the middle of
  that would act on a state that no longer holds. Showing progress in a corner
  was not enough: the rest of the window still looked usable, which invited
  exactly the clicks that must not happen.
-->
<div class="overlay" role="alertdialog" aria-busy="true" aria-label={what}>
  <div class="card">
    <span class="spinner"></span>
    <span>{what}</span>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--bg) 55%, transparent);
    /* Swallows every click, which is the point. */
    cursor: wait;
  }

  .card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 22px;
    font-size: 13px;
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.4);
  }

  .spinner {
    flex: 0 0 auto;
    width: 14px;
    height: 14px;
    border: 2px solid color-mix(in srgb, var(--accent) 30%, transparent);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(1turn);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spinner {
      animation-duration: 2s;
    }
  }
</style>
