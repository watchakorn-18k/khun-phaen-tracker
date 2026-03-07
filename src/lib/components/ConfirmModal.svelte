<script lang="ts">
  import { fade, scale } from "svelte/transition";
  import { X, AlertTriangle, CheckCircle2, CircleAlert } from "lucide-svelte";
  import { _ } from "svelte-i18n";

  let {
    show = false,
    title = "",
    message = "",
    confirmText = "",
    cancelText = "",
    type = "danger" as "danger" | "warning" | "info",
    onConfirm,
    onClose,
  } = $props<{
    show?: boolean;
    title?: string;
    message?: string;
    confirmText?: string;
    cancelText?: string;
    type?: "danger" | "warning" | "info";
    onConfirm?: () => void;
    onClose?: () => void;
  }>();
</script>

{#if show}
  <div
    class="fixed inset-0 z-[30000] flex items-center justify-center bg-slate-950/50 p-4 backdrop-blur-md dark:bg-slate-950/80"
    in:fade={{ duration: 300 }}
    out:fade={{ duration: 250 }}
  >
    <div
      class="flex w-full max-w-md flex-col overflow-hidden rounded-3xl border border-slate-200 bg-white text-slate-900 shadow-[0_32px_64px_-16px_rgba(15,23,42,0.28)] dark:border-white/10 dark:bg-slate-900 dark:text-white dark:shadow-[0_32px_64px_-16px_rgba(0,0,0,0.5)]"
      in:scale={{ start: 0.95, duration: 400, opacity: 0 }}
      out:scale={{ start: 0.95, duration: 250, opacity: 0 }}
    >
      <!-- Header -->
      <div
        class="flex items-center justify-between border-b border-slate-200 px-6 py-4 dark:border-white/10"
      >
        <div class="flex items-center gap-3">
          <div
            class={type === "danger"
              ? "text-rose-500"
              : type === "warning"
                ? "text-amber-500"
                : "text-blue-500"}
          >
            {#if type === "danger"}
              <AlertTriangle size={22} strokeWidth={2.5} />
            {:else if type === "warning"}
              <CircleAlert size={22} strokeWidth={2.5} />
            {:else}
              <CheckCircle2 size={22} strokeWidth={2.5} />
            {/if}
          </div>
          <h3 class="text-lg font-bold tracking-tight text-slate-900 dark:text-white">
            {title || $_("common.confirm")}
          </h3>
        </div>
        <button
          onclick={() => onClose?.()}
          class="rounded-xl p-2 text-slate-400 transition-all active:scale-90 hover:bg-slate-100 hover:text-slate-700 dark:text-slate-500 dark:hover:bg-white/10 dark:hover:text-white"
        >
          <X size={20} />
        </button>
      </div>

      <!-- Body -->
      <div class="px-8 py-10">
        <p class="text-base font-medium leading-relaxed text-slate-600 dark:text-slate-400">
          {message}
        </p>
      </div>

      <!-- Footer -->
      <div class="flex justify-end gap-3 border-t border-slate-200 px-6 py-5 dark:border-white/10">
        <button
          onclick={() => onClose?.()}
          class="rounded-xl border border-slate-300 px-6 py-2.5 text-sm font-bold text-slate-700 transition-all active:scale-95 hover:bg-slate-100 hover:text-slate-900 dark:border-white/10 dark:text-slate-300 dark:hover:bg-white/5 dark:hover:text-white"
        >
          {cancelText || $_("common.cancel")}
        </button>
        <button
          onclick={() => onConfirm?.()}
          class="rounded-xl px-8 py-2.5 text-sm font-bold text-white shadow-lg transition-all active:scale-95 {type ===
          'danger'
            ? 'bg-rose-600 hover:bg-rose-500 shadow-rose-900/30 dark:shadow-rose-900/40'
            : type === 'warning'
              ? 'bg-amber-500 hover:bg-amber-400 shadow-amber-900/25 dark:shadow-amber-900/40'
              : 'bg-blue-600 hover:bg-blue-500 shadow-blue-900/25 dark:shadow-blue-900/40'}"
        >
          {confirmText || $_("common.confirm")}
        </button>
      </div>
    </div>
  </div>
{/if}
