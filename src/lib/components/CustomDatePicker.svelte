<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { ChevronLeft, ChevronRight, Calendar } from 'lucide-svelte';
	import { _, locale } from 'svelte-i18n';

	const dispatch = createEventDispatcher<{ select: string }>();

	export let value = '';
	export let placeholder = 'เลือกวันที่...';
	export let id = 'date-picker';
	export let minimal = false;
	export let dropdownPosition: 'top' | 'bottom' = 'bottom';

	let triggerEl: HTMLButtonElement;
	let dropdownTop = 0;
	let dropdownLeft = 0;
	let dropdownWidth = 256; // w-64 = 16rem = 256px

	let isOpen = false;
	let currentMonth = new Date().getMonth();
	let currentYear = new Date().getFullYear();
	let viewMode: 'days' | 'months' | 'years' = 'days';

	$: monthNames = $_('calendarView__months') as unknown as string[];
	$: shortMonthNames = $_('calendarView__short_months') as unknown as string[];
	$: dayNames = $locale === 'th' ? ['อา', 'จ', 'อ', 'พ', 'พฤ', 'ศ', 'ส'] : ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'];

	$: selectedDate = value ? parseDate(value) : null;
	$: displayValue = selectedDate 
		? `${selectedDate.getDate()} ${shortMonthNames[selectedDate.getMonth()]} ${selectedDate.getFullYear() + ($locale === 'th' ? 543 : 0)}`
		: placeholder;

	function parseDate(dateStr: string): Date | null {
		if (!dateStr) return null;
		const [year, month, day] = dateStr.split('-').map(Number);
		return new Date(year, month - 1, day);
	}

	function formatDate(date: Date): string {
		const year = date.getFullYear();
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const day = String(date.getDate()).padStart(2, '0');
		return `${year}-${month}-${day}`;
	}

	function getDaysInMonth(month: number, year: number): number {
		return new Date(year, month + 1, 0).getDate();
	}

	function getFirstDayOfMonth(month: number, year: number): number {
		return new Date(year, month, 1).getDay();
	}

	function getCalendarDays(): Array<{ date: number; month: number; year: number; isCurrentMonth: boolean }> {
		const days = [];
		const daysInMonth = getDaysInMonth(currentMonth, currentYear);
		const firstDay = getFirstDayOfMonth(currentMonth, currentYear);
		
		// Previous month days
		const prevMonth = currentMonth === 0 ? 11 : currentMonth - 1;
		const prevYear = currentMonth === 0 ? currentYear - 1 : currentYear;
		const daysInPrevMonth = getDaysInMonth(prevMonth, prevYear);
		
		for (let i = firstDay - 1; i >= 0; i--) {
			days.push({
				date: daysInPrevMonth - i,
				month: prevMonth,
				year: prevYear,
				isCurrentMonth: false
			});
		}

		// Current month days
		for (let i = 1; i <= daysInMonth; i++) {
			days.push({
				date: i,
				month: currentMonth,
				year: currentYear,
				isCurrentMonth: true
			});
		}

		// Next month days
		const remainingSlots = 42 - days.length; // 6 rows x 7 columns
		const nextMonth = currentMonth === 11 ? 0 : currentMonth + 1;
		const nextYear = currentMonth === 11 ? currentYear + 1 : currentYear;

		for (let i = 1; i <= remainingSlots; i++) {
			days.push({
				date: i,
				month: nextMonth,
				year: nextYear,
				isCurrentMonth: false
			});
		}

		return days;
	}

	function selectDate(day: { date: number; month: number; year: number }) {
		const date = new Date(day.year, day.month, day.date);
		value = formatDate(date);
		isOpen = false;
		dispatch('select', value);
	}

	function isSelected(day: { date: number; month: number; year: number }): boolean {
		if (!selectedDate) return false;
		return day.date === selectedDate.getDate() && 
			   day.month === selectedDate.getMonth() && 
			   day.year === selectedDate.getFullYear();
	}

	function isToday(day: { date: number; month: number; year: number }): boolean {
		const today = new Date();
		return day.date === today.getDate() && 
			   day.month === today.getMonth() && 
			   day.year === today.getFullYear();
	}

	function previousMonth() {
		if (currentMonth === 0) {
			currentMonth = 11;
			currentYear--;
		} else {
			currentMonth--;
		}
	}

	function nextMonth() {
		if (currentMonth === 11) {
			currentMonth = 0;
			currentYear++;
		} else {
			currentMonth++;
		}
	}

	function showMonthSelector() {
		viewMode = 'months';
	}

	function showYearSelector() {
		viewMode = 'years';
	}

	function selectMonth(month: number) {
		currentMonth = month;
		viewMode = 'days';
	}

	function selectYear(year: number) {
		currentYear = year;
		viewMode = 'days';
	}

	function getYearRange(): number[] {
		const startYear = currentYear - 6;
		const years = [];
		for (let i = 0; i < 12; i++) {
			years.push(startYear + i);
		}
		return years;
	}

	function toggleDropdown() {
		if (!isOpen) {
			// Calculate position from button before opening
			const rect = triggerEl.getBoundingClientRect();
			if (dropdownPosition === 'top') {
				dropdownTop = rect.top - 8; // will use bottom offset via style
			} else {
				dropdownTop = rect.bottom + 4;
			}
			// Align right edge of dropdown to right edge of button
			dropdownLeft = Math.max(4, rect.right - dropdownWidth);
		}
		isOpen = !isOpen;
		if (isOpen && selectedDate) {
			currentMonth = selectedDate.getMonth();
			currentYear = selectedDate.getFullYear();
		}
		viewMode = 'days';
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.date-picker-container')) {
			isOpen = false;
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			isOpen = false;
		}
	}

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return {
			destroy() {
				if (node.parentNode) {
					node.parentNode.removeChild(node);
				}
			}
		};
	}
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeyDown} />

