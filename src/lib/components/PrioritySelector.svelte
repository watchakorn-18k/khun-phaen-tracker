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
		{ value: "none", label: "No priority", icon: Minus },
		{ value: "urgent", label: "Urgent", icon: AlertTriangle },
		{ value: "high", label: "High", icon: SignalHigh },
		{ value: "medium", label: "Medium", icon: SignalMedium },
		{ value: "low", label: "Low", icon: SignalLow },
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
