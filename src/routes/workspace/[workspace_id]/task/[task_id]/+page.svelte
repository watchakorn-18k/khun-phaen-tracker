<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { _ } from "svelte-i18n";
  import { 
    getTaskById, 
    getTaskComments, 
    createTaskComment, 
    updateTask, 
    getProjectsList, 
    getAssignees,
    getSprints,
    deleteTask
  } from "$lib/db";
  import type { Task, TaskComment, Assignee, Project, Sprint, ChecklistItem } from "$lib/types";
  import { user } from "$lib/stores/auth";
  import { currentWorkspaceName } from "$lib/stores/workspace";
  import { 
    MoreHorizontal, 
    PanelRight, 
    CircleCheck, 
    Pin,
    PinOff,
    Calendar,
    Clock,
    User,
    Folder,
    Tag,
    MessageCircle,
    Send,
    Plus,
    ChevronDown,
    ChevronRight,
    ArrowLeft,
    CheckCircle2,
    Layout,
    Circle,
    CircleDashed,
    CircleDot,
    XCircle,
    SignalHigh,
    SignalMedium,
    SignalLow,
    Ban,
    GitBranch,
    Trash2,
    AlertTriangle
  } from "lucide-svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import BranchDialog from "$lib/components/BranchDialog.svelte";
  import RichTextEditor from "$lib/components/editor/RichTextEditor.svelte";
  import TitleEditor from "$lib/components/editor/TitleEditor.svelte";
  import ChecklistManager from "$lib/components/ChecklistManager.svelte";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";
  import PrioritySelector from "$lib/components/PrioritySelector.svelte";
  import CustomDatePicker from "$lib/components/CustomDatePicker.svelte";
  import { showMessage } from "$lib/stores/uiActions";

  let taskId = $page.params.task_id;
  let workspaceId = $page.params.workspace_id;
  let task: Task | null = null;
  let loading = true;
  let error = "";
  
  let comments: TaskComment[] = [];
  let commentsLoading = false;
  let commentContent = "";
  
  let sidebarOpen = true;
  let isEditingTitle = false;
  let editedTitle = "";
  let titleInputRef: any;

  let projects: Project[] = [];
  let assignees: Assignee[] = [];
  let sprints: Sprint[] = [];

  let editedDescription = "";
  $: isDescriptionDirty = task && editedDescription !== (task.notes || "");

  let isBranchDialogOpen = false;

  function openBranchDialog() {
    isBranchDialogOpen = true;
  }

  function closeBranchDialog() {
    isBranchDialogOpen = false;
  }

  let isMoreMenuOpen = false;
  let isDeleteConfirm = false;

  function toggleMoreMenu() {
    isMoreMenuOpen = !isMoreMenuOpen;
    if (!isMoreMenuOpen) isDeleteConfirm = false;
  }

  function handleMoreMenuClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.more-menu-container')) {
      isMoreMenuOpen = false;
      isDeleteConfirm = false;
    }
  }

  async function handleDeleteTask() {
    if (!task || task.id == null) return;
    try {
      await deleteTask(task.id);
      goBack();
    } catch (e) {
      console.error('Delete task failed', e);
    }
  }

  $: workspaceRoom = $page.url.searchParams.get("room");

  let isPinned = false;

  onMount(async () => {
    const pinnedTasks = JSON.parse(localStorage.getItem("pinned-tasks") || "[]");
    isPinned = !!pinnedTasks.find((t: any) => t.id === taskId);

    await Promise.all([
      loadTaskData(),
      loadMetadata()
    ]);
  });

  function togglePin() {
    isPinned = !isPinned;
    let pinnedTasks: any[] = JSON.parse(localStorage.getItem("pinned-tasks") || "[]");
    
    if (isPinned) {
      if (!pinnedTasks.find(t => t.id === taskId)) {
        // Add to the top
        pinnedTasks.unshift({
          id: taskId,
          title: task?.title || "",
          short_name: task?.workspace_short_name || "",
          task_number: task?.task_number || "",
          workspace_id: workspaceId || task?.workspace_id || ""
        });
        // Limit to 5
        if (pinnedTasks.length > 5) {
          pinnedTasks = pinnedTasks.slice(0, 5);
        }
      }
    } else {
      pinnedTasks = pinnedTasks.filter(t => t.id !== taskId);
    }
    localStorage.setItem("pinned-tasks", JSON.stringify(pinnedTasks));
    window.dispatchEvent(new CustomEvent('pinnedTasksChanged'));
  }

  async function loadTaskData() {
    loading = true;
    error = "";
    try {
      if (!taskId) return;
      const result = await getTaskById(taskId);
      if (result) {
        task = result;
        editedTitle = task.title;
        editedDescription = task.notes || "";
        await loadComments();
      } else {
        error = $_("taskList__no_tasks");
      }
    } catch (e) {
      error = $_("page__load_data_error");
    } finally {
      loading = false;
    }
  }

  async function loadMetadata() {
    try {
      const [projList, assList, sprintList] = await Promise.all([
        getProjectsList(),
        getAssignees(true),
        getSprints()
      ]);
      projects = projList;
      assignees = assList;
      sprints = sprintList;
    } catch (e) {
      console.error("Failed to load metadata:", e);
    }
  }

  async function loadComments() {
    if (!task?.id) return;
    commentsLoading = true;
    try {
      const response = await getTaskComments(task.id, { page: 1, limit: 50 });
      comments = response.comments;
    } catch (e) {
      console.error("Failed to load comments:", e);
    } finally {
      commentsLoading = false;
    }
  }

  async function handleUpdateTask(updates: Partial<Task>) {
    if (!task?.id) return;
    try {
      // Explicitly include workspace_id to ensure correct API routing
      const updated = await updateTask(task.id, { 
        ...updates, 
        workspace_id: workspaceId 
      });
      
      if (updated) {
        task = { ...task, ...updated };
        showMessage($_("page__update_task_success"), "success");
      }
    } catch (e) {
      console.error("Failed to update task:", e);
      showMessage($_("page__error"), "error");
    }
  }

  async function saveTitle() {
    if (!task || !editedTitle.trim() || editedTitle === task.title) {
      isEditingTitle = false;
      editedTitle = task?.title || "";
      return;
    }
    await handleUpdateTask({ title: editedTitle.trim() });
    isEditingTitle = false;
  }

  async function handleCommentSubmit() {
    if (!task?.id || !commentContent.trim()) return;
    try {
      await createTaskComment(task.id, { content: commentContent.trim() });
      commentContent = "";
      await loadComments();
    } catch (e) {
      showMessage($_("taskForm__comments_error_submit"), "error");
    }
  }

  function formatRelativeTime(dateStr?: string) {
    if (!dateStr) return "";
    const date = new Date(dateStr);
    const diffMs = Date.now() - date.getTime();
    const diffMin = Math.floor(diffMs / 60000);
    if (diffMin < 1) return $_("taskForm__comments_time_just_now");
    if (diffMin < 60) return $_("taskForm__comments_time_min_ago", { values: { count: diffMin } });
    const diffHr = Math.floor(diffMin / 60);
    if (diffHr < 24) return $_("taskForm__comments_time_hour_ago", { values: { count: diffHr } });
    const diffDay = Math.floor(diffHr / 24);
    if (diffDay < 7) return $_("taskForm__comments_time_day_ago", { values: { count: diffDay } });
    return date.toLocaleDateString();
  }

  function goBack() {
    const roomParam = workspaceRoom ? `?room=${workspaceRoom}` : "";
    goto(`${base}/workspace/${workspaceId}${roomParam}`);
  }

  const statusOptions = [
    { value: "pending", label: $_("taskForm__status_pending"), icon: CircleDashed, iconClass: "text-gray-500" },
    { value: "todo", label: $_("taskForm__status_todo"), icon: Circle, iconClass: "text-gray-400" },
    { value: "in-progress", label: $_("taskForm__status_in_progress"), icon: CircleDot, iconClass: "text-yellow-500" },
    { value: "in-test", label: $_("taskForm__status_in_test"), icon: CircleDot, iconClass: "text-green-500" },
    { value: "done", label: $_("taskForm__status_done"), icon: CheckCircle2, iconClass: "text-blue-500" }
  ];

  function getStatusLabel(status: string) {
    return statusOptions.find(o => o.value === status)?.label || status;
  }

  function getStatusColor(status: string) {
    switch (status) {
      case "pending": return "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300";
      case "todo": return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400";
      case "in-progress": return "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400";
      case "in-test": return "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400";
      case "done": return "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400";
      default: return "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300";
    }
  }

  function handleChecklistUpdate(event: CustomEvent<{ checklist: ChecklistItem[] }>) {
    void handleUpdateTask({ checklist: event.detail.checklist });
  }

  function handleDescriptionChange(value: string) {
    editedDescription = value;
  }

  async function saveDescription() {
    if (!task) return;
    await handleUpdateTask({ notes: editedDescription });
  }

  function cancelDescription() {
    if (task) {
      editedDescription = task.notes || "";
    }
  }
