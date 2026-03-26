<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import type { Assignee, AssigneeGroup } from "$lib/types";
  import {
    Users,
    Plus,
    X,
    Edit2,
    Trash2,
    User,
    Briefcase,
    Check,
    Link,
    RefreshCw,
    Mail,
    Palette,
    Pipette,
  } from "lucide-svelte";
  import { _ } from "svelte-i18n";
  import { api } from "$lib/apis";
  import {
    addAssigneeGroup,
    deleteAssigneeGroup,
    getAssigneeGroups,
    updateAssigneeGroup,
  } from "$lib/db";
  import SearchableSelect from "./SearchableSelect.svelte";
  import { fade, slide, scale } from "svelte/transition";

  const dispatch = createEventDispatcher<{
    close: void;
    add: { name: string; color: string; user_id?: string };
    update: {
      id: string | number;
      name: string;
      color: string;
      user_id?: string;
    };
    delete: string | number;
    groupsChanged: void;
  }>();

  export let assignees: Assignee[] = [];
  export let workerStats: { id: string | number; taskCount: number }[] = [];
  export let isLoading = false;
  export let isOwner = true;
  export let workspaceId = "";

  let assigneeGroups: AssigneeGroup[] = [];

  let showAddForm = false;
  let editingWorker: Assignee | null = null;
  let contentScrollEl: HTMLDivElement | null = null;
  let newWorkerName = "";
  let newWorkerColor = "#6366F1";
  let newWorkerUserId = ""; // For linking with system user
  let deleteConfirmId: string | number | null = null;

  let systemUsers: any[] = [];
  let loadingUsers = false;

  const colorOptions = [
    "#EF4444",
    "#F97316",
    "#F59E0B",
    "#FACC15",
    "#84CC16",
    "#22C55E",
    "#10B981",
    "#14B8A6",
    "#06B6D4",
    "#0EA5E9",
    "#3B82F6",
    "#6366F1",
    "#8B5CF6",
    "#A855F7",
    "#D946EF",
    "#EC4899",
    "#F43F5E",
    "#64748B",
    "#475569",
    "#7C3AED",
    "#DB2777",
    "#059669",
    "#2563EB",
  ];

  function hslToHex(h: number, s: number, l: number) {
    l /= 100;
    const a = (s * Math.min(l, 1 - l)) / 100;
    const f = (n: number) => {
      const k = (n + h / 30) % 12;
      const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
      return Math.round(255 * color)
        .toString(16)
        .padStart(2, "0");
    };
    return `#${f(0)}${f(8)}${f(4)}`.toUpperCase();
  }

  function getRandomColor() {
    // Generate a truly unique "special" color using HSL
    // Hue: 0-360, Saturation: 65-85%, Lightness: 45-55% (Premium look)
    const h = Math.floor(Math.random() * 360);
    const s = Math.floor(Math.random() * 20) + 65;
    const l = Math.floor(Math.random() * 10) + 45;
    return hslToHex(h, s, l);
  }

  let isShuffling = false;
  async function shuffleColor() {
    if (isShuffling) return;
    isShuffling = true;

    // Slot machine effect: rapid changes slowing down
    const totalDuration = 800;
    const initialInterval = 40;
    let elapsed = 0;
    let currentInterval = initialInterval;

    while (elapsed < totalDuration) {
      newWorkerColor = getRandomColor();
      await new Promise((r) => setTimeout(r, currentInterval));
      elapsed += currentInterval;
      // Gradually slow down
      currentInterval += (elapsed / totalDuration) * 20;
    }

    isShuffling = false;
  }

  onMount(async () => {
    await fetchSystemUsers();
    await refreshGroups();
  });

  async function refreshGroups() {
    try {
      assigneeGroups = await getAssigneeGroups(true);
    } catch (e) {
      console.error("Failed to load assignee groups:", e);
      assigneeGroups = [];
    }
  }

  async function fetchSystemUsers() {
    loadingUsers = true;
    try {
      const res = await api.auth.listUsers(workspaceId || undefined);
      if (res.ok) {
        const data = await res.json();
        systemUsers = data.users || [];
      }
    } catch (e) {
      console.error("Failed to fetch system users:", e);
    } finally {
      loadingUsers = false;
    }
  }

  function startAdd() {
    showAddForm = true;
    editingWorker = null;
    newWorkerName = "";
    newWorkerColor = getRandomColor();
    newWorkerUserId = "";
  }

  function startEdit(worker: Assignee) {
    editingWorker = worker;
    showAddForm = true;
    contentScrollEl?.scrollTo({ top: 0, behavior: "smooth" });
    newWorkerName = worker.name;
    newWorkerColor = worker.color || "#6366F1";
    newWorkerUserId = worker.user_id || "";
  }

  function cancelEdit() {
    showAddForm = false;
    editingWorker = null;
    newWorkerName = "";
    newWorkerColor = "#6366F1";
    newWorkerUserId = "";
  }

  function handleUserLinkChange(userId: string) {
    if (userId) {
      const user = systemUsers.find(
        (u) =>
          String(u.id) === String(userId) ||
          String(u.user_id) === String(userId),
      );
      if (user) {
        // Only auto-fill when adding a new worker, NOT when editing
        if (!editingWorker) {
          if (!newWorkerName) {
            newWorkerName =
              user.profile?.nickname || user.profile?.first_name || user.email.split("@")[0];
          }
        }
      }
    }
  }

  let previousUserId = newWorkerUserId;
  $: if (newWorkerUserId !== previousUserId) {
    previousUserId = newWorkerUserId;
    handleUserLinkChange(newWorkerUserId);
  }

  function handleSave() {
    if (!newWorkerName.trim()) return;

    if (editingWorker) {
      dispatch("update", {
        id: editingWorker.id!,
        name: newWorkerName.trim(),
        color: newWorkerColor,
        user_id: newWorkerUserId || undefined,
      });
    } else {
      dispatch("add", {
        name: newWorkerName.trim(),
        color: newWorkerColor,
        user_id: newWorkerUserId || undefined,
      });
    }

    cancelEdit();
  }

  function confirmDelete(id: string | number) {
    deleteConfirmId = id;
  }

  function handleDelete(id: string | number) {
    dispatch("delete", id);
    deleteConfirmId = null;
  }

  function getTaskCount(workerId: string | number): number {
    const stat = workerStats.find((s) => String(s.id) === String(workerId));
    return stat?.taskCount || 0;
  }

  function getLinkedUser(userId?: string) {
    if (!userId) return null;
    return systemUsers.find(
      (u) =>
        String(u.id) === String(userId) || String(u.user_id) === String(userId),
    );
  }

  $: usedUserIds = new Set(
    assignees
      .map((a) => a.user_id)
      .filter((id): id is string => !!id && id !== newWorkerUserId),
  );

  $: availableSystemUsers = systemUsers.filter(
    (u) =>
      !usedUserIds.has(String(u.id)) && !usedUserIds.has(String(u.user_id)),
  );

  // --- Assignee Group Management ---
  let showAddGroupForm = false;
  let newGroupName = '';
  let selectedGroupAssigneeIds: (string | number)[] = [];
  let deleteGroupConfirmId: string | number | null = null;
  let editingGroup: AssigneeGroup | null = null;
  let editGroupName = '';
  let editGroupAssigneeIds: (string | number)[] = [];

  const isSameId = (a: string | number | null | undefined, b: string | number | null | undefined) =>
    a !== null && a !== undefined && b !== null && b !== undefined && String(a) === String(b);

  // IDs of assignees already taken by other groups (excluding the group being edited)
  $: takenAssigneeIds = new Set(
    assigneeGroups
      .filter((g) => !editingGroup || !isSameId(g.id, editingGroup.id))
      .flatMap((g) => g.assignee_ids.map(String))
  );

  function isAssigneeTaken(id: string | number | undefined): boolean {
    if (id == null) return false;
    return takenAssigneeIds.has(String(id));
  }

  function toggleGroupMember(id: string | number | undefined) {
    if (id == null || isAssigneeTaken(id)) return;
    if (selectedGroupAssigneeIds.some((x) => isSameId(x, id))) {
      selectedGroupAssigneeIds = selectedGroupAssigneeIds.filter((x) => !isSameId(x, id));
      return;
    }
    selectedGroupAssigneeIds = [...selectedGroupAssigneeIds, id];
  }

  async function handleCreateGroup() {
    if (!newGroupName.trim()) return;
    try {
      await addAssigneeGroup({
        name: newGroupName.trim(),
        assignee_ids: selectedGroupAssigneeIds,
      });
      newGroupName = '';
      selectedGroupAssigneeIds = [];
      showAddGroupForm = false;
      await refreshGroups();
      dispatch('groupsChanged');
    } catch (error) {
      console.error("Failed to create assignee group:", error);
    }
  }

  function cancelAddGroup() {
    newGroupName = '';
    selectedGroupAssigneeIds = [];
    showAddGroupForm = false;
  }

  function startEditGroup(group: AssigneeGroup) {
    editingGroup = group;
    editGroupName = group.name;
    editGroupAssigneeIds = [...group.assignee_ids];
    // Close other forms
    showAddGroupForm = false;
    deleteGroupConfirmId = null;
  }

  function cancelEditGroup() {
    editingGroup = null;
    editGroupName = '';
    editGroupAssigneeIds = [];
  }

  function toggleEditGroupMember(id: string | number | undefined) {
    if (id == null || isAssigneeTaken(id)) return;
    if (editGroupAssigneeIds.some((x) => isSameId(x, id))) {
      editGroupAssigneeIds = editGroupAssigneeIds.filter((x) => !isSameId(x, id));
      return;
    }
    editGroupAssigneeIds = [...editGroupAssigneeIds, id];
  }

  async function handleUpdateGroup() {
    if (!editingGroup?.id || !editGroupName.trim()) return;
    try {
      await updateAssigneeGroup(editingGroup.id, {
        name: editGroupName.trim(),
        assignee_ids: editGroupAssigneeIds,
      });
      cancelEditGroup();
      await refreshGroups();
      dispatch('groupsChanged');
    } catch (error) {
      console.error("Failed to update assignee group:", error);
    }
  }

  async function handleDeleteGroup(groupId: string | number | undefined) {
    if (groupId == null) return;
    try {
      await deleteAssigneeGroup(groupId);
      deleteGroupConfirmId = null;
      await refreshGroups();
      dispatch('groupsChanged');
    } catch (error) {
      console.error("Failed to delete assignee group:", error);
    }
  }

  function getGroupMembers(group: AssigneeGroup): Assignee[] {
    return group.assignee_ids
      .map((id) => assignees.find((a) => isSameId(a.id, id)))
      .filter(Boolean) as Assignee[];
  }

  // Shake animation when clicking backdrop
  let isShaking = false;

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      // Trigger shake animation
      isShaking = true;
      setTimeout(() => {
        isShaking = false;
      }, 500);
    }
  }
