<script lang="ts">
  import { Bell, CheckCheck } from "lucide-svelte";
  import {
    markAllNotificationsRead,
    markNotificationRead,
    notificationItems,
    unreadNotificationCount,
    type AppNotification,
  } from "$lib/stores/notifications";
  import { goto } from "$app/navigation";

  let open = false;

  function openNotification(item: AppNotification) {
    markNotificationRead(item.id);
    open = false;
    goto(item.href);
  }

  function formatTime(value: string) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "";
    return date.toLocaleString("th-TH", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

<div class="relative">
  <button
    type="button"
    class="relative inline-flex h-10 w-10 items-center justify-center rounded-lg text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
    title="Notifications"
    on:click={() => (open = !open)}
  >
    <Bell size={20} />
    {#if $unreadNotificationCount > 0}
      <span
        class="absolute right-1 top-1 flex min-w-4 items-center justify-center rounded-full bg-red-500 px-1 text-[10px] font-bold leading-4 text-white"
      >
        {$unreadNotificationCount > 9 ? "9+" : $unreadNotificationCount}
      </span>
    {/if}
  </button>

  {#if open}
    <button
      type="button"
      aria-label="Close notifications"
      class="fixed inset-0 z-20 cursor-default"
      on:click={() => (open = false)}
    ></button>

    <div
      class="absolute right-0 top-full z-30 mt-3 w-[min(360px,calc(100vw-24px))] overflow-hidden rounded-xl border border-gray-200 bg-white shadow-2xl ring-1 ring-black/5 dark:border-gray-700 dark:bg-gray-800 dark:ring-white/5"
    >
      <div
        class="flex items-center justify-between border-b border-gray-100 px-4 py-3 dark:border-gray-700"
      >
        <div>
          <p class="text-sm font-bold text-gray-900 dark:text-white">
            Notifications
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            {$unreadNotificationCount} unread
          </p>
        </div>
        <button
          type="button"
          class="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-semibold text-gray-600 transition-colors hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
          on:click={markAllNotificationsRead}
          disabled={$notificationItems.length === 0}
        >
          <CheckCheck size={14} />
          Read all
        </button>
      </div>

      <div class="max-h-[420px] overflow-y-auto py-1">
        {#if $notificationItems.length === 0}
          <div class="px-4 py-8 text-center text-sm text-gray-500 dark:text-gray-400">
            No notifications
          </div>
        {:else}
          {#each $notificationItems as item (item.id)}
            <button
              type="button"
              class="flex w-full gap-3 px-4 py-3 text-left transition-colors hover:bg-gray-50 dark:hover:bg-gray-700/60"
              on:click={() => openNotification(item)}
            >
              <span
                class="mt-1 h-2.5 w-2.5 shrink-0 rounded-full {item.readAt
                  ? 'bg-gray-300 dark:bg-gray-600'
                  : 'bg-indigo-500'}"
              ></span>
              <span class="min-w-0 flex-1">
                <span class="block truncate text-sm font-semibold text-gray-900 dark:text-white">
                  {item.sourceTitle}
                </span>
                <span class="mt-0.5 block text-sm leading-5 text-gray-600 dark:text-gray-300">
                  {item.message}
                </span>
                <span class="mt-1 block text-xs text-gray-400 dark:text-gray-500">
                  {formatTime(item.createdAt)}
                </span>
              </span>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>
