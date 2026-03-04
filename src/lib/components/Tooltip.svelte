<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade } from "svelte/transition";

  export let text: string = "";

  let show = false;
  let triggerRef: HTMLDivElement;
  let tooltipRef: HTMLDivElement | null = null;
  let x = 0;
  let y = 0;
  let mouseX = 0;
  let mouseY = 0;

  // Portal action to move tooltip to body to escape overflow:hidden and transform containers
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        if (node.parentNode) node.parentNode.removeChild(node);
      },
    };
  }

  async function updatePosition(e?: MouseEvent) {
    if (!show) return;

    if (e) {
      mouseX = e.clientX;
      mouseY = e.clientY;
    }

    // Wait for tooltip to be rendered
    await tick();
    if (!tooltipRef) return;

    const tRect = tooltipRef.getBoundingClientRect();
    const padding = 15;
    const offset = 15;

    // Position to the right of the cursor
    let targetX = mouseX + offset;
    let targetY = mouseY - tRect.height / 2; // Center vertically on cursor

    // Flip to left if hitting the right edge
    if (targetX + tRect.width > window.innerWidth - padding) {
      targetX = mouseX - tRect.width - offset;
    }

    // Keep within viewport bounds
    if (targetX < padding) targetX = padding;

    // Vertical boundary checks
    if (targetY < padding) targetY = padding;
    if (targetY + tRect.height > window.innerHeight - padding) {
      targetY = window.innerHeight - tRect.height - padding;
    }

    x = targetX;
    y = targetY;
  }

  function handleMouseEnter(e: MouseEvent) {
    mouseX = e.clientX;
    mouseY = e.clientY;
    show = true;
    updatePosition(e);
  }

  function handleMouseMove(e: MouseEvent) {
    if (show) {
      updatePosition(e);
    }
  }

  function handleMouseLeave() {
    show = false;
  }

  // Handle global scroll and resize to keep tooltip pinned
  onMount(() => {
    const handleEvents = () => {
      if (show) updatePosition();
    };

    window.addEventListener("scroll", handleEvents, true);
    window.addEventListener("resize", handleEvents);

    return () => {
      window.removeEventListener("scroll", handleEvents, true);
      window.removeEventListener("resize", handleEvents);
    };
  });
</script>

<div
  bind:this={triggerRef}
  class="inline-block max-w-full"
  on:mouseenter={handleMouseEnter}
  on:mousemove={handleMouseMove}
  on:mouseleave={handleMouseLeave}
  on:focusin={() => {
    show = true;
    updatePosition();
  }}
  on:focusout={handleMouseLeave}
  role="tooltip"
>
  <slot />
</div>

{#if show}
  <div
    bind:this={tooltipRef}
    use:portal
    transition:fade={{ duration: 100 }}
    class="fixed z-[10000] px-3 py-2 bg-gray-900/95 dark:bg-gray-800/95 backdrop-blur-sm text-white text-xs rounded-lg shadow-xl pointer-events-none whitespace-nowrap border border-gray-700/50 min-w-20 max-w-75"
    style="left: {x}px; top: {y}px;"
  >
    {#if text}
      <span class="block truncate">{text}</span>
    {:else}
      <slot name="content" />
    {/if}
  </div>
{/if}
