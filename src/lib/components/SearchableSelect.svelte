<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	const dispatch = createEventDispatcher();
	export let id: string = 'select';
	export let value: string | number | null = 'all';
	export let options: Array<{ 
		value: string | number | null; 
		label: string; 
		badge?: boolean; 
		badgeColor?: string; 
		disabled?: boolean; 
		icon?: any; 
		iconClass?: string; 
		pillClass?: string;
		avatarUrl?: string;
		avatarColor?: string;
	}> = [];
	export let placeholder: string = 'ค้นหา...';
	export let emptyText: string = 'ไม่พบรายการ';
	export let showSearch: boolean = true;
	export let maxDisplay: number = 8;
	export let minimal: boolean = false;
	export let customClass: string = '';
	export let disabled: boolean = false;

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
	$: selectedOption = options.find(opt => isSameValue(opt.value, value));
	$: selectedLabel = selectedOption?.label || 'ทั้งหมด';
	$: selectedBadge = selectedOption?.badge;
	$: selectedBadgeColor = selectedOption?.badgeColor;
	$: selectedIcon = selectedOption?.icon;
	$: selectedIconClass = selectedOption?.iconClass;
	$: selectedAvatarUrl = selectedOption?.avatarUrl;
	$: selectedAvatarColor = selectedOption?.avatarColor;

	function selectOption(optionValue: string | number | null) {
		value = optionValue;
		isOpen = false;
		searchQuery = '';
		dispatch('select', optionValue);
	}

	function toggleDropdown() {
		if (disabled) return;
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
		class="property-trigger-btn w-full {minimal ? 'h-8 px-2.5 text-[13px] border-transparent bg-transparent hover:bg-white/5' : 'h-10 px-3 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800'} rounded-lg outline-none text-left flex items-center justify-between transition-all {disabled ? 'opacity-60 cursor-default' : ''} {customClass}"
		on:click={toggleDropdown}
		{disabled}
	>
		<span class="truncate flex items-center gap-2">
			{#if selectedAvatarUrl || selectedAvatarColor}
				<div 
					class="shrink-0 w-5 h-5 rounded-full overflow-hidden flex items-center justify-center text-[10px] font-bold text-white"
					style="background-color: {selectedAvatarColor || '#6366f1'}"
				>
					{#if selectedAvatarUrl}
						<img src={selectedAvatarUrl} alt={selectedLabel} class="w-full h-full object-cover" />
					{:else}
						{selectedLabel?.[0]?.toUpperCase() || '?'}
					{/if}
				</div>
			{:else if selectedIcon}
				<span class="shrink-0 flex items-center justify-center {selectedIconClass || 'text-gray-400'}">
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
	{#if isOpen && !disabled}
		<div class="absolute z-[9000] w-full min-w-[200px] mt-1 bg-white dark:bg-slate-900 border border-gray-200 dark:border-white/10 rounded-xl shadow-2xl max-h-80 overflow-hidden ring-1 ring-black/5" transition:fade={{ duration: 100 }}>
			<!-- Search Input -->
			{#if showSearch}
				<div class="border-b border-white/10">
					<input
						type="text"
						bind:this={searchInputRef}
						bind:value={searchQuery}
						placeholder={placeholder}
						class="w-full h-10 px-4 text-[13px] bg-transparent border-none outline-none text-gray-900 dark:text-gray-100 placeholder:text-gray-500 focus:ring-0"
					/>
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
							<div class="px-4 py-2 text-[11px] font-semibold text-gray-500 uppercase tracking-wider">
								{option.label}
							</div>
						{:else}
							<button
								type="button"
								class="w-full px-4 py-2 text-left text-[13px] hover:bg-gray-50 dark:hover:bg-white/5 flex items-center justify-between transition-colors {isSameValue(value, option.value) ? 'text-indigo-600 dark:text-white font-black' : 'text-gray-700 dark:text-gray-300'}"
								on:click={() => selectOption(option.value)}
							>
								<div class="flex items-center gap-2.5 {option.pillClass || ''}">
									{#if option.avatarUrl || option.avatarColor}
										<div 
											class="shrink-0 w-6 h-6 rounded-full overflow-hidden flex items-center justify-center text-[11px] font-bold text-white"
											style="background-color: {option.avatarColor || '#6366f1'}"
										>
											{#if option.avatarUrl}
												<img src={option.avatarUrl} alt={option.label} class="w-full h-full object-cover" />
											{:else}
												{option.label?.[0]?.toUpperCase() || '?'}
											{/if}
										</div>
									{:else if option.icon}
										<span class="shrink-0 flex items-center justify-center {option.iconClass || 'text-gray-400'}">
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
									<span>{option.label}</span>
								</div>
								{#if isSameValue(value, option.value)}
									<svg class="w-3.5 h-3.5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
