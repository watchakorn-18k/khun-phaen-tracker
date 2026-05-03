<script lang="ts">
  import { dndzone, TRIGGERS, type DndEvent } from "svelte-dnd-action";
  import { createEventDispatcher, tick } from "svelte";
  import type { Task, Sprint } from "$lib/types";
  import { MoreHorizontal, Plus, ListTodo } from "lucide-svelte";
  import PaginationFooter from "./PaginationFooter.svelte";
  import PriorityBadge from "./PriorityBadge.svelte";
  import { _ } from "$lib/i18n";
  import { columnSettings } from "$lib/stores/columnSettings";

  $: enabledColumns = new Map($columnSettings.map((s) => [s.id, s.enabled]));

  const dispatch = createEventDispatcher<{
    move: { id: string | number; newStatus: Task["status"] };
    edit: Task;
    delete: string | number;
    dragState: { dragging: boolean };
    pageChange: number;
    pageSizeSettings: void;
  }>();

  export let tasks: Task[] = [];
  export let currentPage: number = 1;
  export let totalPages: number = 1;
  export let totalTasks: number = 0;
  export let pageSize: number = 20;

  interface TaskWithRequiredId extends Task {
    id: string | number;
  }

  let todoItems: TaskWithRequiredId[] = [];
  let inProgressItems: TaskWithRequiredId[] = [];
  let inTestItems: TaskWithRequiredId[] = [];
  let doneItems: TaskWithRequiredId[] = [];

  let isDragging = false;

  function syncItemsFromTasks(nextTasks: Task[]) {
    if (isDragging) return;

    todoItems = nextTasks.filter(
      (t): t is TaskWithRequiredId =>
        (t.status === "todo" || t.status === "pending") &&
        t.id !== undefined &&
        t.id !== null,
    );
    inProgressItems = nextTasks.filter(
      (t): t is TaskWithRequiredId =>
        t.status === "in-progress" && t.id !== undefined && t.id !== null,
    );
    inTestItems = nextTasks.filter(
      (t): t is TaskWithRequiredId =>
        t.status === "in-test" && t.id !== undefined && t.id !== null,
    );
    doneItems = nextTasks.filter(
      (t): t is TaskWithRequiredId =>
        t.status === "done" && t.id !== undefined && t.id !== null,
    );
  }

  $: if (!isDragging && tasks) {
    syncItemsFromTasks(tasks);
  }

  $: statusColumns = [
    {
      status: "todo" as Task["status"],
      items: todoItems,
      label: $_("kanbanBoard__column_todo"),
      dotClass: "border-2 border-gray-400 dark:border-gray-500",
      countClass: "text-gray-500 dark:text-gray-400",
      colBg: "bg-gray-50 dark:bg-gray-800/40",
    },
    {
      status: "in-progress" as Task["status"],
      items: inProgressItems,
      label: $_("kanbanBoard__column_in_progress"),
      dotClass: "bg-orange-400",
      countClass: "text-orange-600 dark:text-orange-400",
      colBg: "bg-orange-50/40 dark:bg-orange-900/10",
    },
    {
      status: "in-test" as Task["status"],
      items: inTestItems,
      label: $_("kanbanBoard__column_in_test"),
      dotClass: "bg-violet-500",
      countClass: "text-violet-600 dark:text-violet-400",
      colBg: "bg-violet-50/40 dark:bg-violet-900/10",
    },
    {
      status: "done" as Task["status"],
      items: doneItems,
      label: $_("kanbanBoard__column_done"),
      dotClass: "bg-green-500",
      countClass: "text-green-600 dark:text-green-400",
      colBg: "bg-green-50/30 dark:bg-green-900/10",
    },
  ];

  function handleDndConsider(
    e: CustomEvent<DndEvent<TaskWithRequiredId>>,
    status: Task["status"],
  ) {
    if (!isDragging) dispatch("dragState", { dragging: true });
    isDragging = true;
    const items = e.detail.items;
    switch (status) {
      case "todo": todoItems = items; break;
      case "in-progress": inProgressItems = items; break;
      case "in-test": inTestItems = items; break;
      case "done": doneItems = items; break;
    }
  }

  async function handleDndFinalize(
    e: CustomEvent<DndEvent<TaskWithRequiredId>>,
    status: Task["status"],
  ) {
    const finishDrag = async () => {
      await tick();
      isDragging = false;
      dispatch("dragState", { dragging: false });
    };
    const items = e.detail.items;
    const trigger = e.detail.info.trigger as any;
    const findTaskById = (id: string | number) =>
      tasks.find((t) => t.id !== undefined && String(t.id) === String(id));

    switch (status) {
      case "todo": todoItems = items; break;
      case "in-progress": inProgressItems = items; break;
      case "in-test": inTestItems = items; break;
      case "done": doneItems = items; break;
    }

    if (trigger === (TRIGGERS.DROPPED_INTO_ANOTHER as any)) return;

    if (trigger === (TRIGGERS.DROPPED_INTO_ZONE as any)) {
      let droppedId: string | number | null = null;
      const infoId = e.detail.info.id;

      if (typeof infoId === "string" || typeof infoId === "number") {
        const droppedItem = items.find((item) => String(item.id) === String(infoId));
        droppedId = droppedItem?.id ?? infoId;
      }

      if (droppedId === null) {
        const movedCandidate = items.find((item) => {
          const original = findTaskById(item.id);
          return original ? original.status !== status : false;
        });
        droppedId = movedCandidate?.id ?? null;
      }

      if (droppedId !== null) {
        const originalTask = findTaskById(droppedId);
        if (originalTask && originalTask.status !== status) {
          dispatch("move", { id: droppedId, newStatus: status });
        }
      }

      await finishDrag();
    } else {
      await finishDrag();
    }
  }

  import { currentWorkspaceShortName } from "$lib/stores/workspace";

  function getTaskId(task: Task): string {
    const prefix = task.workspace_short_name || $currentWorkspaceShortName || "TASK";
    if (task.task_number) return `${prefix}-${task.task_number}`;
    return "";
  }

  function getCategoryBadgeClass(category: string): string {
    const lower = category?.toLowerCase() || "";
    if (lower.includes("high") || lower.includes("urgent") || lower.includes("สูง"))
      return "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300";
    if (lower.includes("low") || lower.includes("ต่ำ"))
      return "bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400";
    return "bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300";
  }

  function getCardOpacity(status: Task["status"]): string {
    return status === "done" ? "opacity-60" : "";
  }

  let openMenuId: string | number | null = null;
  let openColumnMenu: string | null = null;
