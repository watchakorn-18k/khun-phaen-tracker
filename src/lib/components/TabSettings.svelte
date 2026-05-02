<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { tabSettings, type TabConfig, type TabId } from '$lib/stores/tabSettings';
	import { List, CalendarDays, Columns3, Table, GanttChart, UsersRound, GripVertical, ChevronUp, ChevronDown, RotateCcw, X, Check, Settings2, Eye, EyeOff } from 'lucide-svelte';
	import { _ } from 'svelte-i18n';
	import { flip } from 'svelte/animate';
	import { quintOut } from 'svelte/easing';

	const dispatch = createEventDispatcher<{
		close: void;
		save: void;
	}>();

	export let hiddenTabIds: TabId[] = [];

	// Local copy for editing
	let editingTabs: TabConfig[] = [];
	let hasChanges = false;
	let draggedIndex: number | null = null;
	let dragOverIndex: number | null = null;
	let isPressing = false;
	let pressingIndex: number | null = null;

	// Subscribe to get initial values
	const unsubscribe = tabSettings.subscribe(tabs => {
		if (editingTabs.length === 0) {
			editingTabs = tabs.filter((tab) => !hiddenTabIds.includes(tab.id));
		}
	});

	$: visibleStoreTabs = $tabSettings.filter((tab) => !hiddenTabIds.includes(tab.id));
	$: if (editingTabs.length > 0) {
		editingTabs = editingTabs.filter((tab) => !hiddenTabIds.includes(tab.id));
	}

	function getIcon(iconName: string) {
		switch (iconName) {
			case 'List': return List;
			case 'CalendarDays': return CalendarDays;
			case 'Columns3': return Columns3;
			case 'Table': return Table;
			case 'GanttChart': return GanttChart;
			case 'UsersRound': return UsersRound;
			default: return List;
		}
	}

	function enabledCount() {
		return editingTabs.filter((tab) => tab.enabled).length;
	}

	function toggleTabEnabled(id: TabId) {
		const count = enabledCount();
		editingTabs = editingTabs.map((tab) => {
			if (tab.id !== id) return tab;
			if (tab.enabled && count <= 1) return tab;
			return { ...tab, enabled: !tab.enabled };
		});
		hasChanges = true;
	}

	function moveUp(index: number) {
		if (index <= 0) return;
		const newTabs = [...editingTabs];
		[newTabs[index - 1], newTabs[index]] = [newTabs[index], newTabs[index - 1]];
		editingTabs = newTabs;
		hasChanges = true;
	}

	function moveDown(index: number) {
		if (index >= editingTabs.length - 1) return;
		const newTabs = [...editingTabs];
		[newTabs[index], newTabs[index + 1]] = [newTabs[index + 1], newTabs[index]];
		editingTabs = newTabs;
		hasChanges = true;
	}

	// Drag and drop handlers
	function handleDragStart(index: number) {
		draggedIndex = index;
		isPressing = false;
		pressingIndex = null;
	}

	function handleDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		if (draggedIndex === null || draggedIndex === index) return;
		
		// Real-time reordering while dragging
		if (dragOverIndex !== index) {
			dragOverIndex = index;
			
			// Animate swap
			const newTabs = [...editingTabs];
			const [removed] = newTabs.splice(draggedIndex, 1);
			newTabs.splice(index, 0, removed);
			
			editingTabs = newTabs;
			draggedIndex = index;
			hasChanges = true;
		}
	}

	function handleDragLeave() {
		dragOverIndex = null;
	}

	function handleDrop(e: DragEvent, dropIndex: number) {
		e.preventDefault();
		draggedIndex = null;
		dragOverIndex = null;
	}

	function handleDragEnd() {
		draggedIndex = null;
		dragOverIndex = null;
		isPressing = false;
		pressingIndex = null;
	}

	// Touch/Mouse press handlers for animation
	function handlePressStart(index: number) {
		isPressing = true;
		pressingIndex = index;
	}

	function handlePressEnd() {
		setTimeout(() => {
			isPressing = false;
			pressingIndex = null;
		}, 150);
	}

	function handleReset() {
		const defaultTabs: TabConfig[] = [
			{ id: 'list', label: 'list', icon: 'List', enabled: true },
			{ id: 'calendar', label: 'calendar', icon: 'CalendarDays', enabled: true },
			{ id: 'kanban', label: 'kanban', icon: 'Columns3', enabled: true },
			{ id: 'table', label: 'table', icon: 'Table', enabled: true },
			{ id: 'gantt', label: 'gantt', icon: 'GanttChart', enabled: true },
			{ id: 'workload', label: 'workload', icon: 'UsersRound', enabled: true }
		];
		editingTabs = defaultTabs.filter((tab) => !hiddenTabIds.includes(tab.id));
		hasChanges = true;
	}

	function handleSave() {
		const hiddenTabs = $tabSettings.filter((tab) => hiddenTabIds.includes(tab.id));
		tabSettings.set([...editingTabs, ...hiddenTabs]);
		hasChanges = false;
		dispatch('save');
	}

	function handleCancel() {
		// Restore from store
		const current = visibleStoreTabs;
		editingTabs = [...current];
		hasChanges = false;
		dispatch('close');
	}
</script>

