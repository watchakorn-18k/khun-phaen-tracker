<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    currentWorkspaceId,
    currentWorkspaceOwnerId,
  } from "$lib/stores/workspace";
  import { api } from "$lib/apis";
  import type { ChecklistTemplate, ChecklistItem } from "$lib/types";
  import ChecklistTemplateModal from "./ChecklistTemplateModal.svelte";
  import { Check, X, Trash2, ListTodo, ClipboardList } from "lucide-svelte";
  import { _ } from "svelte-i18n";

  const dispatch = createEventDispatcher<{
    update: { checklist: ChecklistItem[] };
  }>();

  export let checklist: ChecklistItem[] = [];
  export let autoDispatch = true; // Auto dispatch on changes (for edit mode)

  let newChecklistItem = "";
  let isAddingChecklistItem = false;
  let isBulkAddModalOpen = false;
  let bulkChecklistText = "";
  let deletingChecklistItemId: string | null = null;
  let checklistVisibleCount = 10;
  let checklistSelectMode = false;
  let selectedChecklistIds: Set<string> = new Set();

  $: completedCount = checklist.filter((i) => i.completed).length;
  $: progress =
    checklist.length > 0
      ? Math.round((completedCount / checklist.length) * 100)
      : 0;
  $: visibleChecklist = checklist.slice(0, checklistVisibleCount);
  $: hasMoreChecklist = checklist.length > checklistVisibleCount;

  // Template management
  let showTemplateModal = false;
  let templates: ChecklistTemplate[] = [];
  let isLoadingTemplates = false;
  $: isOwner = $currentWorkspaceOwnerId === null || true; // Fallback or logic needed

  // Local toast for simple feedback without external lib
  let toastMessage = "";
  let toastType: "success" | "error" = "success";
  let toastTimeout: any;

  function showLocalToast(msg: string, type: "success" | "error" = "success") {
    toastMessage = msg;
    toastType = type;
    clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
      toastMessage = "";
    }, 3000);
  }

  function notifyUpdate() {
    if (autoDispatch) {
      dispatch("update", { checklist });
    }
  }

  function addChecklistItem() {
    if (!newChecklistItem.trim()) return;
    checklist = [
      ...checklist,
      {
        id: Date.now().toString() + Math.random().toString(36).substring(2),
        text: newChecklistItem.trim(),
        completed: false,
      },
    ];
    newChecklistItem = "";
    notifyUpdate();
  }

  function createChecklistItemId() {
    return Date.now().toString() + Math.random().toString(36).substring(2);
  }

  function openBulkAddModal() {
    isBulkAddModalOpen = true;
    bulkChecklistText = "";
  }

  function closeBulkAddModal() {
    isBulkAddModalOpen = false;
    bulkChecklistText = "";
  }

  function addChecklistItemsFromBulkText() {
    const lines = bulkChecklistText
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);

    if (lines.length === 0) {
      closeBulkAddModal();
      return;
    }

    const newItems: ChecklistItem[] = lines.map((text) => ({
      id: createChecklistItemId(),
      text,
      completed: false,
    }));

    checklist = [...checklist, ...newItems];
    notifyUpdate();
    closeBulkAddModal();
  }

  function removeChecklistItem(id: string) {
    checklist = checklist.filter((item) => item.id !== id);
    notifyUpdate();
  }

  function toggleChecklistItem(id: string) {
    checklist = checklist.map((item) =>
      item.id === id ? { ...item, completed: !item.completed } : item,
    );
    notifyUpdate();
  }

  function deleteAll() {
    checklist = [];
    selectedChecklistIds = new Set();
    checklistSelectMode = false;
    notifyUpdate();
  }

  function deleteSelected() {
    checklist = checklist.filter((i) => !selectedChecklistIds.has(i.id));
    selectedChecklistIds = new Set();
    if (checklist.length === 0) checklistSelectMode = false;
    notifyUpdate();
  }

  function toggleSelectAll() {
    if (selectedChecklistIds.size === checklist.length) {
      selectedChecklistIds = new Set();
    } else {
      selectedChecklistIds = new Set(checklist.map((i) => i.id));
    }
  }

  function toggleSelectMode() {
    checklistSelectMode = !checklistSelectMode;
    if (!checklistSelectMode) selectedChecklistIds = new Set();
  }

  function toggleSelectItem(id: string) {
    const next = new Set(selectedChecklistIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedChecklistIds = next;
  }

  async function loadTemplates() {
    if (!$currentWorkspaceId) return;
    isLoadingTemplates = true;
    try {
      const res = await api.data.checklistTemplates.list($currentWorkspaceId);
      if (res.ok) {
        const data = await res.json();
        templates = (data.templates || []).map((t: any) => ({
          ...t,
          id: t.id || t._id,
        }));
      }
    } catch (err) {
      console.error("Failed to load templates:", err);
    } finally {
      isLoadingTemplates = false;
    }
  }

  function openTemplates() {
    showTemplateModal = true;
    loadTemplates();
  }

  async function handleAddTemplate(event: any) {
    if (!$currentWorkspaceId) return;
    try {
      const res = await api.data.checklistTemplates.create(
        $currentWorkspaceId,
        event.detail,
      );
      if (res.ok) {
        showLocalToast($_("checklistTemplate__save_success"), "success");
        loadTemplates();
      }
    } catch (err) {
      showLocalToast($_("checklistManager__error_save"), "error");
    }
  }

  async function handleUpdateTemplate(event: any) {
    if (!$currentWorkspaceId) return;
    try {
      const { id, ...updates } = event.detail;
      const res = await api.data.checklistTemplates.update(
        $currentWorkspaceId,
        id,
        updates,
      );
      if (res.ok) {
        showLocalToast($_("checklistTemplate__save_success"), "success");
        loadTemplates();
      }
    } catch (err) {
      showLocalToast($_("checklistManager__error_update"), "error");
    }
  }

  async function handleDeleteTemplate(event: any) {
    if (!$currentWorkspaceId) return;
    try {
      const res = await api.data.checklistTemplates.delete(
        $currentWorkspaceId,
        event.detail,
      );
      if (res.ok) {
        loadTemplates();
      }
    } catch (err) {
      showLocalToast($_("checklistManager__error_delete"), "error");
    }
  }

  function handleImportTemplate(event: any) {
    const template: ChecklistTemplate = event.detail;
    const newItems: ChecklistItem[] = template.items.map((text) => ({
        id: createChecklistItemId(),
        text,
        completed: false,
      }));

    checklist = [...checklist, ...newItems];
    notifyUpdate();
    showLocalToast($_("checklistTemplate__import_success"), "success");
    showTemplateModal = false;
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <ListTodo size={18} class="text-gray-700 dark:text-gray-300" />
      <span class="text-sm font-bold text-gray-900 dark:text-white"
        >{$_("taskForm__checklist_label")}</span
      >
    </div>
    <div class="flex items-center gap-1.5">
      <button
        type="button"
        on:click={openTemplates}
        class="flex items-center gap-2 px-3 py-1.5 text-[10px] font-black uppercase tracking-widest text-primary hover:bg-primary/10 rounded-xl border border-primary/30 transition-all active:scale-95 shadow-xs shadow-primary/10 hover:shadow-primary/20 bg-white/50 dark:bg-gray-900/50 backdrop-blur-sm"
      >
        <ClipboardList
          size={14}
          class="opacity-80 transition-transform group-hover:scale-110"
        />
        {$_("taskForm__checklist_templates")}
      </button>

      {#if checklist.length > 0}
        <button
          type="button"
          on:click={toggleSelectMode}
          class="px-3 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-xl transition-all active:scale-95 {checklistSelectMode
            ? 'text-white bg-linear-to-r from-blue-500 to-indigo-600 shadow-lg shadow-blue-500/30 ring-2 ring-blue-500/20'
            : 'text-gray-500 hover:text-gray-900 hover:bg-white dark:hover:text-gray-200 dark:hover:bg-gray-800 border-2 border-transparent hover:border-gray-100 dark:hover:border-gray-700'}"
        >
          {checklistSelectMode
            ? $_("checklistManager__select_mode_done")
            : $_("checklistManager__select_mode_select")}
        </button>
        <button
          type="button"
          on:click={deleteAll}
          class="px-3 py-1.5 text-[10px] font-black uppercase tracking-widest text-gray-500 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-xl transition-all active:scale-95 group/del"
        >
          <span
            class="group-hover/del:scale-110 transition-transform inline-block"
            >{$_("checklistManager__delete_all")}</span
          >
        </button>
      {/if}
    </div>
  </div>

  {#if checklistSelectMode && checklist.length > 0}
    <div
      class="flex items-center gap-2 py-1 px-1 bg-gray-50 dark:bg-gray-700/30 rounded-lg border border-gray-100 dark:border-gray-700"
    >
      <button
        type="button"
        on:click={toggleSelectAll}
        class="px-2.5 py-1 text-xs font-bold text-blue-500 hover:text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors"
      >
        {selectedChecklistIds.size === checklist.length
          ? $_("checklistManager__deselect_all")
          : $_("checklistManager__select_all")}
      </button>
      {#if selectedChecklistIds.size > 0}
        <button
          type="button"
          on:click={deleteSelected}
          class="px-2.5 py-1 text-xs font-bold text-red-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors flex items-center gap-1"
        >
          <Trash2 size={12} />
          {$_("checklistManager__delete_selected", {
            values: { count: selectedChecklistIds.size },
          })}
        </button>
      {/if}
      <span
        class="text-[10px] font-bold text-gray-400 ml-auto uppercase tracking-wider"
        >{selectedChecklistIds.size}/{checklist.length}</span
      >
    </div>
  {/if}

  <div class="space-y-2">
    <div class="flex items-center gap-3">
      <span
        class="text-[10px] font-black w-7 {progress === 100
          ? 'text-green-500'
          : 'text-primary'}">{progress}%</span
      >
      <div
        class="flex-1 h-1.5 bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden shadow-inner font-bold"
      >
        <div
          class="h-full bg-linear-to-r from-primary to-indigo-500 rounded-full w-full opacity-50"
        ></div>
        <div
          class="h-full transition-all duration-700 ease-out {progress === 100
            ? 'bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.3)]'
            : 'bg-primary shadow-[0_0_10px_rgba(var(--primary-rgb),0.3)]'}"
          style="width: {progress}%"
        ></div>
      </div>
    </div>
  </div>

  <div class="space-y-1">
    {#each visibleChecklist as item (item.id)}
      <div
        class="flex items-start gap-2 group py-1.5 px-2 rounded-xl transition-all {checklistSelectMode &&
        selectedChecklistIds.has(item.id)
          ? 'bg-primary/5 border-primary/10'
          : 'hover:bg-gray-50 dark:hover:bg-gray-800/50'}"
      >
        {#if checklistSelectMode}
          <button
            type="button"
            on:click={() => toggleSelectItem(item.id)}
            class="mt-0.5 w-5 h-5 flex items-center justify-center rounded-lg border-2 transition-all shrink-0 {selectedChecklistIds.has(
              item.id,
            )
              ? 'bg-primary border-primary shadow-lg shadow-primary/20'
              : 'border-gray-200 dark:border-gray-700 hover:border-primary'}"
          >
            {#if selectedChecklistIds.has(item.id)}
              <Check size={12} strokeWidth={4} class="text-white" />
            {/if}
          </button>
        {:else}
          <button
            type="button"
            on:click={() => toggleChecklistItem(item.id)}
            class="mt-0.5 w-5 h-5 flex items-center justify-center rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-primary transition-all shrink-0 bg-white dark:bg-gray-900"
          >
            {#if item.completed}
              <Check size={12} strokeWidth={4} class="text-primary" />
            {/if}
          </button>
        {/if}

        <div class="flex-1 min-w-0">
          <input
            type="text"
            bind:value={item.text}
            on:change={notifyUpdate}
            class="w-full bg-transparent border-none focus:ring-0 text-sm p-0 font-semibold transition-all {item.completed
              ? 'text-gray-400 line-through opacity-50'
              : 'text-gray-800 dark:text-gray-200'}"
          />
        </div>

        <div
          class="flex items-center gap-1 transition-opacity {deletingChecklistItemId ===
          item.id
            ? 'opacity-100'
            : 'opacity-0 group-hover:opacity-100'}"
        >
          {#if deletingChecklistItemId === item.id}
            <button
              type="button"
              on:click={() => {
                removeChecklistItem(item.id);
                deletingChecklistItemId = null;
              }}
              class="w-7 h-7 flex items-center justify-center rounded-lg bg-transparent hover:bg-emerald-500/10 transition-colors"
              aria-label="Confirm delete checklist item"
            >
              <Check
                size={14}
                strokeWidth={3}
                class="text-emerald-500 hover:text-emerald-400"
              />
            </button>
            <button
              type="button"
              on:click={() => (deletingChecklistItemId = null)}
              class="w-7 h-7 flex items-center justify-center rounded-lg bg-transparent hover:bg-red-500/10 transition-colors"
              aria-label="Cancel delete checklist item"
            >
              <X size={14} strokeWidth={3} class="text-red-500 hover:text-red-400" />
            </button>
          {:else}
            <button
              type="button"
              on:click={() => (deletingChecklistItemId = item.id)}
              class="w-7 h-7 flex items-center justify-center text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-lg transition-colors"
            >
              <Trash2 size={14} />
            </button>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  {#if hasMoreChecklist}
    <button
      type="button"
      on:click={() => (checklistVisibleCount += 10)}
      class="w-full py-2 text-[10px] font-black uppercase tracking-widest text-primary hover:bg-primary/5 rounded-xl transition-all border border-dashed border-primary/20"
    >
      {$_("checklistManager__show_more", {
        values: {
          count: Math.min(10, checklist.length - checklistVisibleCount),
          current: checklistVisibleCount,
          total: checklist.length,
        },
      })}
    </button>
  {/if}

  {#if !isAddingChecklistItem}
    <div class="flex flex-wrap items-center gap-2">
      <button
        type="button"
        on:click={() => (isAddingChecklistItem = true)}
        class="flex items-center gap-2 px-4 py-2 text-sm font-bold text-gray-600 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-700 rounded-xl transition-all shadow-sm active:scale-95 group"
      >
        <div
          class="w-5 h-5 rounded-full bg-gray-100 dark:bg-gray-900 flex items-center justify-center group-hover:bg-primary group-hover:text-white transition-colors"
        >
          <X size={14} class="rotate-45" />
        </div>
        {$_("checklistManager__add_item")}
      </button>
      <button
        type="button"
        on:click={openBulkAddModal}
        class="flex items-center gap-2 px-4 py-2 text-sm font-bold text-primary bg-primary/5 hover:bg-primary/10 border border-primary/20 rounded-xl transition-all active:scale-95"
      >
        <ListTodo size={14} />
        {$_("checklistManager__add_multiple")}
      </button>
    </div>
  {:else}
    <div
      class="space-y-3 p-4 bg-gray-50 dark:bg-gray-900/50 rounded-2xl border border-gray-100 dark:border-gray-800 animate-in fade-in slide-in-from-top-2 duration-200"
    >
      <input
        type="text"
        bind:value={newChecklistItem}
        on:keydown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            if (newChecklistItem.trim()) addChecklistItem();
            else isAddingChecklistItem = false;
          } else if (e.key === "Escape") {
            isAddingChecklistItem = false;
            newChecklistItem = "";
          }
        }}
        placeholder={$_("checklistManager__placeholder")}
        class="w-full px-4 py-2.5 text-sm bg-white dark:bg-gray-800 border-2 border-primary/20 rounded-xl focus:border-primary outline-none transition-all text-gray-900 dark:text-white shadow-sm"
        use:focusOnMount
      />
      <div class="flex items-center gap-2">
        <button
          type="button"
          on:click={() => {
            if (newChecklistItem.trim()) addChecklistItem();
            else isAddingChecklistItem = false;
          }}
          class="px-5 py-2 bg-primary hover:bg-primary-dark text-white text-sm font-bold rounded-xl transition-all shadow-lg shadow-primary/20 active:scale-95"
        >
          {$_("checklistManager__add_item")}
        </button>
        <button
          type="button"
          on:click={() => {
            isAddingChecklistItem = false;
            newChecklistItem = "";
          }}
          class="px-4 py-2 text-gray-500 dark:text-gray-400 text-sm font-bold hover:text-gray-900 dark:hover:text-white transition-colors"
        >
          {$_("checklistManager__cancel")}
        </button>
      </div>
    </div>
  {/if}
</div>

{#if showTemplateModal}
  <ChecklistTemplateModal
    {templates}
    {isOwner}
    showImport={true}
    on:close={() => (showTemplateModal = false)}
    on:add={handleAddTemplate}
    on:update={handleUpdateTemplate}
    on:delete={handleDeleteTemplate}
    on:import={handleImportTemplate}
  />
{/if}

{#if isBulkAddModalOpen}
  <div
    class="fixed inset-0 z-[10001] bg-black/50 backdrop-blur-sm flex items-center justify-center p-4"
    on:click={closeBulkAddModal}
    role="button"
    tabindex="0"
    on:keydown={(e) => e.key === "Escape" && closeBulkAddModal()}
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-2xl shadow-2xl w-full max-w-lg p-5 space-y-4"
      on:mousedown|stopPropagation
      on:click|stopPropagation
    >
      <div class="space-y-1">
        <h3 class="text-base font-bold text-gray-900 dark:text-white">
          {$_("checklistManager__bulk_modal_title")}
        </h3>
        <p class="text-xs text-gray-500 dark:text-gray-400">
          {$_("checklistManager__bulk_modal_desc")}
        </p>
      </div>
      <textarea
        bind:value={bulkChecklistText}
        rows="8"
        placeholder={$_("checklistManager__bulk_modal_placeholder")}
        class="w-full px-3 py-2 text-sm bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-xl focus:ring-2 focus:ring-primary focus:border-primary outline-none transition-colors text-gray-900 dark:text-white"
        on:keydown={(e) => {
          if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
            e.preventDefault();
            addChecklistItemsFromBulkText();
          }
        }}
      ></textarea>
      <div class="flex justify-end items-center gap-2">
        <button
          type="button"
          on:click={closeBulkAddModal}
          class="px-4 py-2 text-sm font-bold text-gray-500 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
        >
          {$_("checklistManager__cancel")}
        </button>
        <button
          type="button"
          on:click={addChecklistItemsFromBulkText}
          class="px-4 py-2 text-sm font-bold text-white bg-primary hover:bg-primary-dark rounded-xl transition-colors"
        >
          {$_("checklistManager__bulk_modal_add")}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if toastMessage}
  <div
    class="fixed top-20 right-4 z-20002 animate-in fade-in slide-in-from-top-4 duration-300 pointer-events-none"
  >
    <div
      class="px-5 py-3 rounded-2xl shadow-2xl flex items-center gap-3 border backdrop-blur-md {toastType ===
      'success'
        ? 'bg-green-500/90 border-green-400 text-white shadow-green-500/20'
        : 'bg-red-500/90 border-red-400 text-white shadow-red-500/20'}"
    >
      <div
        class="w-6 h-6 rounded-full bg-white/20 flex items-center justify-center"
      >
        {#if toastType === "success"}
          <Check size={14} strokeWidth={4} />
        {:else}
          <X size={14} strokeWidth={4} />
        {/if}
      </div>
      <span class="font-bold text-sm tracking-wide">{toastMessage}</span>
    </div>
  </div>
{/if}

<style>
  .animate-in {
    animation-fill-mode: forwards;
  }
</style>
