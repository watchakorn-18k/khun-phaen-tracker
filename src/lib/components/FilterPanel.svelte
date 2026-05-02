<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    ArchiveX,
    Calendar,
    Check,
    CheckCircle2,
    ChevronRight,
    Circle,
    CircleDot,
    Clock3,
    Folder,
    ListFilter,
    PlayCircle,
    Tag,
    Trash2,
    User,
    Zap,
  } from "lucide-svelte";
  import type { FilterOptions, Assignee, Task } from "$lib/types";
  import type { Sprint } from "$lib/stores/sprintStore";

  type FilterGroup =
    | "status"
    | "priority"
    | "assignee"
    | "project"
    | "label"
    | "dueDate"
    | "sprint";
  type FilterValue = string | number | null;
  type MenuOption = {
    value: FilterValue;
    label: string;
    count: number;
    icon?: any;
    iconClass?: string;
    dotColor?: string;
  };

  export let filters: FilterOptions;
  export let categories: string[] = [];
  export let projects: string[] = [];
  export let assignees: Assignee[] = [];
  export let sprints: Sprint[] = [];
  export let myAssigneeId: string | number | undefined = undefined;
  export let tasks: Task[] = [];
  export let dropdown: boolean = false;

  const dispatch = createEventDispatcher();
  let activeGroup: FilterGroup | null = null;
  let activeOptions: MenuOption[] = [];
  let activeValue: FilterValue = "all";

  function clearFilters() {
    dispatch("clearFilters");
  }

  function sameValue(a: FilterValue | undefined, b: FilterValue) {
    if (a === null || b === null) return a === b;
    if (a === undefined) return b === "all";
    return String(a) === String(b);
  }

  function countTasks(predicate: (task: Task) => boolean) {
    return tasks.filter(predicate).length;
  }

  function assigneeMatches(task: Task, value: FilterValue) {
    if (value === "all") return true;
    if (value === null) {
      return !(task.assignees?.length || task.assignee_id || task.assignee);
    }
    return (
      task.assignees?.some((assignee) => String(assignee.id) === String(value)) ||
      String(task.assignee_id || "") === String(value)
    );
  }

  function sprintMatches(task: Task, value: FilterValue) {
    if (value === "all") return true;
    if (value === null) return !task.sprint_id;
    return String(task.sprint_id || "") === String(value);
  }

  function dueDateMatches(task: Task, value: FilterValue) {
    if (value === "all") return true;
    const dueDate = task.due_date || task.end_date || "";
    if (value === "no_dates") return !dueDate;
    if (!dueDate) return false;

    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const due = new Date(dueDate);
    due.setHours(0, 0, 0, 0);

    if (value === "overdue") return due < today && task.status !== "done";
    const days = value === "next_day" ? 1 : value === "next_week" ? 7 : 30;
    const limit = new Date(today);
    limit.setDate(limit.getDate() + days);
    return due >= today && due <= limit;
  }

  $: statusOptions = [
    {
      value: "all",
      label: $_("page__filter_status_all"),
      count: tasks.length,
      icon: ListFilter,
      iconClass: "text-gray-400",
    },
    {
      value: "pending",
      label: $_("page__filter_status_pending"),
      count: countTasks((task) => task.status === "pending" && !task.is_archived),
      icon: Circle,
      iconClass: "text-slate-400",
    },
    {
      value: "todo",
      label: $_("page__filter_status_todo"),
      count: countTasks((task) => task.status === "todo" && !task.is_archived),
      icon: CircleDot,
      iconClass: "text-gray-400",
    },
    {
      value: "in-progress",
      label: $_("page__filter_status_in_progress"),
      count: countTasks((task) => task.status === "in-progress" && !task.is_archived),
      icon: PlayCircle,
      iconClass: "text-amber-400",
    },
    {
      value: "in-test",
      label: $_("page__filter_status_in_test"),
      count: countTasks((task) => task.status === "in-test" && !task.is_archived),
      icon: CircleDot,
      iconClass: "text-green-400",
    },
    {
      value: "done",
      label: $_("page__filter_status_done"),
      count: countTasks((task) => task.status === "done" && !task.is_archived),
      icon: CheckCircle2,
      iconClass: "text-blue-400",
    },
    {
      value: "archived",
      label: $_("page__filter_status_archived"),
      count: countTasks((task) => !!task.is_archived),
      icon: ArchiveX,
      iconClass: "text-gray-400",
    },
  ] satisfies MenuOption[];

  $: priorityOptions = [
    { value: "all", label: "All priorities", count: tasks.length, icon: ListFilter, iconClass: "text-gray-400" },
    { value: "urgent", label: "Urgent", count: countTasks((task) => task.priority === "urgent"), dotColor: "#ef4444" },
    { value: "high", label: "High", count: countTasks((task) => task.priority === "high"), dotColor: "#f97316" },
    { value: "medium", label: "Medium", count: countTasks((task) => task.priority === "medium"), dotColor: "#eab308" },
    { value: "low", label: "Low", count: countTasks((task) => task.priority === "low"), dotColor: "#22c55e" },
    { value: "none", label: "None", count: countTasks((task) => !task.priority || task.priority === "none"), dotColor: "#64748b" },
  ] satisfies MenuOption[];

  $: assigneeOptions = [
    {
      value: "all",
      label: $_("page__filter_assignee_all"),
      count: tasks.length,
      icon: User,
      iconClass: "text-gray-400",
    },
    ...(myAssigneeId !== undefined && myAssigneeId !== null
      ? [
          {
            value: myAssigneeId,
            label: $_("page__filter_assignee_me"),
            count: countTasks((task) => assigneeMatches(task, myAssigneeId)),
            dotColor: "#14b8a6",
          },
        ]
      : []),
    {
      value: null,
      label: $_("page__unassigned"),
      count: countTasks((task) => assigneeMatches(task, null)),
      dotColor: "#64748b",
    },
    ...assignees
      .filter((assignee) => assignee.id !== undefined && String(assignee.id) !== String(myAssigneeId))
      .map((assignee) => ({
        value: assignee.id!,
        label: assignee.name,
        count: countTasks((task) => assigneeMatches(task, assignee.id!)),
        dotColor: assignee.color || "#94a3b8",
      })),
  ] satisfies MenuOption[];

  $: projectOptions = [
    { value: "all", label: $_("page__filter_project_all"), count: tasks.length, icon: Folder, iconClass: "text-gray-400" },
    ...projects.map((project) => ({
      value: project,
      label: project,
      count: countTasks((task) => task.project === project),
      icon: Folder,
      iconClass: "text-purple-400",
    })),
  ] satisfies MenuOption[];

  $: labelOptions = [
    { value: "all", label: $_("page__filter_category_all"), count: tasks.length, icon: Tag, iconClass: "text-gray-400" },
    ...categories.map((category) => ({
      value: category,
      label: category,
      count: countTasks((task) => task.category === category),
      icon: Tag,
      iconClass: "text-indigo-400",
    })),
  ] satisfies MenuOption[];

  $: dueDateOptions = [
    { value: "all", label: $_("page__filter_due_date_all"), count: tasks.length, icon: Calendar, iconClass: "text-gray-400" },
    { value: "no_dates", label: $_("page__filter_due_date_no_dates"), count: countTasks((task) => dueDateMatches(task, "no_dates")), icon: Calendar, iconClass: "text-slate-400" },
    { value: "overdue", label: $_("page__filter_due_date_overdue"), count: countTasks((task) => dueDateMatches(task, "overdue")), icon: Clock3, iconClass: "text-red-400" },
    { value: "next_day", label: $_("page__filter_due_date_next_day"), count: countTasks((task) => dueDateMatches(task, "next_day")), icon: Calendar, iconClass: "text-blue-400" },
    { value: "next_week", label: $_("page__filter_due_date_next_week"), count: countTasks((task) => dueDateMatches(task, "next_week")), icon: Calendar, iconClass: "text-blue-400" },
    { value: "next_month", label: $_("page__filter_due_date_next_month"), count: countTasks((task) => dueDateMatches(task, "next_month")), icon: Calendar, iconClass: "text-blue-400" },
  ] satisfies MenuOption[];

  $: sprintOptions = [
    { value: "all", label: $_("page__filter_sprint_all"), count: tasks.length, icon: Zap, iconClass: "text-gray-400" },
    { value: null, label: $_("page__filter_sprint_none"), count: countTasks((task) => sprintMatches(task, null)), icon: Zap, iconClass: "text-slate-400" },
    ...sprints
      .filter((sprint) => sprint.id !== undefined)
      .map((sprint) => ({
        value: sprint.id!,
        label: sprint.name,
        count: countTasks((task) => sprintMatches(task, sprint.id!)),
        icon: Zap,
        iconClass: sprint.status === "active" ? "text-green-400" : "text-amber-400",
      })),
  ] satisfies MenuOption[];

  $: groups = [
    { id: "status", label: $_("page__filter_status"), icon: CircleDot, value: filters.status ?? "all", options: statusOptions },
    { id: "priority", label: "Priority", icon: Zap, value: filters.priority ?? "all", options: priorityOptions },
    { id: "assignee", label: $_("page__filter_assignee"), icon: User, value: filters.assignee_id ?? "all", options: assigneeOptions },
    { id: "project", label: $_("page__filter_project"), icon: Folder, value: filters.project ?? "all", options: projectOptions },
    { id: "label", label: "Label", icon: Tag, value: filters.category ?? "all", options: labelOptions },
    { id: "dueDate", label: $_("page__filter_due_date"), icon: Calendar, value: filters.dueDatePreset ?? "all", options: dueDateOptions },
    { id: "sprint", label: $_("page__filter_sprint"), icon: Zap, value: filters.sprint_id ?? "all", options: sprintOptions },
  ] as Array<{
    id: FilterGroup;
    label: string;
    icon: any;
    value: FilterValue;
    options: MenuOption[];
  }>;

  $: activeOptions = activeGroup
    ? groups.find((group) => group.id === activeGroup)?.options || []
    : [];
  $: activeValue = activeGroup
    ? groups.find((group) => group.id === activeGroup)?.value ?? "all"
    : "all";

  function selectOption(value: FilterValue) {
    filters = {
      ...filters,
      ...(activeGroup === "status" ? { status: value as FilterOptions["status"] } : {}),
      ...(activeGroup === "priority" ? { priority: value as FilterOptions["priority"] } : {}),
      ...(activeGroup === "assignee" ? { assignee_id: value as FilterOptions["assignee_id"] } : {}),
      ...(activeGroup === "project" ? { project: value as FilterOptions["project"] } : {}),
      ...(activeGroup === "label" ? { category: value as FilterOptions["category"] } : {}),
      ...(activeGroup === "dueDate" ? { dueDatePreset: value as FilterOptions["dueDatePreset"] } : {}),
      ...(activeGroup === "sprint" ? { sprint_id: value as FilterOptions["sprint_id"] } : {}),
    };
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  on:mouseleave={() => (activeGroup = null)}
  class="{dropdown
    ? 'inline-flex overflow-hidden rounded-xl border border-gray-700/80 bg-gray-950 text-gray-100 shadow-2xl'
    : 'inline-flex overflow-hidden rounded-xl border border-gray-700/80 bg-gray-950 text-gray-100 shadow-2xl'}"
