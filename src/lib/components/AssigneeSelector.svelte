<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { Assignee, AssigneeGroup } from '$lib/types';
	import { Users, User, Plus, Trash2, X } from 'lucide-svelte';
	import { _ } from 'svelte-i18n';
	import SearchableSelect from './SearchableSelect.svelte';

	const dispatch = createEventDispatcher<{
		addAssignee: { name: string; color: string };
		addAssigneeGroup: { name: string; assignee_ids: (string | number)[] };
		deleteAssigneeGroup: { id: string | number };
	}>();

	export let assignees: Assignee[] = [];
	export let assigneeGroups: AssigneeGroup[] = [];
	export let assignee_ids: (string | number)[] = [];
	export let assignee_id_to_add: string | number | null = null;
	export let readonly = false;
	let showAddAssigneeForm = false;
	let showAddGroupForm = false;
	let newAssigneeName = '';
	let newAssigneeColor = '#6366F1';
	let newGroupName = '';
	let selectedGroupAssigneeIds: (string | number)[] = [];
	let selectedGroupIdToAdd: string | number | null = null;

	const colorOptions = [
		'#EF4444', '#F97316', '#F59E0B', '#84CC16', '#22C55E',
		'#10B981', '#14B8A6', '#06B6D4', '#0EA5E9', '#3B82F6',
		'#6366F1', '#8B5CF6', '#A855F7', '#D946EF', '#EC4899',
		'#F43F5E', '#78716C', '#6B7280', '#4B5563', '#1F2937'
	];

	const isSameId = (a: string | number | null | undefined, b: string | number | null | undefined) =>
		a !== null && a !== undefined && b !== null && b !== undefined && String(a) === String(b);

	$: selectedAssignees = assignee_ids
		.map((id) => assignees.find((a) => isSameId(a.id, id)))
		.filter(Boolean) as Assignee[];
	$: availableAssignees = assignees.filter((a) => a.id != null && !assignee_ids.some((id) => isSameId(a.id, id)));
	$: selectedGroupToAdd = assigneeGroups.find((g) => isSameId(g.id, selectedGroupIdToAdd));

	function addAssigneeToTask() {
		if (assignee_id_to_add !== null && !assignee_ids.some((id) => isSameId(id, assignee_id_to_add))) {
			assignee_ids = [...assignee_ids, assignee_id_to_add];
			assignee_id_to_add = null;
		}
	}

	function addSelectedGroupMembersToTask() {
		if (!selectedGroupToAdd) return;
		const next = [...assignee_ids];
		for (const memberId of selectedGroupToAdd.assignee_ids || []) {
			if (!next.some((id) => isSameId(id, memberId))) {
				next.push(memberId);
			}
		}
		assignee_ids = next;
		selectedGroupIdToAdd = null;
	}

	function removeAssigneeFromTask(id: string | number) {
		assignee_ids = assignee_ids.filter((assigneeId) => !isSameId(assigneeId, id));
	}

	function handleAddAssignee() {
		if (!newAssigneeName.trim()) return;

		dispatch('addAssignee', {
			name: newAssigneeName.trim(),
			color: newAssigneeColor
		});

		newAssigneeName = '';
		newAssigneeColor = '#6366F1';
		showAddAssigneeForm = false;
	}

	function handleCreateGroup() {
		if (!newGroupName.trim()) return;
		dispatch('addAssigneeGroup', {
			name: newGroupName.trim(),
			assignee_ids: selectedGroupAssigneeIds
		});
		newGroupName = '';
		selectedGroupAssigneeIds = [];
		showAddGroupForm = false;
	}

	function handleDeleteGroup(groupId: string | number | undefined) {
		if (groupId == null) return;
		dispatch('deleteAssigneeGroup', { id: groupId });
	}

	function toggleGroupMember(id: string | number | undefined) {
		if (id == null) return;
		if (selectedGroupAssigneeIds.some((x) => isSameId(x, id))) {
			selectedGroupAssigneeIds = selectedGroupAssigneeIds.filter((x) => !isSameId(x, id));
			return;
		}
		selectedGroupAssigneeIds = [...selectedGroupAssigneeIds, id];
	}

	function cancelAddAssignee() {
		newAssigneeName = '';
		newAssigneeColor = '#6366F1';
		showAddAssigneeForm = false;
	}

	function cancelAddGroup() {
		newGroupName = '';
		selectedGroupAssigneeIds = [];
		showAddGroupForm = false;
	}
