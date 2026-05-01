<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import { Markdown } from '@tiptap/markdown';
	import Link from '@tiptap/extension-link';
	import Image from '@tiptap/extension-image';
	import Placeholder from '@tiptap/extension-placeholder';
	import BubbleMenu from './BubbleMenu.svelte';

	interface Props {
		value: string;
		placeholder?: string;
		className?: string;
		onchange?: (value: string) => void;
	}

	let { 
		value = $bindable(''), 
		placeholder = 'Write something...', 
		className = '', 
		onchange 
	}: Props = $props();

	let element = $state<HTMLElement | null>(null);
	let editor = $state<Editor | null>(null);

	onMount(() => {
		if (!element) return;
		
		editor = new Editor({
			element: element,
			extensions: [
				StarterKit,
				Markdown,
				Link.configure({
					openOnClick: false,
					HTMLAttributes: {
						class: 'text-primary underline cursor-pointer'
					}
				}),
				Image.configure({
					HTMLAttributes: {
						class: 'rounded-lg max-w-full my-4 cursor-zoom-in'
					}
				}),
				Placeholder.configure({
					placeholder: placeholder,
					emptyEditorClass: 'is-editor-empty'
				})
			],
			content: value,
			editorProps: {
				attributes: {
					class: `rich-text-editor outline-none min-h-[150px] py-2 ${className}`
				},
				handlePaste: (view, event) => {
					const items = event.clipboardData?.items;
					if (!items) return false;
					for (const item of items) {
						if (item.type.startsWith('image/')) {
							const file = item.getAsFile();
							if (!file) continue;
							const reader = new FileReader();
							reader.onload = (e) => {
								const dataUrl = e.target?.result as string;
								if (dataUrl) {
									view.dispatch(
										view.state.tr.replaceSelectionWith(
											view.state.schema.nodes.image.create({ src: dataUrl })
										)
									);
								}
							};
							reader.readAsDataURL(file);
							return true;
						}
					}
					return false;
				},
				handleDrop: (view, event) => {
					const files = event.dataTransfer?.files;
					if (!files) return false;
					for (const file of Array.from(files)) {
						if (file.type.startsWith('image/')) {
							const reader = new FileReader();
							reader.onload = (e) => {
								const dataUrl = e.target?.result as string;
								if (dataUrl) {
									const { schema } = view.state;
									const coordinates = view.posAtCoords({ left: event.clientX, top: event.clientY });
									if (coordinates) {
										view.dispatch(
											view.state.tr.insert(
												coordinates.pos,
												schema.nodes.image.create({ src: dataUrl })
											)
										);
									}
								}
							};
							reader.readAsDataURL(file);
							return true;
						}
					}
					return false;
				}
			},
			onUpdate: ({ editor }) => {
				const markdown = (editor as any).getMarkdown();
				if (markdown !== value) {
					value = markdown;
					onchange?.(markdown);
				}
			}
		});
	});

	onDestroy(() => {
		if (editor) {
			editor.destroy();
		}
	});

	$effect(() => {
		if (editor && (editor as any).getMarkdown() !== value) {
			editor.commands.setContent(value, { emitUpdate: false });
		}
	});
</script>

<div class="relative w-full">
	<div bind:this={element} class="prose prose-sm dark:prose-invert max-w-none"></div>
	{#if editor}
		<BubbleMenu {editor} />
	{/if}
</div>

<style>
	:global(.rich-text-editor) {
		font-family: inherit;
	}

	:global(.rich-text-editor p.is-editor-empty:first-child::before) {
		content: attr(data-placeholder);
		float: left;
		color: #9ca3af;
		pointer-events: none;
		height: 0;
	}

	:global(.dark .rich-text-editor p.is-editor-empty:first-child::before) {
		color: #4b5563;
	}

	:global(.rich-text-editor ul) {
		list-style-type: disc;
		padding-left: 1.5rem;
	}

	:global(.rich-text-editor ol) {
		list-style-type: decimal;
		padding-left: 1.5rem;
	}

	:global(.rich-text-editor blockquote) {
		border-left: 3px solid #e5e7eb;
		padding-left: 1rem;
		font-style: italic;
		color: #6b7280;
	}

	:global(.dark .rich-text-editor blockquote) {
		border-color: #374151;
		color: #9ca3af;
	}
</style>
