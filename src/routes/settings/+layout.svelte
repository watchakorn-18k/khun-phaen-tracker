<script lang="ts">
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { page } from "$app/stores";
  import { browser } from "$app/environment";
  import { authLoading, user } from "$lib/stores/auth";
  import { _ } from "svelte-i18n";
  import { Settings, Shield, Users, ChevronRight } from "lucide-svelte";

  function normalizePath(pathname: string) {
    if (pathname.length > 1 && pathname.endsWith("/")) {
      return pathname.slice(0, -1);
    }
    return pathname;
  }

  const tabs = [
    {
      href: `${base}/settings`,
      label: "settings__tab_general",
      description: "settings__tab_general_desc",
      icon: Settings,
      match: (pathname: string) =>
        normalizePath(pathname) === normalizePath(`${base}/settings`),
    },
    {
      href: `${base}/settings/users`,
      label: "settings__tab_users",
      description: "settings__tab_users_desc",
      icon: Users,
      match: (pathname: string) =>
        normalizePath(pathname).startsWith(
          normalizePath(`${base}/settings/users`),
        ),
    },
  ];

  $: if (browser && !$authLoading && $user?.role && $user.role !== "admin") {
    goto(`${base}/dashboard`);
  }
</script>

<div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
  <div class="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
    <aside class="lg:sticky lg:top-24 lg:self-start">
      <div class="overflow-hidden rounded-3xl border border-gray-200 bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="border-b border-gray-200 bg-gradient-to-br from-slate-900 via-slate-800 to-indigo-950 p-5 text-white dark:border-gray-700">
          <div class="mb-3 inline-flex rounded-2xl bg-white/10 p-3 ring-1 ring-white/15 backdrop-blur">
            <Shield size={20} />
          </div>
          <h1 class="text-lg font-bold tracking-tight">{$_("settings__title")}</h1>
          <p class="mt-1 text-sm text-slate-200/80">
            {$_("settings__subtitle")}
          </p>
        </div>

        <nav class="space-y-2 p-3">
          {#each tabs as tab}
            <a
              href={tab.href}
              class={`group flex items-start gap-3 rounded-2xl px-4 py-3 transition-all ${
                tab.match($page.url.pathname)
                  ? "bg-indigo-50 text-indigo-700 ring-1 ring-indigo-200 dark:bg-indigo-500/10 dark:text-indigo-300 dark:ring-indigo-500/20"
                  : "text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-white/5 dark:hover:text-white"
              }`}
            >
              <div
                class={`mt-0.5 rounded-xl p-2 ${
                  tab.match($page.url.pathname)
                    ? "bg-indigo-100 text-indigo-700 dark:bg-indigo-500/10 dark:text-indigo-300"
                    : "bg-gray-100 text-gray-500 group-hover:bg-white dark:bg-gray-700 dark:text-gray-300 dark:group-hover:bg-gray-700"
                }`}
              >
                <svelte:component this={tab.icon} size={16} />
              </div>

              <div class="min-w-0 flex-1">
                <div class="flex items-center justify-between gap-3">
                  <span class="text-sm font-semibold">{$_(tab.label)}</span>
                  <ChevronRight
                    size={14}
                    class={tab.match($page.url.pathname)
                      ? "opacity-100"
                      : "opacity-40 transition-opacity group-hover:opacity-80"}
                  />
                </div>
                <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                  {$_(tab.description)}
                </p>
              </div>
            </a>
          {/each}
        </nav>
      </div>
    </aside>

    <section class="min-w-0">
      <slot />
    </section>
  </div>
</div>