<div class="w-72 overflow-hidden rounded-xl border border-gray-700/80 bg-gray-950 p-3 text-gray-100 shadow-2xl">
	<div class="flex items-center justify-between mb-3">
		<h3 class="font-semibold text-white flex items-center gap-2">
			<Settings2 size={18} />
			{$_('tabSettings__title')}
		</h3>
		<button
			on:click={handleCancel}
			class="text-gray-400 hover:text-white p-1 rounded-lg hover:bg-white/5 transition-colors"
		>
			<X size={18} />
		</button>
	</div>

	<p class="text-sm text-gray-400 mb-3">
        <!-- Hack to replace <icon /> since simple-i18n doesn't support components in strings easily -->
		<span>{$_('tabSettings__subtitle').split('<icon />')[0]}</span>
        <GripVertical size={14} class="inline" />
        <span>{$_('tabSettings__subtitle').split('<icon />')[1]}</span>
	</p>

	<div class="space-y-2 mb-3">
		{#each editingTabs as tab, index (tab.id)}
			<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
			<div
				animate:flip={{ duration: 250, easing: quintOut }}
				draggable={true}
				on:dragstart={() => handleDragStart(index)}
				on:dragover={(e) => handleDragOver(e, index)}
				on:dragleave={handleDragLeave}
				on:drop={(e) => handleDrop(e, index)}
				on:dragend={handleDragEnd}
				on:mousedown={() => handlePressStart(index)}
				on:mouseup={handlePressEnd}
				on:mouseleave={handlePressEnd}
				on:touchstart={() => handlePressStart(index)}
				on:touchend={handlePressEnd}
				role="listitem"
				class="tab-item flex items-center gap-2 rounded-lg border px-3 py-2 cursor-move select-none transition-colors {draggedIndex === index ? 'dragging' : 'border-gray-700 bg-white/[0.03] hover:bg-white/5'} {isPressing && pressingIndex === index ? 'pressing' : ''} {tab.enabled ? 'opacity-100' : 'opacity-45'}"
			>
				<div class="drag-handle text-gray-500 hover:text-gray-300 transition-colors">
					<GripVertical size={16} />
				</div>
				
				<svelte:component this={getIcon(tab.icon)} size={17} class="text-gray-300" />
				
				<span class="flex-1 font-medium text-gray-200">
					{$_(`tabs__${tab.id}`)}
				</span>
				
				<div class="flex items-center gap-1">
					<button
						on:click={() => toggleTabEnabled(tab.id)}
						class="p-1.5 rounded-md border transition-colors {tab.enabled ? 'text-emerald-300 bg-emerald-600/10 border-emerald-700/40' : 'text-gray-400 bg-white/[0.03] border-gray-700'}"
						title={tab.enabled ? 'Hide tab' : 'Show tab'}
						aria-label={tab.enabled ? 'Hide tab' : 'Show tab'}
					>
						{#if tab.enabled}
							<Eye size={14} />
						{:else}
							<EyeOff size={14} />
						{/if}
					</button>
					<button
						on:click={() => moveUp(index)}
						disabled={index === 0}
						class="p-1.5 text-gray-400 hover:text-white hover:bg-white/5 rounded disabled:opacity-30 disabled:cursor-not-allowed transition-all"
						title={$_('tabSettings__move_up')}
					>
						<ChevronUp size={16} />
					</button>
					<button
						on:click={() => moveDown(index)}
						disabled={index === editingTabs.length - 1}
						class="p-1.5 text-gray-400 hover:text-white hover:bg-white/5 rounded disabled:opacity-30 disabled:cursor-not-allowed transition-all"
						title={$_('tabSettings__move_down')}
					>
						<ChevronDown size={16} />
					</button>
				</div>
			</div>
		{/each}
	</div>

	<div class="flex items-center gap-2 pt-3 border-t border-gray-800">
		<button
			on:click={handleReset}
			class="px-2.5 py-1.5 text-sm font-medium text-gray-300 hover:text-white hover:bg-white/5 rounded-lg flex items-center gap-1.5 transition-all"
		>
			<RotateCcw size={16} />
			{$_('tabSettings__btn_reset')}
		</button>
		
		<div class="flex-1"></div>
		
		<button
			on:click={handleCancel}
			class="px-2.5 py-1.5 text-sm font-medium text-gray-300 hover:text-white hover:bg-white/5 rounded-lg transition-all"
		>
			{$_('tabSettings__btn_cancel')}
		</button>
		
		<button
			on:click={handleSave}
			disabled={!hasChanges}
			class="px-2.5 py-1.5 text-sm font-medium bg-primary text-white rounded-lg hover:bg-primary-dark disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5 transition-all"
		>
			<Check size={16} />
			{$_('tabSettings__btn_save')}
		</button>
	</div>
</div>

<style>
	.tab-item {
		transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.15s ease;
	}
	
	.tab-item:hover {
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
	}
	
	.tab-item.pressing {
		transform: scale(0.97);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
	}
	
	.tab-item.dragging {
		opacity: 0.7;
		border-color: #6366F1;
		border-style: dashed;
		box-shadow: 0 8px 24px rgba(99, 102, 241, 0.25);
		background: linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(99, 102, 241, 0.05) 100%);
		z-index: 10;
	}
	
	.drag-handle {
		transition: transform 0.2s ease, color 0.2s ease;
	}
	
	.tab-item:hover .drag-handle {
		transform: scale(1.2);
		color: #6366F1;
	}
	
	.tab-item.dragging .drag-handle {
		transform: scale(1.3);
		color: #6366F1;
	}
	
</style>
