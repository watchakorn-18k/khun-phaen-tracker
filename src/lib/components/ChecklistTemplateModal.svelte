<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { ChecklistTemplate } from "$lib/types";
  import {
    ClipboardList,
    Plus,
    X,
    Edit2,
    Trash2,
    Check,
    Download,
    ListChecks,
  } from "lucide-svelte";
  import { _ } from "svelte-i18n";

  const dispatch = createEventDispatcher<{
    close: void;
    add: { name: string; items: string[] };
    update: { id: string; name: string; items: string[] };
    delete: string;
    import: ChecklistTemplate;
  }>();

  export let templates: ChecklistTemplate[] = [];
  export let isOwner = true;
  export let showImport = false;

  let showAddForm = false;
  let editingTemplate: ChecklistTemplate | null = null;
  let newName = "";
  let itemsText = "";
  let deleteConfirmId: string | null = null;

  function startAdd() {
    showAddForm = true;
    editingTemplate = null;
    newName = "";
    itemsText = "";
  }

  function startEdit(template: ChecklistTemplate) {
    editingTemplate = template;
    showAddForm = true;
    newName = template.name;
    itemsText = template.items.join("\n");
  }

  function cancelEdit() {
    showAddForm = false;
    editingTemplate = null;
  }

  function handleSave() {
    if (!newName.trim()) return;

    const items = itemsText
      .split("\n")
      .map((i) => i.trim())
      .filter((i) => i.length > 0);

    if (editingTemplate) {
      dispatch("update", {
        id: editingTemplate.id,
        name: newName.trim(),
        items,
      });
    } else {
      dispatch("add", {
        name: newName.trim(),
        items,
      });
    }

    cancelEdit();
  }

  function handleDelete(id: string) {
    dispatch("delete", id);
    deleteConfirmId = null;
  }

  function handleImport(template: ChecklistTemplate) {
    dispatch("import", template);
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      dispatch("close");
    }
  }
</script>

<!-- Modal Backdrop -->
<div
  class="fixed inset-0 bg-black/50 z-20000 flex items-center justify-center p-4 backdrop-blur-sm"
  on:mousedown={handleBackdropClick}
  role="button"
  tabindex="-1"
  on:keydown={(e) => e.key === "Escape" && dispatch("close")}