</script>

<svelte:window on:click={handleMoreMenuClickOutside} />

<div class="flex flex-col h-full bg-white dark:bg-[#0b1120] text-slate-900 dark:text-slate-100">
  <!-- Header -->
  <header class="flex items-center justify-between px-4 h-14 border-b border-slate-200 dark:border-slate-800 shrink-0 bg-white/80 dark:bg-[#0b1120]/80 backdrop-blur-md sticky top-0 z-10">
    <div class="flex items-center gap-3 overflow-hidden">
      <button 
        on:click={goBack}
        class="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors shrink-0"
        title={$_("sidebar__back")}
      >
        <ArrowLeft size={18} />
      </button>
      <div class="flex items-center gap-1.5 text-sm text-slate-500 dark:text-slate-400 truncate">
        <span class="truncate hover:text-slate-900 dark:hover:text-white cursor-pointer flex items-center gap-1" 
          on:click={goBack}
          on:keydown={(e) => e.key === 'Enter' && goBack()}
          role="link"
          tabindex="0"
        >
          <Layout size={14} /> {$currentWorkspaceName || 'Workspace'}
        </span>
        <ChevronRight size={14} class="shrink-0" />
        {#if task}
          <span class="font-medium text-slate-900 dark:text-white truncate">
            {task.title}
          </span>
        {/if}
      </div>
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <button 
        class="p-2 rounded-lg transition-all duration-200 {task?.status === 'done' ? 'text-emerald-500/40 dark:text-emerald-400/30 cursor-default' : 'text-slate-500 dark:text-slate-400 hover:text-emerald-500 hover:bg-emerald-500/10'}"
        on:click={() => task && task.status !== 'done' && handleUpdateTask({ status: 'done' })}
        disabled={task?.status === 'done'}
        title={task?.status === 'done' ? $_("taskForm__status_done") : "Mark as done"}
      >
        <CheckCircle2 size={18} />
      </button>
      <button 
        on:click={openBranchDialog}
        class="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors text-slate-500 dark:text-slate-400"
        title={$_("taskForm__branch")}
      >
        <GitBranch size={18} />
      </button>
      <button 
        on:click={togglePin}
        class="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors {isPinned ? 'text-indigo-500 bg-indigo-50 dark:bg-indigo-500/10' : 'text-slate-500 dark:text-slate-400'}"
        title={isPinned ? 'Unpin from sidebar' : 'Pin to sidebar'}
      >
        {#if isPinned}
          <PinOff size={18} />
        {:else}
          <Pin size={18} />
        {/if}
      </button>
      <div class="relative more-menu-container">
        <button 
          on:click={toggleMoreMenu}
          class="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors {isMoreMenuOpen ? 'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-white' : 'text-slate-500 dark:text-slate-400'}"
        >
          <MoreHorizontal size={18} />
        </button>

        {#if isMoreMenuOpen}
          <div class="absolute right-0 top-full mt-1 w-52 bg-[#1c1c1c] border border-white/10 rounded-xl shadow-2xl z-[99999] overflow-hidden animate-dropdown-in py-1">
              <button
                type="button"
                on:click={() => { isDeleteConfirm = true; isMoreMenuOpen = false; }}
                class="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-red-400 hover:bg-red-500/10 hover:text-red-300 transition-colors"
              >
                <Trash2 size={14} />
                <span>{$_("taskDetail__delete_task")}</span>
              </button>
          </div>
        {/if}
      </div>
      <div class="w-px h-4 bg-slate-200 dark:bg-slate-800 mx-1"></div>
      <button 
        on:click={() => sidebarOpen = !sidebarOpen}
        class="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors {sidebarOpen ? 'text-primary dark:text-indigo-400 bg-slate-100 dark:bg-slate-800' : 'text-slate-500 dark:text-slate-400'}"
      >
        <PanelRight size={18} />
      </button>
    </div>
  </header>

  <div class="flex flex-1 overflow-hidden relative">
    <!-- Mobile Sidebar Backdrop -->
    {#if sidebarOpen}
      <div 
        class="lg:hidden absolute inset-0 bg-slate-900/20 backdrop-blur-sm z-20"
        on:click={() => sidebarOpen = false}
        on:keydown={(e) => e.key === 'Escape' && (sidebarOpen = false)}
        role="button"
        tabindex="0"
        aria-label="Close sidebar"
      ></div>
    {/if}

    <!-- Main Content -->
    <div class="flex-1 overflow-y-auto custom-scrollbar bg-white dark:bg-[#0b1120]">
      {#if loading}
        <div class="max-w-4xl mx-auto p-8 space-y-6 animate-pulse">
          <div class="h-10 bg-slate-100 dark:bg-slate-800 rounded-lg w-3/4"></div>
          <div class="space-y-3">
            <div class="h-4 bg-slate-100 dark:bg-slate-800 rounded w-full"></div>
            <div class="h-4 bg-slate-100 dark:bg-slate-800 rounded w-5/6"></div>
            <div class="h-4 bg-slate-100 dark:bg-slate-800 rounded w-4/6"></div>
          </div>
          <div class="pt-8 space-y-4">
            <div class="h-px bg-slate-100 dark:bg-slate-800"></div>
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-full bg-slate-100 dark:bg-slate-800"></div>
              <div class="flex-1 space-y-2">
                <div class="h-4 bg-slate-100 dark:bg-slate-800 rounded w-1/4"></div>
                <div class="h-16 bg-slate-100 dark:bg-slate-800 rounded w-full"></div>
              </div>
            </div>
          </div>
        </div>
      {:else if error}
        <div class="flex flex-col items-center justify-center h-full text-slate-500 p-8">
          <Folder size={48} class="mb-4 opacity-20" />
          <p>{error}</p>
          <button on:click={goBack} class="mt-4 text-primary hover:underline">
            {$_("sidebar__back")}
          </button>
        </div>
      {:else if task}
        <div class="max-w-4xl mx-auto w-full px-4 sm:px-8 py-8 space-y-10">
          <!-- Title Section -->
          <div class="space-y-4">
            {#if isEditingTitle}
              <TitleEditor
                bind:this={titleInputRef}
                bind:value={editedTitle}
                onblur={saveTitle}
                className="text-3xl font-bold tracking-tight text-slate-900 dark:text-white leading-tight"
              />
            {:else}
              <h1 
                on:click={() => { isEditingTitle = true; setTimeout(() => titleInputRef?.focus(), 0); }}
                on:keydown={(e) => e.key === 'Enter' && (isEditingTitle = true)}
                tabindex="0"
                class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white leading-tight cursor-text hover:bg-slate-50 dark:hover:bg-slate-900/50 rounded-lg transition-colors -ml-2 px-2 py-1 outline-none"
              >
                {task.title}
              </h1>
            {/if}
            
            {#if task.project}
              <div class="flex items-center gap-2 text-sm text-slate-500 dark:text-slate-400">
                <span class="font-medium">{$_("taskDetail__sub_issue_of")}</span>
                <Folder size={14} />
                <span class="hover:text-primary dark:hover:text-indigo-400 cursor-pointer transition-colors">
                  {task.project}
                </span>
              </div>
            {/if}
          </div>

          <!-- Description Section -->
          <div class="space-y-4">
            <RichTextEditor 
              value={editedDescription} 
              onchange={handleDescriptionChange}
              placeholder={$_("taskDetail__no_description")}
            />

            {#if isDescriptionDirty}
              <div class="flex items-center justify-end gap-2 mt-2 animate-fade-in">
                <button 
                  on:click={cancelDescription}
                  class="px-3 py-1.5 text-xs font-bold text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 transition-colors"
                >
                  {$_("page__cancel")}
                </button>
                <button 
                  on:click={saveDescription}
                  class="px-4 py-1.5 text-xs font-bold bg-primary hover:bg-primary-dark text-white rounded-lg shadow-lg shadow-primary/20 transition-all"
                >
                  {$_("page__save")}
                </button>
              </div>
            {/if}
          </div>

          <!-- Checklist Section -->
          <div class="space-y-4">
            <ChecklistManager 
              checklist={task.checklist || []} 
              on:update={handleChecklistUpdate}
            />
          </div>

          <hr class="border-slate-100 dark:border-slate-800" />

          <!-- Activity / Comments Section -->
          <div class="space-y-6 pb-20">
            <div class="flex items-center gap-2 font-semibold">
              <MessageCircle size={18} />
              <span>{$_("taskDetail__activity")}</span>
            </div>

            <!-- Comment Input -->
            <div class="flex gap-4 group">
              <div class="shrink-0">
                <div class="w-10 h-10 rounded-full bg-indigo-500 flex items-center justify-center text-white font-bold">
                  {$user?.profile?.first_name?.[0] || $user?.email?.[0]?.toUpperCase() || "?"}
                </div>
              </div>
              <div class="flex-1 space-y-3">
                <div class="relative">
                  <textarea 
                    bind:value={commentContent}
                    placeholder={$_("taskForm__comments_placeholder")}
                    class="w-full min-h-[100px] p-4 rounded-2xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all resize-none dark:text-white"
                  ></textarea>
                  <div class="absolute bottom-3 right-3 flex items-center gap-2 opacity-0 group-focus-within:opacity-100 transition-opacity">
                    <button 
                      on:click={handleCommentSubmit}
                      disabled={!commentContent.trim()}
                      class="p-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-xl transition-all shadow-lg shadow-indigo-500/20"
                    >
                      <Send size={18} />
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <!-- Comments List -->
            <div class="space-y-6 pt-4">
              {#if commentsLoading && comments.length === 0}
                 <p class="text-sm text-slate-500 animate-pulse">Loading activity...</p>
              {:else if comments.length === 0}
                 <p class="text-sm text-slate-400 italic">No activity yet.</p>
              {:else}
                {#each comments as comment}
                  <div class="flex gap-4">
                    <div class="shrink-0">
                      <div class="w-10 h-10 rounded-full bg-slate-200 dark:bg-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 font-bold">
                        {comment.created_by?.[0]?.toUpperCase() || "?"}
                      </div>
                    </div>
                    <div class="flex-1 space-y-1">
                      <div class="flex items-center gap-2 text-sm">
                        <span class="font-semibold text-slate-900 dark:text-white">{comment.created_by}</span>
                        <span class="text-slate-500 dark:text-slate-400 text-xs">{formatRelativeTime(comment.created_at)}</span>
                      </div>
                      <div class="bg-slate-50 dark:bg-slate-900/80 rounded-2xl p-4 text-slate-800 dark:text-slate-200 border border-slate-100 dark:border-slate-800/50">
                        <p class="whitespace-pre-wrap">{comment.content}</p>
                      </div>
                    </div>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Right Sidebar -->
    {#if sidebarOpen}
      <aside 
        class="fixed lg:relative inset-y-0 right-0 w-80 border-l border-slate-200 dark:border-slate-800 bg-white dark:bg-[#0b1120] lg:bg-slate-50/50 lg:dark:bg-[#0b1120]/50 backdrop-blur-sm overflow-y-auto custom-scrollbar p-6 space-y-8 shrink-0 z-30 lg:z-auto transition-transform duration-300 {sidebarOpen ? 'translate-x-0' : 'translate-x-full lg:hidden'}"
      >
        {#if loading}
          <!-- Sidebar Skeleton -->
          <div class="space-y-8 animate-pulse">
            <div class="space-y-4">
              <div class="h-3 bg-slate-100 dark:bg-slate-800 rounded w-1/3"></div>
              <div class="space-y-4">
                {#each Array(5) as _}
                  <div class="space-y-2">
                    <div class="h-2 bg-slate-100 dark:bg-slate-800 rounded w-1/4"></div>
                    <div class="h-10 bg-slate-100 dark:bg-slate-800 rounded-lg w-full"></div>
                  </div>
                {/each}
              </div>
            </div>
            <hr class="border-slate-100 dark:border-slate-800" />
            <div class="space-y-4">
              <div class="h-3 bg-slate-100 dark:bg-slate-800 rounded w-1/3"></div>
              <div class="space-y-3">
                {#each Array(2) as _}
                  <div class="space-y-2">
                    <div class="h-2 bg-slate-100 dark:bg-slate-800 rounded w-1/4"></div>
                    <div class="h-4 bg-slate-100 dark:bg-slate-800 rounded w-1/2"></div>
                  </div>
                {/each}
              </div>
            </div>
          </div>
        {:else if task}
        <!-- Properties Section -->
        <div class="space-y-6">
          <div class="flex items-center justify-between text-[11px] font-bold uppercase tracking-[0.05em] text-slate-400 dark:text-slate-500/70">
            <span>{$_("taskDetail__properties")}</span>
            <ChevronDown size={12} class="opacity-50" />
          </div>
          
          <div class="grid grid-cols-[80px_1fr] gap-y-1">

            <div class="col-span-2 grid grid-cols-subgrid items-center min-h-[32px] -mx-2 px-2 rounded-md hover:bg-slate-500/5 transition-colors group">
              <div class="flex items-center gap-2 text-[13px] text-slate-500 dark:text-slate-400">
                <Clock size={14} class="opacity-70 group-hover:opacity-100 transition-opacity" />
                <span>{$_("taskForm__status_label")}</span>
              </div>
              <div class="flex items-center justify-start min-w-0 property-select">
                <SearchableSelect 
                  value={task.status}
                  options={statusOptions}
                  on:select={(e: CustomEvent<any>) => handleUpdateTask({ status: e.detail })}
                  showSearch={false}
                />
              </div>
            </div>

            <div class="col-span-2 grid grid-cols-subgrid items-center min-h-[32px] -mx-2 px-2 rounded-md hover:bg-slate-500/5 transition-colors group">
              <div class="flex items-center gap-2 text-[13px] text-slate-500 dark:text-slate-400">
                <AlertTriangle size={14} class="opacity-70 group-hover:opacity-100 transition-opacity" />
                <span>Priority</span>
              </div>
              <div class="flex items-center justify-start min-w-0 property-select">
                <PrioritySelector 
                  value={task.priority || "none"}
                  on:select={(e: CustomEvent<any>) => handleUpdateTask({ priority: e.detail as Task["priority"] })}
                />
              </div>
            </div>

            <div class="col-span-2 grid grid-cols-subgrid items-center min-h-[32px] -mx-2 px-2 rounded-md hover:bg-slate-500/5 transition-colors group">
              <div class="flex items-center gap-2 text-[13px] text-slate-500 dark:text-slate-400">
                <User size={14} class="opacity-70 group-hover:opacity-100 transition-opacity" />
                <span>{$_("taskForm__assignee_label")}</span>
              </div>
              <div class="flex items-center justify-start min-w-0 property-select">
                <SearchableSelect 
                  value={task.assignee_id || task.assignee_ids?.[0] || 'all'}
                  options={[
                    { value: 'all', label: 'Unassigned' },
                    ...assignees.map(a => ({ value: a.id || '', label: a.name }))
                  ]}
                  on:select={(e: CustomEvent<any>) => handleUpdateTask({ assignee_ids: e.detail === 'all' ? [] : [e.detail] })}
                />
              </div>
            </div>

            <div class="col-span-2 grid grid-cols-subgrid items-center min-h-[32px] -mx-2 px-2 rounded-md hover:bg-slate-500/5 transition-colors group">
              <div class="flex items-center gap-2 text-[13px] text-slate-500 dark:text-slate-400">
                <Folder size={14} class="opacity-70 group-hover:opacity-100 transition-opacity" />
                <span>{$_("taskForm__project_label")}</span>
              </div>
              <div class="flex items-center justify-start min-w-0 property-select">
                <SearchableSelect 
                  value={task.project || ''}
                  options={[
                    { value: '', label: 'No Project' },
                    ...projects.map(p => ({ value: p.name, label: p.name }))
                  ]}
                  on:select={(e: CustomEvent<any>) => handleUpdateTask({ project: e.detail })}
                />
              </div>
            </div>

            <div class="col-span-2 grid grid-cols-subgrid items-center min-h-[32px] -mx-2 px-2 rounded-md hover:bg-slate-500/5 transition-colors group">
              <div class="flex items-center gap-2 text-[13px] text-slate-500 dark:text-slate-400">
                <Tag size={14} class="opacity-70 group-hover:opacity-100 transition-opacity" />
                <span>{$_("taskForm__sprint_label")}</span>
              </div>
              <div class="flex items-center justify-start min-w-0 property-select">
                <SearchableSelect 
                  value={task.sprint_id || 'all'}
                  options={[
                    { value: 'all', label: 'No Sprint' },
                    ...sprints.map(s => ({ value: s.id || '', label: s.name }))
                  ]}
                  on:select={(e: CustomEvent<any>) => handleUpdateTask({ sprint_id: e.detail === 'all' ? null : e.detail })}
                />
              </div>
            </div>

            <div class="col-span-2 grid grid-cols-subgrid items-center min-h-[32px] -mx-2 px-2 rounded-md hover:bg-slate-500/5 transition-colors group">
              <div class="flex items-center gap-2 text-[13px] text-slate-500 dark:text-slate-400">
                <Calendar size={14} class="opacity-70 group-hover:opacity-100 transition-opacity" />
                <span>{$_("taskForm__due_date_label")}</span>
              </div>
              <div class="flex items-center justify-start min-w-0 property-select">
                <CustomDatePicker
                  value={task.due_date || task.end_date || ""}
                  placeholder={$_("taskForm__due_date_placeholder")}
                  on:select={(e: CustomEvent<string>) => handleUpdateTask({ due_date: e.detail })}
                  minimal={true}
                />
              </div>
            </div>
          </div>
        </div>

        <div class="h-px bg-slate-200 dark:bg-slate-800/50 mx-2"></div>

        <!-- Details Section -->
        <div class="space-y-6">
          <div class="flex items-center justify-between text-[11px] font-bold uppercase tracking-[0.05em] text-slate-400 dark:text-slate-500/70">
            <span>{$_("taskDetail__details")}</span>
            <ChevronDown size={12} class="opacity-50" />
          </div>
          
          <div class="grid grid-cols-[80px_1fr] gap-y-1">
            <div class="col-span-2 grid grid-cols-subgrid items-center min-h-[32px] -mx-2 px-2 rounded-md group transition-colors">
               <div class="flex items-center gap-2 text-[13px] text-slate-500 dark:text-slate-400">
                 <div class="w-3.5" aria-hidden></div>
                 <span>{$_("taskDetail__created_at")}</span>
               </div>
               <span class="text-[13px] text-slate-600 dark:text-slate-400 px-2">{task.created_at ? new Date(task.created_at).toLocaleDateString() : '-'}</span>
            </div>

            {#if task.updated_at}
              <div class="col-span-2 grid grid-cols-subgrid items-center min-h-[32px] -mx-2 px-2 rounded-md group transition-colors">
                 <div class="flex items-center gap-2 text-[13px] text-slate-500 dark:text-slate-400">
                   <div class="w-3.5" aria-hidden></div>
                   <span>{$_("taskDetail__updated_at")}</span>
                 </div>
                 <span class="text-[13px] text-slate-600 dark:text-slate-400 px-2">{new Date(task.updated_at).toLocaleDateString()}</span>
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </aside>
  {/if}
</div>

{#if isBranchDialogOpen && task}
  <BranchDialog 
    show={isBranchDialogOpen}
    title={task.title}
    workspaceShortName={task.workspace_short_name || ''}
    taskNumber={task.task_number || null}
    on:close={closeBranchDialog}
  />
{/if}

<ConfirmModal 
  show={isDeleteConfirm} 
  title={$_("taskDetail__delete_task")} 
  message={$_("taskDetail__delete_confirm")}
  confirmText={$_("taskDetail__delete_confirm_yes")}
  cancelText={$_("page__cancel")}
  type="danger"
  onConfirm={handleDeleteTask}
  onClose={() => isDeleteConfirm = false}
/>
</div>

<style>
  @reference "../../../../../app.css";

  :global(.property-select .property-trigger-btn) {
    @apply !bg-transparent !border !border-solid !border-white/10 !text-gray-400 !rounded-full !h-8 !px-3 !text-[13px] hover:!bg-white/5 hover:!border-white/20 !shadow-none !ring-0 !transition-all;
  }

  :global(.property-select .property-trigger-btn span) {
    @apply !text-[13px];
  }

  :global(.property-select .property-trigger-btn svg) {
    @apply !text-slate-400;
  }

  :global(.custom-scrollbar::-webkit-scrollbar) {
    width: 6px;
  }
  :global(.custom-scrollbar::-webkit-scrollbar-track) {
    background: transparent;
  }
  :global(.custom-scrollbar::-webkit-scrollbar-thumb) {
    background: rgba(148, 163, 184, 0.1);
    border-radius: 10px;
  }
  :global(.custom-scrollbar::-webkit-scrollbar-thumb:hover) {
    background: rgba(148, 163, 184, 0.2);
  }
  
  textarea {
    scrollbar-width: none;
  }
  textarea::-webkit-scrollbar {
    display: none;
  }

  @keyframes fade-in {
    from { opacity: 0; transform: translateY(5px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  .animate-fade-in {
    animation: fade-in 0.3s ease-out forwards;
  }
</style>
