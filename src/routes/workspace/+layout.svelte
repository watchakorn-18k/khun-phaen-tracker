<script lang="ts">
  import { page } from "$app/stores";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import BookmarkManager from "$lib/components/BookmarkManager.svelte";
  import QuickNotes from "$lib/components/QuickNotes.svelte";
  import TimerDashboard from "$lib/components/TimerDashboard.svelte";
  import WhiteboardModal from "$lib/components/WhiteboardModal.svelte";
  import HtmlPreview from "$lib/components/HtmlPreview.svelte";
  import ProfileModal from "$lib/components/ProfileModal.svelte";
  import GlobalConfirmModal from "$lib/components/GlobalConfirmModal.svelte";
  import { modals, createUIActions } from "$lib/stores/uiActions";

  const ui = createUIActions();

  $: workspaceId = $page.params.workspace_id;

  let whiteboardMessage = "";
  let whiteboardMessageType: "success" | "error" = "success";
  let showProfileModal = false;

  function showToast(message: string, type: "success" | "error" = "success") {
    whiteboardMessage = message;
    whiteboardMessageType = type;
    setTimeout(() => {
      whiteboardMessage = "";
    }, 2500);
  }
  $: isPublicPage = $page.url.pathname.includes("/public/");
</script>

{#if isPublicPage}
  <main class="min-h-screen bg-gray-50 dark:bg-gray-900">
    <slot />
  </main>
{:else}
  <div class="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
    <Sidebar />

    <!-- Main Content -->
    <main class="flex-1 overflow-y-auto min-w-0 relative">
      {#key workspaceId}
        <slot />
      {/key}

      {#if whiteboardMessage}
        <div class="fixed top-4 right-4 z-[110] animate-fade-in">
          <div
            class="{whiteboardMessageType === 'success'
              ? 'bg-green-500'
              : 'bg-red-500'} text-white px-4 py-2.5 rounded-lg shadow-lg text-sm font-medium"
          >
            {whiteboardMessage}
          </div>
        </div>
      {/if}
    </main>
  </div>

  {#if $modals.bookmarkManager}
    <BookmarkManager on:close={() => ui.closeModal("bookmarkManager")} />
  {/if}

  {#if $modals.quickNotes}
    <QuickNotes on:close={() => ui.closeModal("quickNotes")} />
  {/if}

  {#if $modals.timerDashboard}
    <TimerDashboard />
  {/if}

  {#if $modals.htmlPreview}
    <HtmlPreview on:close={() => ui.closeModal("htmlPreview")} />
  {/if}

  {#if $modals.whiteboard}
    <WhiteboardModal
      open={$modals.whiteboard}
      on:close={() => ui.closeModal("whiteboard")}
      on:notify={(event) => showToast(event.detail.message, event.detail.type)}
    />
  {/if}

  {#if showProfileModal}
    <ProfileModal
      open={showProfileModal}
      on:close={() => (showProfileModal = false)}
      on:notify={(event) => showToast(event.detail.message, event.detail.type)}
    />
  {/if}

  <GlobalConfirmModal />
{/if}
