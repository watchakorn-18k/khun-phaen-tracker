<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	const dispatch = createEventDispatcher();
	export let id: string = 'select';
	export let value: string | number | null = 'all';
	export let options: Array<{ value: string | number | null; label: string; badge?: boolean; badgeColor?: string; disabled?: boolean; icon?: any }> = [];
	export let placeholder: string = 'ค้นหา...';
	export let emptyText: string = 'ไม่พบรายการ';
	export let showSearch: boolean = true;
	export let maxDisplay: number = 8;
	export let minimal: boolean = false;

	let isOpen = false;
	let searchQuery = '';
	let dropdownRef: HTMLDivElement;
	let searchInputRef: HTMLInputElement;


	// Filter by search query
	$: filteredOptions = searchQuery.trim()
		? options.filter(opt => 
			opt.label.toLowerCase().includes(searchQuery.toLowerCase())
		)
		: options;

	// Show all filtered options (remove slice to ensure "Me" is always visible at the top)
	$: displayOptions = filteredOptions;
	$: hasMore = false;
	const isSameValue = (a: string | number | null, b: string | number | null) => {
		if (a === null || b === null) return a === b;
		return String(a) === String(b);
	};

	// Get selected label
	$: selectedLabel = options.find(opt => isSameValue(opt.value, value))?.label || 'ทั้งหมด';
	$: selectedBadge = options.find(opt => isSameValue(opt.value, value))?.badge;
	$: selectedBadgeColor = options.find(opt => isSameValue(opt.value, value))?.badgeColor;
	$: selectedIcon = options.find(opt => isSameValue(opt.value, value))?.icon;

	function selectOption(optionValue: string | number | null) {
		value = optionValue;
		isOpen = false;
		searchQuery = '';
		dispatch('select', optionValue);
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
		class="property-trigger-btn w-full {minimal ? 'h-8 px-2.5 text-[13px] border-transparent bg-transparent hover:bg-white/5' : 'h-10 px-3 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800'} rounded-lg outline-none text-left flex items-center justify-between transition-all"
		on:click={toggleDropdown}
	>
		<span class="truncate flex items-center gap-2">
			{#if selectedIcon}
				<span class="shrink-0 flex items-center justify-center">
					<svelte:component this={selectedIcon} size={14} class="shrink-0" />
				</span>
			{/if}
			<span class={String(value) === 'all' || value === null || value === '' ? 'text-gray-500 dark:text-gray-400' : 'text-gray-900 dark:text-gray-100'}>
				{selectedLabel}
			</span>
		</span>
		{#if !minimal}
			<svg class="w-4 h-4 text-gray-400 shrink-0 ml-2 transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		{/if}
	</button>

	<!-- Dropdown -->
	{#if isOpen}
		<div class="absolute z-[9000] w-full min-w-[180px] mt-1 bg-gray-900 border border-white/10 rounded-xl shadow-2xl max-h-80 overflow-hidden" transition:fade={{ duration: 100 }}>
			<!-- Search Input -->
			{#if showSearch && options.length > maxDisplay}
				<div class="p-2 border-b border-gray-200 dark:border-gray-700">
					<div class="relative">
						<svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
						</svg>
						<input
							type="text"
							bind:this={searchInputRef}
							bind:value={searchQuery}
							placeholder={placeholder}
							class="w-full h-8 pl-9 pr-3 text-[13px] border border-white/10 rounded-lg focus:ring-1 focus:ring-white/20 outline-none bg-white/5 text-gray-200"
						/>
					</div>
				</div>
			{/if}

			<!-- Options -->
			<div class="overflow-y-auto max-h-60">
				{#if displayOptions.length === 0}
					<div class="px-4 py-3 text-sm text-gray-500 text-center">
						{emptyText}
					</div>
				{:else}
					{#each displayOptions as option}
						{#if option.disabled}
							<div class="px-4 py-1.5 text-xs text-gray-400 dark:text-gray-500 font-medium text-center uppercase tracking-wider border-t border-b border-gray-100 dark:border-gray-700">
								{option.label}
							</div>
						{:else}
							<button
								type="button"
								class="w-full px-4 py-2.5 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2 transition-colors {isSameValue(value, option.value) ? 'bg-primary/10 text-primary font-medium' : 'text-gray-900 dark:text-gray-100'}"
								on:click={() => selectOption(option.value)}
							>
								{#if option.icon}
									<span class="shrink-0 flex items-center justify-center">
										<svelte:component this={option.icon} size={14} class="shrink-0" />
									</span>
								{:else}
									{#if option.badge}
										{#if option.badgeColor?.startsWith('#')}
											<span class="shrink-0 w-2 h-2 rounded-full" style="background-color: {option.badgeColor}"></span>
										{:else}
											<span class="shrink-0 w-2 h-2 rounded-full {option.badgeColor || 'bg-gray-400'}"></span>
										{/if}
									{/if}
								{/if}
								<span class="truncate flex-1 text-[13px]">{option.label}</span>
								{#if isSameValue(value, option.value)}
									<svg class="w-3.5 h-3.5 shrink-0 opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
									</svg>
								{/if}
							</button>
						{/if}
					{/each}

					{#if hasMore}
						<div class="px-4 py-2 text-xs text-gray-500 text-center border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-700/50">
							พิมพ์เพื่อค้นหารายการอื่นๆ ({filteredOptions.length - maxDisplay} รายการ)
						</div>
					{/if}
				{/if}
			</div>
		</div>
	{/if}
</div>

<script lang="ts" context="module">
	import { fade } from 'svelte/transition';
</script>
