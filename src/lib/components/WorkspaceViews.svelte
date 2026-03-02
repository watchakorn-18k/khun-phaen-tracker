<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import KanbanBoard from "./KanbanBoard.svelte";
  import TaskList from "./TaskList.svelte";
  import CalendarView from "./CalendarView.svelte";
  import TableView from "./TableView.svelte";
  import GanttView from "./GanttView.svelte";
  import WorkloadView from "./WorkloadView.svelte";
  import TaskViewPlaceholder from "./TaskViewPlaceholder.svelte";
  import type { Task, Assignee } from "$lib/types";
  import type { Sprint } from "$lib/stores/sprintStore";

  export let loadingData: boolean;
  export let filteredTasks: Task[];
  export let tasks: Task[];
  export let currentView: string;
  export let sprints: Sprint[];
  export let currentPage: any;
  export let totalPages: number;
  export let totalTasks: number;
  export let pageSize: any;
  export let sprintManagerTasks: Task[];
  export let assignees: Assignee[];

  const dispatch = createEventDispatcher();
</script>

<div class="mt-8">
  {#if loadingData}
    <TaskViewPlaceholder loading={true} />
  {:else if filteredTasks.length === 0}
    <TaskViewPlaceholder loading={false} />
  {:else if currentView === "kanban"}
    <div class="animate-fade-in">
      <KanbanBoard
        {tasks}
        {sprints}
        currentPage={$currentPage}
        {totalPages}
        {totalTasks}
        pageSize={$pageSize}
        on:move={(e) =>
          dispatch("statusChange", {
            id: e.detail.id,
            status: e.detail.newStatus,
          })}
        on:edit={(e) => dispatch("edit", e.detail)}
        on:delete={(e) => dispatch("delete", e.detail)}
        on:dragState={(e) => dispatch("dragState", e.detail)}
        on:pageChange={(e) => dispatch("pageChange", e.detail)}
        on:pageSizeSettings={() => dispatch("pageSizeSettings")}
      />
    </div>
  {:else if currentView === "list"}
    <div class="animate-fade-in">
      <TaskList
        {tasks}
        {sprints}
        currentPage={$currentPage}
        {totalPages}
        {totalTasks}
        pageSize={$pageSize}
        on:edit={(e) => dispatch("edit", e.detail)}
        on:delete={(e) => dispatch("delete", e.detail)}
        on:statusChange={(e) => dispatch("statusChange", e.detail)}
        on:pageChange={(e) => dispatch("pageChange", e.detail)}
        on:pageSizeSettings={() => dispatch("pageSizeSettings")}
      />
    </div>
  {:else if currentView === "calendar"}
    <div class="animate-fade-in">
      <CalendarView
        tasks={filteredTasks}
        on:selectTask={(e) => dispatch("edit", e.detail)}
      />
    </div>
  {:else if currentView === "table"}
    <div class="animate-fade-in">
      <TableView
        {tasks}
        {sprints}
        currentPage={$currentPage}
        {totalPages}
        {totalTasks}
        pageSize={$pageSize}
        on:edit={(e) => dispatch("edit", e.detail)}
        on:delete={(e) => dispatch("delete", e.detail)}
        on:deleteSelected={(e) => dispatch("deleteSelected", e.detail)}
        on:statusChange={(e) => dispatch("statusChange", e.detail)}
        on:checklistToggle={(e) => dispatch("checklistToggle", e.detail)}
        on:exportQR={(e) => dispatch("exportQR", e.detail)}
        on:pageChange={(e) => dispatch("pageChange", e.detail)}
        on:pageSizeSettings={() => dispatch("pageSizeSettings")}
      />
    </div>
  {:else if currentView === "gantt"}
    <GanttView
      tasks={filteredTasks}
      {sprints}
      on:edit={(e) => dispatch("edit", e.detail)}
    />
  {:else if currentView === "workload"}
    <WorkloadView
      tasks={sprintManagerTasks}
      {assignees}
      on:selectAssignee={(e) => dispatch("selectAssignee", e.detail)}
    />
  {/if}
</div>
