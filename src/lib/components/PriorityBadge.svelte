<script lang="ts">
  import {
    Minus,
    AlertTriangle,
    SignalHigh,
    SignalMedium,
    SignalLow,
  } from "lucide-svelte";
  import { _ } from "svelte-i18n";

  export let priority: "urgent" | "high" | "medium" | "low" | "none" | undefined | null;

  $: config = getPriorityConfig(priority, $_);

  function getPriorityConfig(p: string | undefined | null, t: any) {
    switch (p) {
      case "urgent":
        return { label: t("priority__urgent", { default: "Urgent" }), icon: AlertTriangle, pillClass: "bg-orange-500 text-white px-1.5 py-0.5 rounded shadow-sm border border-orange-600", iconClass: "text-white" };
      case "high":
        return { label: t("priority__high", { default: "High" }), icon: SignalHigh, pillClass: "bg-orange-600 text-white px-1.5 py-0.5 rounded shadow-sm border border-orange-700", iconClass: "text-white" };
      case "medium":
        return { label: t("priority__medium", { default: "Medium" }), icon: SignalMedium, pillClass: "bg-orange-900/40 text-orange-600 dark:text-orange-400 px-1.5 py-0.5 rounded border border-orange-900/50", iconClass: "text-orange-600 dark:text-orange-400" };
      case "low":
        return { label: t("priority__low", { default: "Low" }), icon: SignalLow, pillClass: "bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400 px-1.5 py-0.5 rounded border border-gray-200 dark:border-gray-700", iconClass: "text-gray-500 dark:text-gray-400" };
      default:
        return null; // Return null to not render anything for 'none' or undefined
    }
  }
</script>

{#if config}
  <span class="inline-flex items-center gap-1 text-[10px] font-medium {config.pillClass}" title={config.label}>
    <svelte:component this={config.icon} size={10} class={config.iconClass} />
    {config.label}
  </span>
{/if}
