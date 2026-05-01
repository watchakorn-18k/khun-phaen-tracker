<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { fade } from 'svelte/transition';
	import type { Sprint } from '$lib/stores/sprintStore';

	export let sprints: Sprint[] = [];
	export let value: string | number | 'all' | null = 'all';
	export let id: string = 'sprint';
	export let formMode: boolean = false; // When true, hides 'all' option and changes labels for form usage

	let isOpen = false;
	let searchQuery = '';
	let dropdownRef: HTMLDivElement;
	let searchInputRef: HTMLInputElement;


	// Sort sprints by date (newest first), then by status
	$: sprintsWithId = sprints.filter((s): s is Sprint & { id: string | number } => s.id !== undefined);
	$: sortedSprints = [...sprintsWithId].sort((a, b) => {
		// Active sprint first
		if (a.status === 'active' && b.status !== 'active') return -1;
		if (b.status === 'active' && a.status !== 'active') return 1;
		// Then by start date (newest first)
		return new Date(b.start_date).getTime() - new Date(a.start_date).getTime();
	});

	// Filter by search query
	$: filteredSprints = searchQuery.trim()
		? sortedSprints.filter(s => 
			s.name.toLowerCase().includes(searchQuery.toLowerCase())
		)
		: sortedSprints;

	// Show all filtered sprints
	$: displaySprints = filteredSprints;
	$: hasMore = false;

	// Get selected sprint name
	$: selectedLabel = getSelectedLabel(value, sprints);

	function getSelectedLabel(val: string | number | 'all' | null, sprintList: Sprint[]): string {
		if (val === 'all') return formMode ? $_('page__filter_sprint_unspecified') : $_('page__filter_sprint_all');
		if (val === null) return formMode ? $_('page__filter_sprint_unspecified') : $_('page__filter_sprint_none');
		const sprint = sprintList.find(s => String(s.id) === String(val));
		if (!sprint) return formMode ? $_('page__filter_sprint_unspecified') : $_('page__filter_sprint_all');
		const statusText = sprint.status === 'active' ? ` (${$_('page__filter_sprint_status_active')})` : 
			sprint.status === 'completed' ? ` (${$_('page__filter_sprint_status_completed')})` : '';
		return sprint.name + statusText;
	}

	function selectSprint(sprintId: string | number | 'all' | null) {
		value = sprintId;
		isOpen = false;
		searchQuery = '';
	}

	function toggleDropdown() {
		isOpen = !isOpen;
		if (isOpen) {
			setTimeout(() => searchInputRef?.focus(), 0);
		}
	}

	function handleClickOutside(event: MouseEvent) {
		if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
			isOpen = false;
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			isOpen = false;
		}
	}
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeyDown} />