>
  <div class="w-36 py-2 border-r border-gray-800">
    {#each groups as group}
      <button
        type="button"
        on:mouseenter={() => (activeGroup = group.id)}
        on:focus={() => (activeGroup = group.id)}
        on:click={() => (activeGroup = group.id)}
        class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm transition-colors hover:bg-white/5 {activeGroup === group.id ? 'bg-white/5 text-white' : 'text-gray-200'}"
      >
        <svelte:component this={group.icon} size={14} class="shrink-0 text-gray-300" />
        <span class="min-w-0 flex-1 truncate">{group.label}</span>
        {#if !sameValue(group.value, "all")}
          <span class="h-1.5 w-1.5 shrink-0 rounded-full bg-primary"></span>
        {/if}
        <ChevronRight size={14} class="shrink-0 text-gray-400" />
      </button>
    {/each}

    <div class="mx-2 my-2 h-px bg-gray-800"></div>
    <button
      type="button"
      on:click={clearFilters}
      class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-gray-300 transition-colors hover:bg-white/5 hover:text-white"
    >
      <Trash2 size={14} class="shrink-0 text-gray-400" />
      <span class="flex-1 truncate">{$_("page__btn_clear")}</span>
    </button>
  </div>

  {#if activeGroup}
    <div class="w-48 max-h-80 overflow-y-auto py-2">
      {#each activeOptions as option}
        <button
          type="button"
          on:click={() => selectOption(option.value)}
          class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm transition-colors hover:bg-white/5 {sameValue(activeValue, option.value) ? 'text-white' : 'text-gray-200'}"
        >
          <span class="flex h-4 w-4 shrink-0 items-center justify-center">
            {#if option.dotColor}
              <span class="h-2.5 w-2.5 rounded-full" style="background-color: {option.dotColor}"></span>
            {:else if option.icon}
              <svelte:component this={option.icon} size={15} class={option.iconClass || "text-gray-400"} />
            {/if}
          </span>
          <span class="min-w-0 flex-1 truncate">{option.label}</span>
          <span class="shrink-0 text-xs tabular-nums text-gray-500">{option.count}</span>
          {#if sameValue(activeValue, option.value)}
            <Check size={13} class="shrink-0 text-gray-400" />
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