</script>

<div>
	<label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 flex items-center gap-1">
		<User size={14} />
		{$_('taskForm__assignee_label')}
	</label>

	{#if showAddAssigneeForm}
		<div class="bg-gray-50 dark:bg-gray-700 p-3 rounded-lg space-y-3">
			<div>
				<label for="new-assignee-name" class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">{$_('taskForm__new_assignee_name')}</label>
				<input
					id="new-assignee-name"
					type="text"
					bind:value={newAssigneeName}
					placeholder={$_('taskForm__new_assignee_placeholder')}
					class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none text-sm dark:bg-gray-700 dark:text-white"
				/>
			</div>
			<div>
				<label for="new-assignee-color-picker" class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">{$_('taskForm__color_label')}</label>
				<div class="flex flex-wrap gap-1.5 mb-2">
					{#each colorOptions as color}
						<button
							type="button"
							on:click={() => newAssigneeColor = color}
							aria-label={$_('taskForm__select_color', { values: { color } })}
							class="w-6 h-6 rounded-full border-2 transition-all {newAssigneeColor === color ? 'border-gray-800 dark:border-white scale-110' : 'border-transparent'}"
							style="background-color: {color}"
						></button>
					{/each}
				</div>
				<input
					id="new-assignee-color-picker"
					type="color"
					bind:value={newAssigneeColor}
					class="w-10 h-8 rounded cursor-pointer border border-gray-300 dark:border-gray-600"
				/>
				<input
					type="text"
					bind:value={newAssigneeColor}
					placeholder="#6366F1"
					class="flex-1 px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none dark:bg-gray-700 dark:text-white"
					maxlength="7"
				/>
			</div>
		</div>
		<div class="flex gap-2 pt-1">
			<button
				type="button"
				on:click={handleAddAssignee}
				disabled={!newAssigneeName.trim()}
				class="flex-1 bg-primary hover:bg-primary-dark disabled:bg-gray-300 dark:disabled:bg-gray-600 text-white py-1.5 px-3 rounded-lg text-sm font-medium transition-colors"
			>
				{$_('taskForm__btn_add')}
			</button>
			<button
				type="button"
				on:click={cancelAddAssignee}
				class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg text-sm hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
			>
				{$_('taskForm__btn_cancel')}
			</button>
		</div>
	{:else}
		{#if selectedAssignees.length > 0}
			<div class="flex flex-wrap gap-2 mb-2">
				{#each selectedAssignees as assignee (assignee.id)}
					<div class="inline-flex items-center gap-1.5 px-2.5 py-1.5 bg-gray-100 dark:bg-gray-700 rounded-lg text-sm">
						<span class="w-2.5 h-2.5 rounded-full" style="background-color: {assignee.color}"></span>
						<span class="text-gray-700 dark:text-gray-300">{assignee.name}</span>
						{#if !readonly}
							<button
								type="button"
								on:click={() => {
									if (assignee.id != null) removeAssigneeFromTask(assignee.id);
								}}
								class="text-gray-400 hover:text-red-500 transition-colors"
								title="Remove assignee"
							>
								<X size={14} />
							</button>
						{/if}
					</div>
				{/each}
			</div>
		{/if}

		{#if !readonly}
			<div class="space-y-2">
				<div class="flex gap-2">
					<div class="flex-1">
						<SearchableSelect
							bind:value={assignee_id_to_add}
							options={[
								{ value: null, label: $_('taskForm__assignee_placeholder') },
								...availableAssignees
								.filter((assignee) => assignee.id !== undefined)
								.map(assignee => ({
									value: assignee.id!,
									label: assignee.name,
									badge: true,
									badgeColor: assignee.color
								}))
							]}
							placeholder={$_('taskForm__assignee_placeholder')}
						/>
					</div>
					<button
						type="button"
						on:click={addAssigneeToTask}
						disabled={assignee_id_to_add === null}
						class="px-3 py-2 border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-1"
						title="Add selected assignee"
					>
						<Plus size={16} />
					</button>
					<button
						type="button"
						on:click={() => showAddAssigneeForm = true}
						class="px-3 py-2 border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center gap-1"
						title={$_('taskForm__add_assignee')}
					>
						<User size={16} />
						<Plus size={12} />
					</button>
				</div>

				<div class="rounded-lg border border-gray-200 dark:border-gray-700 p-2 space-y-2">
					<div class="text-xs font-semibold text-gray-600 dark:text-gray-300 flex items-center gap-1"><Users size={13} />Assignee Groups</div>
					{#if showAddGroupForm}
						<div class="space-y-2">
							<input
								type="text"
								bind:value={newGroupName}
								placeholder="Group name"
								class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none text-sm dark:bg-gray-700 dark:text-white"
							/>
							<div class="max-h-28 overflow-y-auto space-y-1">
								{#each assignees.filter((a) => a.id != null) as assignee (assignee.id)}
									<label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
										<input
											type="checkbox"
											checked={selectedGroupAssigneeIds.some((id) => isSameId(id, assignee.id))}
											on:change={() => toggleGroupMember(assignee.id)}
										/>
										<span class="w-2.5 h-2.5 rounded-full" style="background-color: {assignee.color}"></span>
										<span>{assignee.name}</span>
									</label>
								{/each}
							</div>
							<div class="flex gap-2">
								<button
									type="button"
									on:click={handleCreateGroup}
									disabled={!newGroupName.trim()}
									class="flex-1 bg-primary hover:bg-primary-dark disabled:bg-gray-300 dark:disabled:bg-gray-600 text-white py-1.5 px-3 rounded-lg text-sm font-medium transition-colors"
								>
									Create Group
								</button>
								<button
									type="button"
									on:click={cancelAddGroup}
									class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg text-sm hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
								>
									Cancel
								</button>
							</div>
						</div>
					{:else}
						<div class="flex gap-2">
							<div class="flex-1">
								<SearchableSelect
									bind:value={selectedGroupIdToAdd}
									options={[
										{ value: null, label: 'เลือกกลุ่มผู้รับผิดชอบ' },
										...assigneeGroups
											.filter((g) => g.id !== undefined)
											.map((group) => ({
												value: group.id!,
												label: `${group.name} (${group.assignee_ids.length})`
											}))
									]}
									placeholder="เลือกกลุ่มผู้รับผิดชอบ"
								/>
							</div>
							<button
								type="button"
								on:click={addSelectedGroupMembersToTask}
								disabled={selectedGroupIdToAdd === null}
								class="px-3 py-2 border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
								title="Add members from selected group"
							>
								<Plus size={16} />
							</button>
							<button
								type="button"
								on:click={() => showAddGroupForm = true}
								class="px-3 py-2 border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
								title="Create group"
							>
								<Users size={16} />
								<Plus size={12} />
							</button>
						</div>
					{/if}

					{#if assigneeGroups.length > 0}
						<div class="flex flex-wrap gap-1.5 pt-1">
							{#each assigneeGroups as group (group.id)}
								<div class="inline-flex items-center gap-1 px-2 py-1 rounded bg-gray-100 dark:bg-gray-700 text-xs text-gray-700 dark:text-gray-300">
									<span>{group.name} ({group.assignee_ids.length})</span>
									<button type="button" class="text-gray-400 hover:text-red-500" on:click={() => handleDeleteGroup(group.id)} title="Delete group">
										<Trash2 size={12} />
									</button>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		{:else if selectedAssignees.length === 0}
			<p class="text-sm text-gray-500 dark:text-gray-400 italic">{$_('taskForm__unassigned')}</p>
		{/if}
	{/if}
</div>
