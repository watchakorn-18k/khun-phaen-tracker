<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import Placeholder from '@tiptap/extension-placeholder';

	interface Props {
		value: string;
		placeholder?: string;
		className?: string;
		onchange?: (value: string) => void;
		onblur?: (value: string) => void;
	}

	let { 
		value = $bindable(''), 
		placeholder = 'Title', 
		className = '', 
		onchange,
		onblur
	}: Props = $props();

	let element = $state<HTMLElement | null>(null);
	let editor = $state<Editor | null>(null);

	onMount(() => {
		if (!element) return;

		editor = new Editor({
			element: element,
			extensions: [
				StarterKit.configure({
					heading: false,
					codeBlock: false,
					blockquote: false,
					bulletList: false,
					orderedList: false,
					listItem: false,
					horizontalRule: false,
				}),
				Placeholder.configure({
					placeholder: placeholder,
				})
			],
			content: value,
			editorProps: {
				attributes: {
					class: `title-editor outline-none ${className}`
				},
				handleKeyDown: (view, event) => {
					if (event.key === 'Enter') {
						event.preventDefault();
						onblur?.(editor?.getText() ?? '');
						return true;
					}
					return false;
				}
			},
			onUpdate: ({ editor }) => {
				const text = editor.getText();
				if (text !== value) {
					value = text;
					onchange?.(text);
				}
			},
			onBlur: ({ editor }) => {
				onblur?.(editor.getText());
			}
		});
	});

	onDestroy(() => {
		if (editor) {
			editor.destroy();
		}
	});

	$effect(() => {
		if (editor && editor.getText() !== value) {
			editor.commands.setContent(value, { emitUpdate: false });
		}
	});

	export function focus() {
		editor?.commands.focus();
	}
</script>

<div bind:this={element} class="w-full"></div>

<style>
	:global(.title-editor p) {
		margin: 0;
	}
</style>