</script>

<style>
  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    10%, 30%, 50%, 70%, 90% { transform: translateX(-8px); }
    20%, 40%, 60%, 80% { transform: translateX(8px); }
  }

  .shake {
    animation: shake 0.1s ease-in-out;
  }
</style>

<!-- Modal Backdrop -->
<div
  class="fixed inset-0 bg-black/50 z-[20000] flex items-center justify-center p-4"
  on:click={handleBackdropClick}
  on:keydown={(e) => e.key === "Escape" && dispatch("close")}
  role="button"
  tabindex="-1"
>
  <!-- Modal Content -->
  <div
    class:shake={isShaking}
    class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl w-full max-w-lg max-h-[90vh] flex flex-col transition-colors"
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between p-6 border-b border-gray-100 dark:border-gray-700"
    >
      <div class="flex items-center gap-3">
        <div
          class="w-10 h-10 bg-primary/10 rounded-xl flex items-center justify-center"
        >
          <Users size={20} class="text-primary" />
        </div>
        <div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-white">
            {$_("workerManager__title")}
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {assignees.length}
            {$_("workerManager__count_suffix")}
          </p>
        </div>
      </div>
      <button
        on:click={() => dispatch("close")}
        class="p-2 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
      >
        <X size={20} />
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6" bind:this={contentScrollEl}>
      <!-- Add/Edit Form -->
      {#if showAddForm && isOwner}
        <div
          transition:slide={{ duration: 300 }}
          class="bg-gray-50 dark:bg-gray-700/50 rounded-2xl p-5 mb-6 space-y-5 border border-gray-100 dark:border-gray-600 shadow-inner"
        >
          <!-- Link with System User -->
          <div>
            <label
              for="link-user-select"
              class="block text-xs font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-2"
            >
              {$_("workerManager__link_user_label")}
            </label>
            <div class="relative z-10 block">
              <SearchableSelect
                id="link-user-select"
                bind:value={newWorkerUserId}
                options={[
                  {
                    value: "",
                    label:
                      "-- " +
                      $_("workerManager__link_user_placeholder") +
                      " --",
                  },
                  ...availableSystemUsers.map((user) => ({
                    value: String(user.id || user.user_id || ""),
                    label: `${user.profile?.nickname || user.profile?.first_name || user.email.split("@")[0]} (${user.email})`,
                    badge: user.is_active !== undefined,
                    badgeColor: user.is_active ? "#22C55E" : "#FBBF24",
                  })),
                ]}
                placeholder={$_("workerManager__link_user_placeholder")}
              />
            </div>
            {#if newWorkerUserId}
              <p
                class="text-[10px] text-green-500 mt-1.5 flex items-center gap-1 font-medium"
              >
                <Check size={10} />
                {$_("workerManager__auto_fill_hint")}
              </p>
            {/if}
          </div>

          <div>
            <div>
              <label
                for="worker-name-input"
                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >
                {editingWorker
                  ? $_("workerManager__edit_name")
                  : $_("workerManager__name_label")}
              </label>
              <input
                id="worker-name-input"
                type="text"
                bind:value={newWorkerName}
                placeholder={$_("workerManager__name_placeholder")}
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none"
              />
            </div>
          </div>

          <div>
            <div class="flex items-center justify-between mb-2">
              <label
                for="worker-color-picker"
                class="block text-sm font-medium text-gray-700 dark:text-gray-300"
              >
                {$_("workerManager__color_label")}
              </label>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  on:click={shuffleColor}
                  disabled={isShuffling}
                  class="text-[10px] font-bold uppercase tracking-wider text-primary hover:text-primary-dark flex items-center gap-1 transition-colors disabled:opacity-50"
                >
                  <RefreshCw
                    size={10}
                    class={isShuffling ? "animate-spin" : ""}
                  />
                  {$_("workerManager__random_color_btn")}
                </button>
              </div>
            </div>

            <div class="grid grid-cols-8 gap-2 mb-3">
              {#each colorOptions as color}
                <button
                  type="button"
                  on:click={() => (newWorkerColor = color)}
                  aria-label={`${$_("workerManager__select_color")} ${color}`}
                  class="aspect-square rounded-full border-2 transition-all flex items-center justify-center hover:scale-110 active:scale-95 shadow-sm"
                  class:border-white={newWorkerColor === color}
                  class:ring-2={newWorkerColor === color}
                  class:ring-primary={newWorkerColor === color}
                  class:border-transparent={newWorkerColor !== color}
                  style="background-color: {color}"
                >
                  {#if newWorkerColor === color}
                    <div
                      class="w-1.5 h-1.5 bg-white rounded-full shadow-sm"
                    ></div>
                  {/if}
                </button>
              {/each}

              <!-- Custom Color Picker -->
              <div class="relative group aspect-square">
                <input
                  type="color"
                  bind:value={newWorkerColor}
                  class="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                />
                <div
                  class="absolute inset-0 rounded-full border-2 border-dashed border-gray-300 dark:border-gray-600 flex items-center justify-center text-gray-400 dark:text-gray-500 group-hover:border-primary group-hover:text-primary transition-all bg-white dark:bg-gray-800"
                  style={!colorOptions.includes(newWorkerColor)
                    ? `background-color: ${newWorkerColor}; border-style: solid; border-color: white; color: white;`
                    : ""}
                >
                  {#if !colorOptions.includes(newWorkerColor)}
                    <Pipette size={12} />
                  {:else}
                    <Plus size={12} />
                  {/if}
                </div>
              </div>
            </div>
          </div>

          <div class="flex gap-2 pt-2">
            <button
              on:click={handleSave}
              disabled={!newWorkerName.trim()}
              class="flex-1 bg-primary hover:bg-primary-dark disabled:bg-gray-300 dark:disabled:bg-gray-600 text-white py-2 px-4 rounded-lg font-bold transition-all flex items-center justify-center gap-2 active:scale-95 shadow-sm"
            >
              {#if editingWorker}
                <Check size={16} />
                {$_("workerManager__btn_save")}
              {:else}
                <Plus size={16} />
                {$_("workerManager__btn_add")}
              {/if}
            </button>
            <button
              on:click={cancelEdit}
              class="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
            >
              {$_("workerManager__btn_cancel")}
            </button>
          </div>
        </div>
      {:else if isOwner}
        <!-- Add Button -->
        <button
          on:click={startAdd}
          class="w-full mb-6 py-4 border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-2xl text-gray-500 dark:text-gray-400 hover:border-primary hover:text-primary hover:bg-primary/5 transition-all flex flex-col items-center justify-center gap-1 group active:scale-[0.98]"
        >
          <div
            class="p-1 bg-gray-100 dark:bg-gray-800 rounded-md group-hover:bg-primary/10 transition-colors"
          >
            <Plus size={20} />
          </div>
          <span class="text-sm font-bold uppercase tracking-wider"
            >{$_("workerManager__add_new")}</span
          >
        </button>
      {/if}

      <!-- Worker List -->
      <div class="space-y-3">
        <div class="flex items-center justify-between mb-2">
          <h3
            class="text-[10px] font-black text-gray-400 dark:text-gray-500 uppercase tracking-[0.2em]"
          >
            {$_("workerManager__list_title")}
          </h3>
          <button
            on:click={fetchSystemUsers}
            class="text-gray-400 hover:text-primary transition-colors"
            title="Reload System Users"
          >
            <RefreshCw size={12} class={loadingUsers ? "animate-spin" : ""} />
          </button>
        </div>

        {#if isLoading}
          <div
            class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400 mb-3"
          >
            <RefreshCw size={12} class="animate-spin" />
            <span>Loading assignees...</span>
          </div>
        {/if}

        {#if !isLoading && assignees.length === 0}
          <div
            class="text-center py-12 bg-gray-50 dark:bg-gray-900/50 rounded-2xl border border-gray-100 dark:border-gray-800"
          >
            <div
              class="w-16 h-16 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center mx-auto mb-4 text-gray-400"
            >
              <User size={32} />
            </div>
            <p class="font-bold text-gray-500 dark:text-gray-400">
              {$_("workerManager__no_workers")}
            </p>
            <p class="text-xs text-gray-400 dark:text-gray-500">
              {$_("workerManager__add_hint")}
            </p>
          </div>
        {:else}
          {#each assignees as worker (worker.id)}
            {@const linkedUser = getLinkedUser(worker.user_id)}
            {@const isLinked = !!worker.user_id || !!linkedUser}
            {@const displayEmail = worker.email || linkedUser?.email}
            <div
              class="flex items-center gap-3 p-3 bg-white dark:bg-gray-800/50 border border-gray-100 dark:border-gray-700/50 rounded-2xl group hover:border-primary/30 hover:bg-gray-50 dark:hover:bg-white/5 transition-all shadow-sm"
            >
              <!-- Color Avatar -->
              <div
                class="w-12 h-12 rounded-xl flex items-center justify-center text-white font-black text-lg shrink-0 shadow-sm"
                style="background-color: {worker.color || '#6366F1'}"
              >
                {worker.name.charAt(0)}
              </div>

              <!-- Info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <h4 class="font-bold text-gray-900 dark:text-white truncate">
                    {worker.name}
                  </h4>
                  {#if isLinked}
                    <div
                      class="px-1.5 py-0.5 bg-green-500/10 text-green-500 rounded text-[9px] font-black uppercase tracking-tighter flex items-center gap-0.5"
                      title={`${$_("workerManager__linked_status")}${displayEmail ? `: ${displayEmail}` : ""}`}
                    >
                      <Link size={8} />
                      {$_("workerManager__linked_status")}
                    </div>
                  {/if}
                </div>
                <div class="flex items-center gap-3 mt-0.5">
                  {#if displayEmail}
                    <div
                      class="flex items-center gap-1 text-[11px] text-gray-500 dark:text-gray-400 font-medium truncate"
                    >
                      <Mail size={10} />
                      <span class="truncate" title={displayEmail}
                        >{displayEmail}</span
                      >
                    </div>
                  {/if}
                  <div
                    class="flex items-center gap-1 text-[11px] text-gray-500 dark:text-gray-400 font-medium"
                  >
                    <Briefcase size={10} />
                    <span
                      >{getTaskCount(worker.id!)}
                      {$_("workerManager__task_count_suffix")}</span
                    >
                  </div>
                </div>
              </div>

              <!-- Actions -->
              {#if isOwner}
                <div
                  class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all"
                >
                  <button
                    on:click={() => startEdit(worker)}
                    class="p-2 text-gray-400 dark:text-gray-500 hover:text-primary hover:bg-white dark:hover:bg-gray-700 rounded-lg transition-colors border border-transparent hover:border-gray-100 dark:hover:border-gray-600 shadow-sm"
                    title={$_("workerManager__btn_edit")}
                  >
                    <Edit2 size={14} />
                  </button>
                  <button
                    on:click={() => confirmDelete(worker.id!)}
                    class="p-2 text-gray-400 dark:text-gray-500 hover:text-red-500 hover:bg-white dark:hover:bg-gray-700 rounded-lg transition-colors border border-transparent hover:border-gray-100 dark:hover:border-gray-600 shadow-sm"
                    title={$_("workerManager__btn_delete")}
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              {/if}
            </div>

            <!-- Delete Confirmation -->
            {#if deleteConfirmId === worker.id}
              <div
                class="bg-red-50 dark:bg-red-500/10 border border-red-100 dark:border-red-500/20 rounded-2xl p-4 mt-2 mb-4 animate-in fade-in slide-in-from-top-2"
              >
                <p
                  class="text-sm text-red-600 dark:text-red-400 mb-3 font-medium"
                >
                  {$_("workerManager__delete_confirm", {
                    values: { name: worker.name },
                  })}
                  {#if getTaskCount(worker.id!) > 0}
                    <span class="block text-xs mt-1 opacity-80"
                      >{$_("workerManager__delete_warning")}</span
                    >
                  {/if}
                </p>
                <div class="flex gap-2">
                  <button
                    on:click={() => handleDelete(worker.id!)}
                    class="px-4 py-1.5 bg-red-500 text-white rounded-lg text-sm font-bold hover:bg-red-600 transition-all shadow-sm active:scale-95"
                  >
                    {$_("workerManager__confirm_delete")}
                  </button>
                  <button
                    on:click={() => (deleteConfirmId = null)}
                    class="px-4 py-1.5 bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 border border-gray-200 dark:border-gray-700 rounded-lg text-sm font-bold hover:bg-gray-50 transition-colors shadow-sm"
                  >
                    {$_("workerManager__btn_cancel")}
                  </button>
                </div>
              </div>
            {/if}
          {/each}
        {/if}
      </div>

      <!-- Assignee Groups Section -->
      <div class="mt-8 space-y-3">
        <div class="flex items-center justify-between mb-2">
          <h3
            class="text-[10px] font-black text-gray-400 dark:text-gray-500 uppercase tracking-[0.2em]"
          >
            <span class="flex items-center gap-1"><Users size={10} /> {$_("workerManager__groups_title")}</span>
          </h3>
        </div>

        {#if isOwner}
          {#if showAddGroupForm}
            <div
              transition:slide={{ duration: 300 }}
              class="bg-gray-50 dark:bg-gray-700/50 rounded-2xl p-4 space-y-3 border border-gray-100 dark:border-gray-600 shadow-inner"
            >
              <input
                type="text"
                bind:value={newGroupName}
                placeholder={$_("workerManager__groups_name_placeholder")}
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none text-sm dark:bg-gray-700 dark:text-white"
              />
              <div class="max-h-36 overflow-y-auto space-y-1">
                {#each assignees.filter((a) => a.id != null) as assignee (assignee.id)}
                  {@const taken = isAssigneeTaken(assignee.id)}
                  <label class="flex items-center gap-2 text-sm p-1 rounded cursor-pointer {taken ? 'text-gray-400 dark:text-gray-500 opacity-50 cursor-not-allowed' : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-600'}">
                    <input
                      type="checkbox"
                      checked={selectedGroupAssigneeIds.some((id) => isSameId(id, assignee.id))}
                      on:change={() => toggleGroupMember(assignee.id)}
                      disabled={taken}
                      class="accent-primary"
                    />
                    <span class="w-3 h-3 rounded-full shrink-0" style="background-color: {assignee.color}"></span>
                    <span class="truncate">{assignee.name}</span>
                    {#if taken}
                      <span class="text-[10px] text-gray-400 dark:text-gray-500 ml-auto whitespace-nowrap">{$_("workerManager__groups_member_taken")}</span>
                    {/if}
                  </label>
                {/each}
              </div>
              <div class="flex gap-2">
                <button
                  type="button"
                  on:click={handleCreateGroup}
                  disabled={!newGroupName.trim() || selectedGroupAssigneeIds.length === 0}
                  class="flex-1 bg-primary hover:bg-primary-dark disabled:bg-gray-300 dark:disabled:bg-gray-600 text-white py-2 px-4 rounded-lg text-sm font-bold transition-all flex items-center justify-center gap-2 active:scale-95 shadow-sm"
                >
                  <Plus size={14} />
                  {$_("workerManager__groups_create_btn")}
                </button>
                <button
                  type="button"
                  on:click={cancelAddGroup}
                  class="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg text-sm hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                >
                  {$_("workerManager__btn_cancel")}
                </button>
              </div>
            </div>
          {:else}
            <button
              on:click={() => showAddGroupForm = true}
              class="w-full py-3 border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-2xl text-gray-500 dark:text-gray-400 hover:border-primary hover:text-primary hover:bg-primary/5 transition-all flex items-center justify-center gap-2 group active:scale-[0.98] text-sm"
            >
              <Users size={14} />
              <Plus size={12} />
              <span class="font-bold uppercase tracking-wider text-xs">{$_("workerManager__groups_add_btn_short")}</span>
            </button>
          {/if}
        {/if}

        {#if assigneeGroups.length === 0 && !showAddGroupForm}
          <div class="text-center py-6 bg-gray-50 dark:bg-gray-900/50 rounded-2xl border border-gray-100 dark:border-gray-800">
            <p class="text-sm text-gray-400 dark:text-gray-500">{$_("workerManager__groups_no_groups")}</p>
          </div>
        {:else}
          {#each assigneeGroups as group (group.id)}
            {@const members = getGroupMembers(group)}
            {#if editingGroup && isSameId(editingGroup.id, group.id)}
              <!-- Edit Group Form -->
              <div
                transition:slide={{ duration: 300 }}
                class="bg-gray-50 dark:bg-gray-700/50 rounded-2xl p-4 space-y-3 border border-primary/30 shadow-inner"
              >
                <input
                  type="text"
                  bind:value={editGroupName}
                  placeholder={$_("workerManager__groups_name_placeholder")}
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none text-sm dark:bg-gray-700 dark:text-white"
                />
                <div class="max-h-36 overflow-y-auto space-y-1">
                  {#each assignees.filter((a) => a.id != null) as assignee (assignee.id)}
                    {@const taken = isAssigneeTaken(assignee.id)}
                    <label class="flex items-center gap-2 text-sm p-1 rounded cursor-pointer {taken ? 'text-gray-400 dark:text-gray-500 opacity-50 cursor-not-allowed' : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-600'}">
                      <input
                        type="checkbox"
                        checked={editGroupAssigneeIds.some((id) => isSameId(id, assignee.id))}
                        on:change={() => toggleEditGroupMember(assignee.id)}
                        disabled={taken}
                        class="accent-primary"
                      />
                      <span class="w-3 h-3 rounded-full shrink-0" style="background-color: {assignee.color}"></span>
                      <span class="truncate">{assignee.name}</span>
                      {#if taken}
                        <span class="text-[10px] text-gray-400 dark:text-gray-500 ml-auto whitespace-nowrap">{$_("workerManager__groups_member_taken")}</span>
                      {/if}
                    </label>
                  {/each}
                </div>
                <div class="flex gap-2">
                  <button
                    type="button"
                    on:click={handleUpdateGroup}
                    disabled={!editGroupName.trim()}
                    class="flex-1 bg-primary hover:bg-primary-dark disabled:bg-gray-300 dark:disabled:bg-gray-600 text-white py-2 px-4 rounded-lg text-sm font-bold transition-all flex items-center justify-center gap-2 active:scale-95 shadow-sm"
                  >
                    <Check size={14} />
                    {$_("workerManager__groups_save_btn")}
                  </button>
                  <button
                    type="button"
                    on:click={cancelEditGroup}
                    class="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg text-sm hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                  >
                    {$_("workerManager__btn_cancel")}
                  </button>
                </div>
              </div>
            {:else}
              <!-- Group Card -->
              <div class="bg-white dark:bg-gray-800/50 border border-gray-100 dark:border-gray-700/50 rounded-2xl p-3 group/grp hover:border-primary/30 transition-all shadow-sm">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2 min-w-0">
                    <div class="w-8 h-8 bg-primary/10 rounded-lg flex items-center justify-center shrink-0">
                      <Users size={14} class="text-primary" />
                    </div>
                    <div class="min-w-0">
                      <h4 class="font-bold text-sm text-gray-900 dark:text-white truncate">{group.name}</h4>
                      <p class="text-[11px] text-gray-500 dark:text-gray-400">{members.length} {$_("workerManager__groups_members")}</p>
                    </div>
                  </div>
                  {#if isOwner}
                    {#if deleteGroupConfirmId === group.id}
                      <div class="flex items-center gap-1">
                        <button
                          on:click={() => handleDeleteGroup(group.id)}
                          class="px-2 py-1 bg-red-500 text-white rounded text-xs font-bold hover:bg-red-600 transition-all"
                        >
                          {$_("workerManager__groups_delete_btn")}
                        </button>
                        <button
                          on:click={() => deleteGroupConfirmId = null}
                          class="px-2 py-1 bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded text-xs font-bold hover:bg-gray-300 transition-colors"
                        >
                          {$_("workerManager__btn_cancel")}
                        </button>
                      </div>
                    {:else}
                      <div class="flex items-center gap-1 opacity-0 group-hover/grp:opacity-100 transition-all">
                        <button
                          on:click={() => startEditGroup(group)}
                          class="p-1.5 text-gray-400 dark:text-gray-500 hover:text-primary hover:bg-white dark:hover:bg-gray-700 rounded-lg transition-colors"
                          title={$_("workerManager__groups_edit_title")}
                        >
                          <Edit2 size={13} />
                        </button>
                        <button
                          on:click={() => deleteGroupConfirmId = group.id ?? null}
                          class="p-1.5 text-gray-400 dark:text-gray-500 hover:text-red-500 hover:bg-white dark:hover:bg-gray-700 rounded-lg transition-colors"
                          title={$_("workerManager__groups_delete_title")}
                        >
                          <Trash2 size={13} />
                        </button>
                      </div>
                    {/if}
                  {/if}
                </div>
                {#if members.length > 0}
                  <div class="flex flex-wrap gap-1 mt-2">
                    {#each members as member (member.id)}
                      <span class="inline-flex items-center gap-1 px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded-full text-[11px] text-gray-600 dark:text-gray-300">
                        <span class="w-2 h-2 rounded-full" style="background-color: {member.color}"></span>
                        {member.name}
                      </span>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}
          {/each}
        {/if}
      </div>
    </div>

    <!-- Footer -->
    <div
      class="p-4 border-t border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 rounded-b-2xl"
    >
      <p
        class="text-[10px] text-gray-400 dark:text-gray-500 text-center font-bold uppercase tracking-widest"
      >
        {$_("workerManager__footer_hint")}
      </p>
    </div>
  </div>
</div>