<div class="date-picker-container relative {isOpen ? 'z-[99999]' : 'z-auto'}">
	<!-- Trigger Button -->
	<button
		type="button"
		id={id}
		bind:this={triggerEl}
		class="property-trigger-btn w-full {minimal ? 'h-8 px-2.5 text-[13px] border-transparent bg-transparent hover:bg-white/5' : 'h-10 px-3 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700'} rounded-lg outline-none text-left flex items-center justify-between transition-all group"
		on:click={toggleDropdown}
	>
		<span class="truncate flex items-center gap-2">
			{#if !minimal}
				<Calendar size={16} class="text-gray-400 group-hover:text-primary transition-colors" />
			{/if}
			<span class={value ? 'text-gray-900 dark:text-gray-100' : 'text-gray-500 dark:text-gray-400'}>
				{displayValue}
			</span>
		</span>
		{#if !minimal}
			<svg 
				class="w-4 h-4 text-gray-400 flex-shrink-0 ml-2 transition-transform duration-300 {isOpen ? 'rotate-180' : ''}" 
				fill="none" 
				stroke="currentColor" 
				viewBox="0 0 24 24"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		{/if}
	</button>

	<!-- Dropdown -->
	{#if isOpen}
		<div 
			use:portal
			class="fixed z-[99999] w-64 bg-gray-900 border border-white/10 rounded-xl shadow-2xl overflow-hidden animate-dropdown-in"
			style="{dropdownPosition === 'top'
				? `bottom: ${window.innerHeight - dropdownTop}px; left: ${dropdownLeft}px;`
				: `top: ${dropdownTop}px; left: ${dropdownLeft}px;`}"
		>
			<!-- Header -->
			<div class="px-4 py-2 border-b border-white/5">
				<div class="flex items-center justify-between">
					{#if viewMode === 'days'}
						<button
							type="button"
							class="text-sm font-bold text-gray-200 hover:text-white transition-colors flex items-center gap-1"
							on:click={showMonthSelector}
						>
							{monthNames[currentMonth]}
							<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
							</svg>
						</button>
						<button
							type="button"
							class="text-sm font-bold text-gray-200 hover:text-white transition-colors"
							on:click={showYearSelector}
						>
							{currentYear + ($locale === 'th' ? 543 : 0)}
						</button>
					{:else if viewMode === 'months'}
						<button
							type="button"
							class="text-lg font-semibold text-gray-800 dark:text-white hover:text-primary transition-colors"
							on:click={() => viewMode = 'years'}
						>
							{currentYear + ($locale === 'th' ? 543 : 0)}
						</button>
						<span class="text-lg font-semibold text-gray-800 dark:text-white">{$locale === 'th' ? 'เลือกเดือน' : 'Select Month'}</span>
					{:else}
						<span class="text-lg font-semibold text-gray-800 dark:text-white">{$locale === 'th' ? 'เลือกปี' : 'Select Year'}</span>
					{/if}
					
					{#if viewMode === 'days'}
						<div class="flex gap-0.5">
							<button
								type="button"
								class="p-1.5 rounded-lg hover:bg-white/5 text-gray-400 hover:text-white transition-all"
								on:click={previousMonth}
							>
								<ChevronLeft size={16} />
							</button>
							<button
								type="button"
								class="p-1.5 rounded-lg hover:bg-white/5 text-gray-400 hover:text-white transition-all"
								on:click={nextMonth}
							>
								<ChevronRight size={16} />
							</button>
						</div>
					{/if}
				</div>
			</div>

			<!-- Days View -->
			{#if viewMode === 'days'}
				<div class="p-3">
					<!-- Day Headers -->
					<div class="grid grid-cols-7 gap-1 mb-2">
						{#each dayNames as day}
							<div class="text-center text-xs font-medium text-gray-500 dark:text-gray-400 py-1">
								{day}
							</div>
						{/each}
					</div>

					<!-- Calendar Grid -->
					<div class="grid grid-cols-7 gap-0.5">
						{#each getCalendarDays() as day}
							<button
								type="button"
								class="aspect-square rounded-lg text-[13px] font-medium transition-all duration-200 relative overflow-hidden
									{day.isCurrentMonth 
										? 'text-gray-300 hover:bg-white/5' 
										: 'text-gray-600'}
									{isSelected(day) 
										? 'bg-white/10 text-white shadow-sm' 
										: ''}
									{isToday(day) && !isSelected(day)
										? 'text-indigo-400 font-bold'
										: ''}"
								on:click={() => selectDate(day)}
							>
								<span class="relative z-10">{day.date}</span>
								{#if isToday(day) && isSelected(day)}
									<span class="absolute bottom-1 left-1/2 transform -translate-x-1/2 w-1 h-1 rounded-full bg-white/50"></span>
								{/if}
							</button>
						{/each}
					</div>
				</div>

				<!-- Quick Actions -->
				<div class="px-3 pb-3 pt-2 border-t border-white/5">
					<div class="flex gap-1.5">
						<button
							type="button"
							class="flex-1 py-1.5 px-2 text-[11px] font-bold text-gray-400 bg-white/5 rounded-lg hover:bg-white/10 hover:text-white transition-all"
							on:click={() => {
								const today = new Date();
								selectDate({ date: today.getDate(), month: today.getMonth(), year: today.getFullYear() });
							}}
						>
							{$locale === 'th' ? 'วันนี้' : 'Today'}
						</button>
						<button
							type="button"
							class="flex-1 py-1.5 px-2 text-[11px] font-bold text-gray-400 bg-white/5 rounded-lg hover:bg-white/10 hover:text-white transition-all"
							on:click={() => {
								const tomorrow = new Date();
								tomorrow.setDate(tomorrow.getDate() + 1);
								selectDate({ date: tomorrow.getDate(), month: tomorrow.getMonth(), year: tomorrow.getFullYear() });
							}}
						>
							{$locale === 'th' ? 'พรุ่งนี้' : 'Tomorrow'}
						</button>
						<button
							type="button"
							class="flex-1 py-1.5 px-2 text-[11px] font-bold text-gray-400 bg-white/5 rounded-lg hover:bg-white/10 hover:text-white transition-all"
							on:click={() => {
								const nextWeek = new Date();
								nextWeek.setDate(nextWeek.getDate() + 7);
								selectDate({ date: nextWeek.getDate(), month: nextWeek.getMonth(), year: nextWeek.getFullYear() });
							}}
						>
							{$locale === 'th' ? 'สัปดาห์หน้า' : 'Next Week'}
						</button>
					</div>
				</div>
			{/if}

			<!-- Months View -->
			{#if viewMode === 'months'}
				<div class="p-3">
					<div class="grid grid-cols-3 gap-2">
						{#each monthNames as monthName, index}
							<button
								type="button"
								class="py-3 px-2 rounded-lg text-sm font-medium transition-all duration-200
									{currentMonth === index 
										? 'bg-primary text-white shadow-md' 
										: 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}"
								on:click={() => selectMonth(index)}
							>
								{monthName}
							</button>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Years View -->
			{#if viewMode === 'years'}
				<div class="p-3">
					<div class="grid grid-cols-3 gap-2">
						{#each getYearRange() as year}
							<button
								type="button"
								class="py-3 px-2 rounded-lg text-sm font-medium transition-all duration-200
									{currentYear === year 
										? 'bg-primary text-white shadow-md' 
										: 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}"
								on:click={() => selectYear(year)}
							>
								{year + ($locale === 'th' ? 543 : 0)}
							</button>
						{/each}
					</div>
					<div class="flex justify-between mt-3 px-1">
						<button
							type="button"
							class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
							on:click={() => currentYear -= 12}
						>
							<ChevronLeft size={18} />
						</button>
						<button
							type="button"
							class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
							on:click={() => currentYear += 12}
						>
							<ChevronRight size={18} />
						</button>
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	@keyframes dropdown-in {
		from {
			opacity: 0;
			transform: translateY(-8px) scale(0.95);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	.animate-dropdown-in {
		animation: dropdown-in 0.2s cubic-bezier(0.16, 1, 0.3, 1);
	}
</style>