</script>

<div class="flex flex-col">
  <!-- Kanban columns -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3 overflow-x-auto pb-4">
    {#each statusColumns as { status, items, label, dotClass, countClass, colBg }}
      <div class="{colBg} rounded-xl p-3 flex flex-col min-w-65">

        <!-- Column header -->
        <div class="flex items-center justify-between mb-3 px-0.5">
          <div class="flex items-center gap-2">
            <span class="w-2.5 h-2.5 rounded-full {dotClass} shrink-0"></span>
            <span class="text-sm font-semibold text-gray-700 dark:text-gray-200">{label}</span>
            <span class="text-xs font-medium {countClass} tabular-nums">{items.length}</span>
          </div>
          <div class="flex items-center gap-0.5">
            <button
              on:click|stopPropagation={() =>
                (openColumnMenu = openColumnMenu === status ? null : status)}
              class="p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
            >
              <MoreHorizontal size={14} />
            </button>
            <button
              class="p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
              title="Add task"
            >
              <Plus size={14} />
            </button>
          </div>
        </div>

        <!-- Drop zone -->
        <div
          use:dndzone={{ items, flipDurationMs: 200 }}
          on:consider={(e) => handleDndConsider(e, status)}
          on:finalize={(e) => handleDndFinalize(e, status)}
          class="space-y-2 min-h-75 flex-1"
        >
          {#each items as task (task.id)}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <div
              class="relative group bg-white dark:bg-gray-900 rounded-lg border border-gray-200/80 dark:border-gray-700/60 p-3 cursor-pointer hover:border-gray-300 dark:hover:border-gray-600 hover:shadow-sm transition-all {getCardOpacity(status)}"
              on:click={() => dispatch("edit", task)}
              role="button"
              tabindex="0"
            >
              <!-- Task ID + menu -->
               <div class="flex items-center justify-between mb-1">
                 <div class="flex items-center">
                   {#if enabledColumns.get('priority') && task.priority && task.priority !== 'none'}
                     <PriorityBadge priority={task.priority} />
                   {/if}
                 </div>
                <button
                  on:click|stopPropagation={() =>
                    (openMenuId = openMenuId === task.id ? null : task.id)}
                  class="p-0.5 rounded text-gray-300 dark:text-gray-600 hover:text-gray-500 dark:hover:text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  <MoreHorizontal size={13} />
                </button>
              </div>

              <!-- Title -->
              <h4 class="text-sm font-semibold text-gray-900 dark:text-white leading-snug mb-1.5 {status === 'done' ? 'line-through' : ''}">
                {#if enabledColumns.get('id') && getTaskId(task)}
                  <span class="text-[10px] font-bold text-gray-400 dark:text-gray-500 mr-1.5 tracking-wide">
                    {getTaskId(task)}
                  </span>
                {/if}
                {task.title}
              </h4>

               <!-- Description (notes) -->
               {#if enabledColumns.get('description') && task.notes}
                 <p class="text-xs text-gray-400 dark:text-gray-500 line-clamp-2 mb-2">
                   {task.notes}
                 </p>
               {/if}

               <!-- Checklist progress -->
               {#if enabledColumns.get('checklist') && task.checklist && task.checklist.length > 0}
                 {@const completed = task.checklist.filter((i) => i.completed).length}
                 {@const total = task.checklist.length}
                 {@const percent = Math.round((completed / total) * 100)}
                 <div class="mb-2">
                   <div class="flex items-center gap-1.5 text-[10px] text-gray-400 dark:text-gray-500 mb-1">
                     <ListTodo size={10} />
                     <span>{completed}/{total}</span>
                     <div class="flex-1 h-1 bg-gray-100 dark:bg-gray-700/50 rounded-full overflow-hidden">
                       <div class="h-full bg-primary transition-all duration-300" style="width: {percent}%"></div>
                     </div>
                   </div>
                 </div>
               {/if}

              <!-- Footer: assignees + category -->
              <div class="flex items-center justify-between mt-1.5 gap-2">
                <!-- Assignee avatars -->
                <div class="flex items-center -space-x-1.5">
                  {#if task.assignees && task.assignees.length > 0}
                    {#each task.assignees.slice(0, 3) as assignee}
                      <div
                        class="w-5 h-5 rounded-full border-2 border-white dark:border-gray-900 flex items-center justify-center text-[9px] font-bold text-white shrink-0"
                        style="background-color: {assignee.color || '#6366f1'}"
                        title={assignee.name}
                      >
                        {assignee.name?.[0]?.toUpperCase() || "?"}
                      </div>
                    {/each}
                    {#if task.assignees.length > 3}
                      <div class="w-5 h-5 rounded-full border-2 border-white dark:border-gray-900 bg-gray-200 dark:bg-gray-700 flex items-center justify-center text-[9px] font-bold text-gray-600 dark:text-gray-300 shrink-0">
                        +{task.assignees.length - 3}
                      </div>
                    {/if}
                  {/if}
                </div>

                <div class="flex items-center gap-1.5 flex-wrap justify-end">
                  <!-- Category badge -->
                  {#if task.category}
                    <span class="text-[10px] font-medium px-1.5 py-0.5 rounded {getCategoryBadgeClass(task.category)} shrink-0">
                      {task.category}
                    </span>
                  {/if}
                </div>
              </div>

              <!-- Context menu -->
              {#if openMenuId === task.id}
                <div class="absolute right-2 top-7 w-32 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-10">
                  <button
                    on:click={() => { dispatch("edit", task); openMenuId = null; }}
                    class="w-full px-3 py-1.5 text-left text-xs text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
                  >
                    {$_("kanbanBoard__edit")}
                  </button>
                  <button
                    on:click={() => { dispatch("delete", task.id!); openMenuId = null; }}
                    class="w-full px-3 py-1.5 text-left text-xs text-danger hover:bg-gray-50 dark:hover:bg-gray-700"
                  >
                    {$_("kanbanBoard__delete")}
                  </button>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  {#if totalTasks > 0}
    <PaginationFooter
      {currentPage}
      {totalPages}
      {totalTasks}
      {pageSize}
      on:pageChange
      on:pageSizeSettings
    />
  {/if}
</div>

<!-- Click outside to close menus -->
{#if openMenuId !== null || openColumnMenu !== null}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <button
    class="fixed inset-0 z-0"
    on:click={() => { openMenuId = null; openColumnMenu = null; }}
    tabindex="-1"
    aria-label={$_("kanbanBoard__close_menu")}
  ></button>
{/if}
