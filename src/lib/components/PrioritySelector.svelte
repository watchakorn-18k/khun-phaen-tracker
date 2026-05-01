<script lang="ts">
	import SearchableSelect from "$lib/components/SearchableSelect.svelte";
	import {
		Minus,
		AlertTriangle,
		SignalHigh,
		SignalMedium,
		SignalLow,
	} from "lucide-svelte";
	import { _ } from "svelte-i18n";
	import { createEventDispatcher } from "svelte";

	export let value: string = "none";
	export let minimal: boolean = true;
	export let id: string = "priority";

	const dispatch = createEventDispatcher<{
		select: string;
	}>();

	const priorityOptions = [
		{ value: "none", label: "No priority", icon: Minus, iconClass: "text-gray-400", pillClass: "" },
		{ value: "urgent", label: "Urgent", icon: AlertTriangle, iconClass: "text-white", pillClass: "bg-orange-500 text-white px-2 py-0.5 rounded shadow-sm" },
		{ value: "high", label: "High", icon: SignalHigh, iconClass: "text-white", pillClass: "bg-orange-600 text-white px-2 py-0.5 rounded shadow-sm" },
		{ value: "medium", label: "Medium", icon: SignalMedium, iconClass: "text-orange-400", pillClass: "bg-orange-900/40 text-orange-400 px-2 py-0.5 rounded" },
		{ value: "low", label: "Low", icon: SignalLow, iconClass: "text-orange-500", pillClass: "bg-orange-900/20 text-orange-500 px-2 py-0.5 rounded" },
	];
</script>

<SearchableSelect
	{id}
	bind:value
	options={priorityOptions}
	showSearch={false}
	{minimal}
	on:select={(e) => dispatch("select", e.detail)}
/>
