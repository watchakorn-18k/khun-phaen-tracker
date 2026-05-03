<script lang="ts">
	import { createEventDispatcher, tick } from 'svelte';
	import type { Assignee, AssigneeGroup } from '$lib/types';
	import { Users, User, Plus, X, UserMinus } from 'lucide-svelte';
	import { _ } from 'svelte-i18n';
	import SearchableSelect from './SearchableSelect.svelte';
	import Tooltip from './Tooltip.svelte';

	const dispatch = createEventDispatcher<{
		addAssignee: { name: string; color: string };
	}>();

	export let assignees: Assignee[] = [];
	export let assigneeGroups: AssigneeGroup[] = [];
	export let assignee_ids: (string | number)[] = [];
	export let readonly = false;
	export let minimal = false;
	export let selfAssigneeId: string | number | null = null;
	let showAddAssigneeForm = false;
	let newAssigneeName = '';
	let newAssigneeColor = '#6366F1';

	// Combined selection - can be either group ID or assignee ID
	let selectedValue: string | number | null = null;

	// Watch for selection changes and handle them
	$: if (selectedValue !== null) {
		const valueStr = String(selectedValue);

		if (valueStr === 'all') {
			// Clear selection
			assignee_ids = [];
		} else if (valueStr.startsWith('group_')) {
			// Group selected - add all members
			const groupId = valueStr.replace('group_', '');
			const group = assigneeGroups.find((g) => isSameId(g.id, groupId));
			if (group) {
				const next = [...assignee_ids];
				for (const memberId of group.assignee_ids || []) {
					if (!next.some((id) => isSameId(id, memberId))) {
						next.push(memberId);
					}
				}
				assignee_ids = next;
			}
		} else if (valueStr.startsWith('assignee_')) {
			// Individual assignee selected
			const assigneeId = valueStr.replace('assignee_', '');
			if (!assignee_ids.some((id) => isSameId(id, assigneeId))) {
				assignee_ids = [...assignee_ids, assigneeId];
			}
		}

		// Reset selection after handling (using tick to avoid infinite loop)
		tick().then(() => {
			selectedValue = null;
		});
	}

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

	// Build combined options: groups first, then individual assignees
	$: combinedOptions = [
		{ value: 'all', label: 'Unassigned', icon: UserMinus },
		{ value: 'label_members', label: 'Members', disabled: true },
		// Groups (if any)
		...assigneeGroups
			.filter((g) => g.id !== undefined && g.assignee_ids.length > 0)
			.map((group) => ({
				value: 'group_' + String(group.id),
				label: `👥 ${group.name} (${group.assignee_ids.length})`,
				isGroup: true,
				groupId: group.id
			})),
		// Individual assignees
		...availableAssignees
			.filter((assignee) => assignee.id !== undefined)
			.map((assignee) => ({
				value: 'assignee_' + String(assignee.id),
				label: assignee.name,
				badge: true,
				badgeColor: assignee.color,
				isAssignee: true,
				assigneeId: assignee.id
			}))
	];

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

	function cancelAddAssignee() {
		newAssigneeName = '';
		newAssigneeColor = '#6366F1';
		showAddAssigneeForm = false;
	}

	$: isSelfAssigned = selfAssigneeId != null && assignee_ids.some((id) => isSameId(id, selfAssigneeId));

	function assignSelf() {
		if (selfAssigneeId == null || isSelfAssigned) return;
		assignee_ids = [...assignee_ids, selfAssigneeId];
	}
</script>

<div>
	{#if !minimal}
		<label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 flex items-center gap-1">
			<User size={14} />
			{$_('taskForm__assignee_label')}
		</label>
	{/if}

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
		{#if minimal && selectedAssignees.length > 0}
			<!-- Minimal mode: stacked avatars + add button -->
			<div class="flex items-center gap-1.5 h-8">
				<div class="flex items-center -space-x-2">
					{#each selectedAssignees as assignee (assignee.id)}
						<div class="relative hover:z-10">
							<Tooltip text={assignee.name}>
								<button
									type="button"
									on:click={() => { if (!readonly && assignee.id != null) removeAssigneeFromTask(assignee.id); }}
									class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold text-white ring-2 ring-gray-900 shrink-0 {!readonly ? 'hover:ring-red-500 hover:brightness-75 transition-all' : ''}"
									style="background-color: {assignee.color || '#6366f1'}"
								>
									{assignee.name?.[0]?.toUpperCase() || '?'}
								</button>
							</Tooltip>
						</div>
					{/each}
				</div>
				{#if !readonly}
					<SearchableSelect
						bind:value={selectedValue}
						options={combinedOptions}
						placeholder="+"
						minimal
						customClass="!px-1.5 !min-w-0 w-8"
					/>
				{/if}
			</div>
		{:else}
			{#if !minimal && selectedAssignees.length > 0}
				<div class="flex flex-wrap gap-1.5 mb-2">
					{#each selectedAssignees as assignee (assignee.id)}
						<div class="inline-flex items-center gap-1.5 px-2.5 py-1.5 border border-gray-200 dark:border-gray-600 bg-gray-50 dark:bg-gray-700/50 text-sm rounded-lg transition-colors">
							<span class="w-2 h-2 rounded-full shrink-0" style="background-color: {assignee.color}"></span>
							<span class="text-gray-700 dark:text-gray-300">{assignee.name}</span>
							{#if !readonly}
								<button
									type="button"
									on:click={() => { if (assignee.id != null) removeAssigneeFromTask(assignee.id); }}
									class="text-gray-400 hover:text-red-500 transition-colors ml-0.5"
									title="Remove assignee"
								>
									<X size={12} />
								</button>
							{/if}
						</div>
					{/each}
				</div>
			{/if}

			{#if !readonly}
				<SearchableSelect
					bind:value={selectedValue}
					options={combinedOptions}
					placeholder={$_('taskForm__assignee_placeholder')}
					{minimal}
				/>
			{:else if selectedAssignees.length === 0}
				<p class="text-sm text-gray-500 dark:text-gray-400 italic">{$_('taskForm__unassigned')}</p>
			{/if}

			{#if selfAssigneeId != null && !isSelfAssigned}
				<button
					type="button"
					on:click={assignSelf}
					class="mt-1.5 inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-primary border border-primary/30 rounded-lg hover:bg-primary/10 transition-colors"
				>
					<User size={14} />
					{$_('taskForm__assign_to_me')}
				</button>
			{/if}
		{/if}
	{/if}
</div>
