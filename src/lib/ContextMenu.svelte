<script lang="ts">
  interface Item {
    label: string;
    /** Absent means the item is enabled. */
    disabledReason?: string | null;
    action: () => void;
  }

  interface Props {
    x: number;
    y: number;
    items: Item[];
    onclose: () => void;
  }

  let { x, y, items, onclose }: Props = $props();

  let menu = $state<HTMLDivElement | null>(null);

  /**
   * Keep the menu inside the window.
   *
   * Measured after mount rather than guessed at, since the widest item decides
   * the width and that changes with the translation.
   */
  const position = $derived.by(() => {
    if (!menu) return { left: x, top: y };
    const { width, height } = menu.getBoundingClientRect();
    return {
      left: Math.min(x, window.innerWidth - width - 4),
      top: Math.min(y, window.innerHeight - height - 4),
    };
  });

  function choose(item: Item) {
    if (item.disabledReason) return;
    item.action();
    onclose();
  }
</script>

<svelte:window
  onkeydown={(e) => e.key === "Escape" && onclose()}
  onresize={onclose}
/>

<!-- Catches the click that dismisses the menu, including a right-click
     elsewhere, so the menu never lingers over unrelated content. -->
<div
  class="backdrop"
  role="presentation"
  onclick={onclose}
  oncontextmenu={(e) => {
    e.preventDefault();
    onclose();
  }}
></div>

<div
  class="menu"
  role="menu"
  tabindex="-1"
  bind:this={menu}
  style="left: {position.left}px; top: {position.top}px"
>
  {#each items as item (item.label)}
    <button
      role="menuitem"
      disabled={!!item.disabledReason}
      title={item.disabledReason ?? undefined}
      onclick={() => choose(item)}
    >
      {item.label}
    </button>
  {/each}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
  }

  .menu {
    position: fixed;
    min-width: 190px;
    padding: 4px;
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.35);
    display: flex;
    flex-direction: column;
  }

  .menu button {
    all: unset;
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 13px;
    cursor: default;
    white-space: nowrap;
  }

  .menu button:hover:not(:disabled) {
    background: var(--bg-selected);
  }

  .menu button:disabled {
    color: var(--fg-faint);
  }
</style>