>
  <!-- Modal Content -->
  <div
    class="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl w-full max-w-lg max-h-[90vh] flex flex-col transition-all overflow-hidden border border-gray-100 dark:border-gray-700"
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between p-6 border-b border-gray-100 dark:border-gray-700 bg-white/50 dark:bg-gray-800/50 backdrop-blur-md sticky top-0 z-10"
    >
      <div class="flex items-center gap-4">
        <div
          class="w-12 h-12 bg-primary/10 rounded-2xl flex items-center justify-center rotate-3 shadow-inner"
        >
          <ClipboardList size={24} class="text-primary -rotate-3" />
        </div>
        <div>
          <h2
            class="text-xl font-bold bg-linear-to-r from-gray-900 to-gray-600 dark:from-white dark:to-gray-400 bg-clip-text text-transparent"
          >
            {$_("checklistTemplate__title")}
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {templates.length}
            {$_("checklistTemplate__list_title")}
          </p>
        </div>
      </div>
      <button
        type="button"
        on:click={() => dispatch("close")}
        class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-xl transition-all active:scale-95"
      >
        <X size={20} />
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6 space-y-6">
      <!-- Add/Edit Form -->
      {#if showAddForm && isOwner}
        <div
          class="bg-gray-50 dark:bg-gray-900 border border-gray-100 dark:border-gray-700/50 rounded-3xl p-6 space-y-5 shadow-xl shadow-black/5 animate-in fade-in slide-in-from-top-4 duration-500"
        >
          <div class="space-y-2">
            <label
              for="template-name"
              class="block text-[10px] font-black text-primary uppercase tracking-[0.2em] ml-1"
            >
              {$_("checklistTemplate__name_label")}
            </label>
            <input
              id="template-name"
              type="text"
              bind:value={newName}
              placeholder={$_("checklistTemplate__name_placeholder")}
              class="w-full px-5 py-3.5 bg-white dark:bg-gray-800 border-2 border-gray-100 dark:border-gray-700 rounded-2xl focus:border-primary focus:ring-4 focus:ring-primary/10 outline-none text-gray-900 dark:text-white transition-all shadow-sm placeholder:text-gray-400 font-bold"
            />
          </div>

          <div class="space-y-2">
            <label
              for="template-items"
              class="block text-[10px] font-black text-primary uppercase tracking-[0.2em] ml-1"
            >
              {$_("checklistTemplate__items_label")}
            </label>
            <textarea
              id="template-items"
              bind:value={itemsText}
              rows="5"
              placeholder={$_("checklistTemplate__items_placeholder")}
              class="w-full px-5 py-4 bg-white dark:bg-gray-800 border-2 border-gray-100 dark:border-gray-700 rounded-2xl focus:border-primary focus:ring-4 focus:ring-primary/10 outline-none text-gray-900 dark:text-white transition-all shadow-sm resize-none font-bold text-sm placeholder:text-gray-400 leading-relaxed"
            ></textarea>
          </div>

          <div class="flex gap-3 pt-3">
            <button
              type="button"
              on:click={handleSave}
              disabled={!newName.trim()}
              class="flex-[2.5] bg-linear-to-r from-primary to-indigo-600 hover:from-primary-dark hover:to-indigo-700 disabled:from-gray-200 disabled:to-gray-300 dark:disabled:from-white/5 dark:disabled:to-white/10 text-white py-4 px-6 rounded-2xl font-black transition-all flex items-center justify-center gap-2 shadow-2xl shadow-primary/30 hover:shadow-primary/40 active:scale-[0.96] group/save cursor-pointer disabled:cursor-not-allowed disabled:shadow-none"
            >
              {#if editingTemplate}
                <Check
                  size={20}
                  class="group-hover/save:scale-125 transition-transform"
                />
                {$_("checklistTemplate__btn_save")}
              {:else}
                <Plus
                  size={20}
                  class="group-hover/save:rotate-90 transition-transform"
                />
                {$_("checklistTemplate__btn_add")}
              {/if}
            </button>
            <button
              type="button"
              on:click={cancelEdit}
              class="flex-1 px-6 py-4 border-2 border-gray-100 dark:border-gray-700 text-gray-500 dark:text-gray-400 rounded-2xl hover:bg-white dark:hover:bg-gray-700 hover:text-gray-900 dark:hover:text-white transition-all font-bold active:scale-[0.96] cursor-pointer"
            >
              {$_("common.cancel")}
            </button>
          </div>
        </div>
      {:else if isOwner}
        <!-- Add Button -->
        <button
          type="button"
          on:click={startAdd}
          class="w-full py-6 px-8 border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-3xl text-gray-500 dark:text-gray-400 hover:border-primary hover:text-primary hover:bg-primary/5 transition-all flex flex-col items-center justify-center gap-3 group active:scale-[0.98]"
        >
          <div
            class="w-12 h-12 rounded-2xl bg-gray-100 dark:bg-gray-800 flex items-center justify-center group-hover:bg-primary group-hover:text-white group-hover:rotate-12 group-hover:scale-110 transition-all shadow-inner"
          >
            <Plus size={24} />
          </div>
          <span class="font-black text-sm uppercase tracking-widest"
            >{$_("checklistTemplate__add_new")}</span
          >
        </button>
      {/if}

      <!-- Templates List -->
      <div class="space-y-4">
        {#if templates.length === 0}
          <div class="text-center py-12 px-6">
            <div
              class="w-16 h-16 bg-gray-50 dark:bg-gray-900 rounded-full flex items-center justify-center mx-auto mb-4 border border-gray-100 dark:border-gray-700"
            >
              <div
                class="h-1.5 w-full bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden mb-6"
              >
                <div
                  class="h-full bg-linear-to-r from-primary to-indigo-500 rounded-full w-full opacity-50"
                ></div>
              </div>
              <ClipboardList
                size={32}
                class="text-gray-300 dark:text-gray-600"
              />
            </div>
            <p class="font-bold text-gray-800 dark:text-white mb-1">
              {$_("checklistTemplate__no_templates")}
            </p>
            <p class="text-sm text-gray-500 dark:text-gray-400">
              {$_("checklistTemplate__add_hint")}
            </p>
          </div>
        {:else}
          {#each templates as template (template.id)}
            <div
              class="group relative bg-white dark:bg-gray-800/50 border border-gray-100 dark:border-gray-700 rounded-2xl p-4 hover:shadow-xl hover:shadow-primary/5 hover:border-primary/20 transition-all"
            >
              <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                  <h4
                    class="font-bold text-gray-900 dark:text-white mb-1 truncate flex items-center gap-2"
                  >
                    <ListChecks size={16} class="text-primary/60" />
                    {template.name}
                  </h4>
                  <div class="flex flex-wrap gap-1.5">
                    {#each template.items.slice(0, 3) as item}
                      <span
                        class="inline-flex items-center px-2 py-0.5 rounded-md bg-gray-100 dark:bg-gray-700 text-[10px] font-bold text-gray-500 dark:text-gray-400 border border-gray-200 dark:border-gray-600"
                      >
                        {item}
                      </span>
                    {/each}
                    {#if template.items.length > 3}
                      <span
                        class="text-[10px] font-bold text-gray-400 dark:text-gray-500 pt-0.5"
                      >
                        {$_("checklistTemplateModal__more_items", {
                          values: { count: template.items.length - 3 },
                        })}
                      </span>
                    {/if}
                  </div>
                </div>

                <div
                  class="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  {#if showImport}
                    <button
                      type="button"
                      on:click={() => handleImport(template)}
                      class="p-2 text-primary hover:bg-primary/10 rounded-xl transition-all active:scale-90"
                      title={$_("checklistTemplate__btn_import")}
                    >
                      <Download size={18} />
                    </button>
                  {/if}

                  {#if isOwner}
                    <button
                      type="button"
                      on:click={() => startEdit(template)}
                      class="p-2 text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-xl transition-all active:scale-90"
                      title={$_("common.update")}
                    >
                      <Edit2 size={16} />
                    </button>
                    <button
                      type="button"
                      on:click={() => (deleteConfirmId = template.id)}
                      class="p-2 text-gray-300 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-xl transition-all active:scale-90"
                      title={$_("workerManager__btn_delete")}
                    >
                      <Trash2 size={16} />
                    </button>
                  {/if}
                </div>
              </div>

              {#if deleteConfirmId === template.id}
                <div
                  class="mt-4 p-4 bg-red-50 dark:bg-red-500/10 border border-red-100 dark:border-red-500/20 rounded-xl animate-in fade-in zoom-in-95 duration-200"
                >
                  <p
                    class="text-xs font-bold text-red-600 dark:text-red-400 mb-3"
                  >
                    {$_("checklistTemplate__delete_confirm", {
                      values: { name: template.name },
                    })}
                  </p>
                  <div class="flex gap-2">
                    <button
                      type="button"
                      on:click={() => handleDelete(template.id)}
                      class="flex-1 bg-red-500 hover:bg-red-600 text-white py-1.5 px-3 rounded-lg text-xs font-bold transition-all shadow-sm"
                    >
                      {$_("workerManager__confirm_delete")}
                    </button>
                    <button
                      type="button"
                      on:click={() => (deleteConfirmId = null)}
                      class="flex-1 bg-white dark:bg-gray-700 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-gray-600 py-1.5 px-3 rounded-lg text-xs font-bold transition-all"
                    >
                      {$_("common.cancel")}
                    </button>
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <!-- Footer -->
    <div
      class="p-4 border-t border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 rounded-b-2xl"
    >
      <p
        class="text-[10px] font-bold text-gray-400 dark:text-gray-500 text-center uppercase tracking-widest"
      >
        {$_("checklistTemplate__footer_hint")}
      </p>
    </div>
  </div>
</div>

<style>
  .animate-in {
    animation-fill-mode: forwards;
  }
</style>