<div class="relative {isOpen ? 'z-[9000]' : 'z-auto'}" bind:this={dropdownRef}>
	<!-- Trigger Button -->
	<button
		type="button"
		{id}
		class="w-full h-8 px-2 bg-transparent hover:bg-white/5 border border-transparent hover:border-white/10 rounded-md transition-all text-left flex items-center justify-between group"
		on:click={toggleDropdown}
	>
		<div class="flex items-center gap-2 overflow-hidden">
			<div class="w-2 h-2 rounded-full {value === 'all' || value === null ? 'bg-gray-600' : 'bg-indigo-400 shadow-[0_0_8px_rgba(129,140,248,0.5)]'}"></div>
			<span class="truncate text-[13px] {value === 'all' || value === null ? 'text-gray-500' : 'text-gray-200 font-medium'}">
				{selectedLabel}
			</span>
		</div>
		<svg class="w-3.5 h-3.5 text-gray-500 group-hover:text-gray-300 shrink-0 ml-1 transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
		</svg>
	</button>

	<!-- Dropdown -->
	{#if isOpen}
		<div class="absolute z-[9000] w-full min-w-[200px] mt-1 bg-gray-900 border border-white/10 rounded-xl shadow-2xl overflow-hidden animate-dropdown-in" transition:fade={{ duration: 100 }}>
			<!-- Search Input -->
			<div class="p-2 border-b border-white/5">
				<div class="relative">
					<svg class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
					</svg>
					<input
						type="text"
						bind:this={searchInputRef}
						bind:value={searchQuery}
						placeholder={$_('page__filter_sprint_search_placeholder')}
						class="w-full h-8 pl-8 pr-3 text-[13px] bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:border-indigo-500/50 transition-all text-white placeholder:text-gray-600"
					/>
				</div>
			</div>

			<!-- Options -->
			<div class="overflow-y-auto max-h-64 py-1 custom-scrollbar">
				{#if !formMode}
					<button
						type="button"
						class="w-full px-3 py-1.5 text-left text-[13px] hover:bg-white/5 transition-colors flex items-center gap-2 {value === 'all' ? 'text-indigo-400 font-bold' : 'text-gray-400'}"
						on:click={() => selectSprint('all')}
					>
						<div class="w-1.5 h-1.5 rounded-full {value === 'all' ? 'bg-indigo-400' : 'bg-transparent border border-gray-600'}"></div>
						{$_('page__filter_sprint_all')}
					</button>
				{/if}
				<button
					type="button"
					class="w-full px-3 py-1.5 text-left text-[13px] hover:bg-white/5 transition-colors flex items-center gap-2 {value === null ? 'text-indigo-400 font-bold' : 'text-gray-400'}"
					on:click={() => selectSprint(null)}
				>
					<div class="w-1.5 h-1.5 rounded-full {value === null ? 'bg-indigo-400' : 'bg-transparent border border-gray-600'}"></div>
					{formMode ? $_('page__filter_sprint_unspecified') : $_('page__filter_sprint_none')}
				</button>

				{#if displaySprints.length > 0}
					<div class="h-[1px] bg-white/5 my-1 mx-2"></div>
					{#each displaySprints as sprint}
						<button
							type="button"
							class="w-full px-3 py-2 text-left text-[13px] hover:bg-white/5 transition-colors flex items-center gap-3 {String(value) === String(sprint.id) ? 'text-white font-bold bg-white/5' : 'text-gray-400'}"
							on:click={() => selectSprint(sprint.id)}
						>
							<div class="flex flex-col flex-1 overflow-hidden">
								<span class="truncate">{sprint.name}</span>
								<span class="text-[10px] opacity-50 uppercase tracking-tighter">
									{new Date(sprint.start_date).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })} - 
									{new Date(sprint.end_date).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}
								</span>
							</div>
							{#if sprint.status === 'active'}
								<span class="shrink-0 px-1.5 py-0.5 text-[10px] font-bold bg-green-500/10 text-green-400 border border-green-500/20 rounded uppercase tracking-wider">{$_('page__filter_sprint_status_active')}</span>
							{:else if sprint.status === 'completed'}
								<span class="shrink-0 px-1.5 py-0.5 text-[10px] font-bold bg-gray-500/10 text-gray-500 border border-gray-500/20 rounded uppercase tracking-wider">{$_('page__filter_sprint_status_completed')}</span>
							{/if}
						</button>
					{/each}
				{:else if searchQuery}
					<div class="px-4 py-4 text-[13px] text-gray-600 text-center italic">
						{$_('page__filter_sprint_not_found')}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	@keyframes dropdown-in {
		from { opacity: 0; transform: translateY(-4px) scale(0.98); }
		to { opacity: 1; transform: translateY(0) scale(1); }
	}
	.animate-dropdown-in {
		animation: dropdown-in 0.15s cubic-bezier(0.16, 1, 0.3, 1) forwards;
	}
	.custom-scrollbar::-webkit-scrollbar {
		width: 4px;
	}
	.custom-scrollbar::-webkit-scrollbar-track {
		background: transparent;
	}
	.custom-scrollbar::-webkit-scrollbar-thumb {
		background: rgba(255, 255, 255, 0.1);
		border-radius: 10px;
	}
	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background: rgba(255, 255, 255, 0.2);
	}
</style>
