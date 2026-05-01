<script lang="ts">
	import { computePosition, offset, flip, shift, hide, autoUpdate } from '@floating-ui/dom';
	import type { Editor } from '@tiptap/core';
	import { posToDOMRect } from '@tiptap/core';
	import { NodeSelection } from '@tiptap/pm/state';
	import { 
		Bold, Italic, Strikethrough, Code, Link2, List, ListOrdered, Quote, 
		ChevronDown, Check, X, Unlink, Type, Heading1, Heading2, Heading3 
	} from 'lucide-svelte';
	import { _ } from 'svelte-i18n';

	interface Props {
		editor: Editor;
	}

	let { editor }: Props = $props();

	let visible = $state(false);
	let mode = $state<'toolbar' | 'link-edit'>('toolbar');
	let url = $state('');
	let floatingEl = $state<HTMLDivElement | null>(null);

	// Format state
	let fmt = $state({
		bold: false,
		italic: false,
		strike: false,
		code: false,
		link: false,
		blockquote: false,
		bulletList: false,
		orderedList: false,
		h1: false,
		h2: false,
		h3: false
	});

	function shouldShow(editor: Editor): boolean {
		if (!editor.isEditable) return false;
		const { selection } = editor.state;
		if (selection.empty) return false;
		const { from, to } = selection;
		if (!editor.state.doc.textBetween(from, to).trim().length) return false;
		if (selection instanceof NodeSelection) return false;
		const resolvedFrom = editor.state.doc.resolve(from);
		if (resolvedFrom.parent.type.name === 'codeBlock') return false;
		return true;
	}

	function updateFormat() {
		fmt = {
			bold: editor.isActive('bold'),
			italic: editor.isActive('italic'),
			strike: editor.isActive('strike'),
			code: editor.isActive('code'),
			link: editor.isActive('link'),
			blockquote: editor.isActive('blockquote'),
			bulletList: editor.isActive('bulletList'),
			orderedList: editor.isActive('orderedList'),
			h1: editor.isActive('heading', { level: 1 }),
			h2: editor.isActive('heading', { level: 2 }),
			h3: editor.isActive('heading', { level: 3 })
		};
	}

	$effect(() => {
		const onTransaction = () => {
			visible = shouldShow(editor);
			updateFormat();
		};

		const onSelectionUpdate = () => {
			mode = 'toolbar';
		};

		const onBlur = () => {
			setTimeout(() => {
				if (editor.isDestroyed) return;
				if (floatingEl?.contains(document.activeElement)) return;
				if (editor.view.hasFocus()) return;
				visible = false;
			}, 0);
		};

		editor.on('transaction', onTransaction);
		editor.on('selectionUpdate', onSelectionUpdate);
		editor.on('blur', onBlur);

		return () => {
			editor.off('transaction', onTransaction);
			editor.off('selectionUpdate', onSelectionUpdate);
			editor.off('blur', onBlur);
		};
	});

	$effect(() => {
		if (!visible || !floatingEl) return;

		const virtualRef = {
			getBoundingClientRect: () => {
				if (editor.isDestroyed) return new DOMRect();
				const { from, to } = editor.state.selection;
				return posToDOMRect(editor.view, from, to);
			},
			contextElement: editor.view.dom
		};

		const updatePosition = () => {
			if (!floatingEl) return;
			computePosition(virtualRef, floatingEl, {
				strategy: 'fixed',
				placement: 'top',
				middleware: [offset(8), flip(), shift({ padding: 8 }), hide()]
			}).then(({ x, y, middlewareData }) => {
				if (!floatingEl) return;
				const hidden = middlewareData.hide?.referenceHidden;
				floatingEl.style.visibility = hidden ? 'hidden' : 'visible';
				floatingEl.style.left = `${x}px`;
				floatingEl.style.top = `${y}px`;
			});
		};

		const cleanup = autoUpdate(virtualRef, floatingEl, updatePosition);
		return cleanup;
	});

	function toggleBold() { editor.chain().focus().toggleBold().run(); }
	function toggleItalic() { editor.chain().focus().toggleItalic().run(); }
	function toggleStrike() { editor.chain().focus().toggleStrike().run(); }
	function toggleCode() { editor.chain().focus().toggleCode().run(); }
	function toggleQuote() { editor.chain().focus().toggleBlockquote().run(); }
	function toggleBulletList() { editor.chain().focus().toggleBulletList().run(); }
	function toggleOrderedList() { editor.chain().focus().toggleOrderedList().run(); }

	function setHeading(level: number | null) {
		if (level === null) {
			editor.chain().focus().setParagraph().run();
		} else {
			editor.chain().focus().toggleHeading({ level: level as any }).run();
		}
		showHeadingMenu = false;
	}

	let showHeadingMenu = $state(false);
	let showListMenu = $state(false);

	function openLinkEdit() {
		const attrs = editor.getAttributes('link');
		url = attrs.href || '';
		mode = 'link-edit';
	}

	function applyLink() {
		let href = url.trim();
		if (href) {
			if (!/^[a-z][a-z0-9+.-]*:\/\//i.test(href) && !href.startsWith('mailto:')) {
				href = 'https://' + href;
			}
			editor.chain().focus().extendMarkRange('link').setLink({ href }).run();
		} else {
			editor.chain().focus().extendMarkRange('link').unsetLink().run();
		}
		mode = 'toolbar';
		editor.commands.focus();
	}

	function removeLink() {
		editor.chain().focus().extendMarkRange('link').unsetLink().run();
		mode = 'toolbar';
		editor.commands.focus();
	}

	function cancelLink() {
		mode = 'toolbar';
		editor.commands.focus();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			applyLink();
		}
		if (e.key === 'Escape') {
			e.preventDefault();
			cancelLink();
		}
	}
</script>

<div
	bind:this={floatingEl}
	class="bubble-menu-container"
	style:visibility={visible ? 'visible' : 'hidden'}
	onmousedown={(e) => e.preventDefault()}
>
	{#if mode === 'link-edit'}
		<div class="link-edit-bar">
			<input
				type="text"
				bind:value={url}
				onkeydown={handleKeydown}
				placeholder="https://..."
				class="link-input"
			/>
			<button onclick={applyLink} class="icon-btn success">
				<Check size={14} />
			</button>
			{#if editor.isActive('link')}
				<button onclick={removeLink} class="icon-btn danger">
					<Unlink size={14} />
				</button>
			{/if}
			<button onclick={cancelLink} class="icon-btn">
				<X size={14} />
			</button>
		</div>
	{:else}
		<div class="toolbar">
			<button 
				onclick={toggleBold} 
				class="toolbar-btn" 
				class:active={fmt.bold}
				title="Bold (Ctrl+B)"
			>
				<Bold size={14} />
			</button>
			<button 
				onclick={toggleItalic} 
				class="toolbar-btn" 
				class:active={fmt.italic}
				title="Italic (Ctrl+I)"
			>
				<Italic size={14} />
			</button>
			<button 
				onclick={toggleStrike} 
				class="toolbar-btn" 
				class:active={fmt.strike}
				title="Strikethrough"
			>
				<Strikethrough size={14} />
			</button>
			<button 
				onclick={toggleCode} 
				class="toolbar-btn" 
				class:active={fmt.code}
				title="Code (Ctrl+E)"
			>
				<Code size={14} />
			</button>
			
			<div class="divider"></div>
			
			<button 
				onclick={openLinkEdit} 
				class="toolbar-btn" 
				class:active={fmt.link}
				title="Link"
			>
				<Link2 size={14} />
			</button>
			
			<div class="divider"></div>
			
			<div class="dropdown-wrapper">
				<button 
					onclick={() => showHeadingMenu = !showHeadingMenu} 
					class="dropdown-trigger"
					class:active={fmt.h1 || fmt.h2 || fmt.h3}
				>
					<span class="text-[11px] font-bold">
						{fmt.h1 ? 'H1' : fmt.h2 ? 'H2' : fmt.h3 ? 'H3' : 'Text'}
					</span>
					<ChevronDown size={10} />
				</button>
				{#if showHeadingMenu}
					<div class="dropdown-menu">
						<button onclick={() => setHeading(null)} class="menu-item" class:active={!fmt.h1 && !fmt.h2 && !fmt.h3}>
							<Type size={14} /> Normal Text
						</button>
						<button onclick={() => setHeading(1)} class="menu-item" class:active={fmt.h1}>
							<Heading1 size={14} /> Heading 1
						</button>
						<button onclick={() => setHeading(2)} class="menu-item" class:active={fmt.h2}>
							<Heading2 size={14} /> Heading 2
						</button>
						<button onclick={() => setHeading(3)} class="menu-item" class:active={fmt.h3}>
							<Heading3 size={14} /> Heading 3
						</button>
					</div>
				{/if}
			</div>

			<div class="dropdown-wrapper">
				<button 
					onclick={() => showListMenu = !showListMenu} 
					class="toolbar-btn"
					class:active={fmt.bulletList || fmt.orderedList}
				>
					<List size={14} />
					<ChevronDown size={10} />
				</button>
				{#if showListMenu}
					<div class="dropdown-menu">
						<button onclick={() => { toggleBulletList(); showListMenu = false; }} class="menu-item" class:active={fmt.bulletList}>
							<List size={14} /> Bullet List
						</button>
						<button onclick={() => { toggleOrderedList(); showListMenu = false; }} class="menu-item" class:active={fmt.orderedList}>
							<ListOrdered size={14} /> Ordered List
						</button>
					</div>
				{/if}
			</div>

			<button 
				onclick={toggleQuote} 
				class="toolbar-btn" 
				class:active={fmt.blockquote}
				title="Quote"
			>
				<Quote size={14} />
			</button>
		</div>
	{/if}
</div>

<style>
	.bubble-menu-container {
		position: fixed;
		z-index: 1000;
		width: max-content;
		pointer-events: auto;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: 2px;
		background: #1c1c1c;
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 8px;
		padding: 4px;
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5), 0 4px 6px -2px rgba(0, 0, 0, 0.2);
	}

	.toolbar-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: 6px;
		color: #a1a1aa;
		transition: all 0.2s;
		border: none;
		background: transparent;
		cursor: pointer;
	}

	.toolbar-btn:hover {
		background: rgba(255, 255, 255, 0.1);
		color: #ffffff;
	}

	.toolbar-btn.active {
		background: rgba(255, 255, 255, 0.15);
		color: #ffffff;
	}

	.divider {
		width: 1px;
		height: 16px;
		background: rgba(255, 255, 255, 0.1);
		margin: 0 4px;
	}

	.dropdown-wrapper {
		position: relative;
	}

	.dropdown-trigger {
		display: flex;
		align-items: center;
		gap: 4px;
		height: 28px;
		padding: 0 8px;
		border-radius: 6px;
		color: #a1a1aa;
		background: transparent;
		border: none;
		cursor: pointer;
		transition: all 0.2s;
	}

	.dropdown-trigger:hover {
		background: rgba(255, 255, 255, 0.1);
		color: #ffffff;
	}

	.dropdown-trigger.active {
		color: #ffffff;
	}

	.dropdown-menu {
		position: absolute;
		top: calc(100% + 8px);
		left: 0;
		background: #1c1c1c;
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 8px;
		padding: 4px;
		min-width: 140px;
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
	}

	.menu-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 8px;
		border-radius: 6px;
		color: #a1a1aa;
		text-align: left;
		font-size: 12px;
		background: transparent;
		border: none;
		cursor: pointer;
		transition: all 0.2s;
	}

	.menu-item:hover {
		background: rgba(255, 255, 255, 0.1);
		color: #ffffff;
	}

	.menu-item.active {
		color: #ffffff;
		background: rgba(255, 255, 255, 0.05);
	}

	.link-edit-bar {
		display: flex;
		align-items: center;
		gap: 4px;
		background: #1c1c1c;
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 8px;
		padding: 4px 8px;
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
		min-width: 240px;
	}

	.link-input {
		flex: 1;
		background: transparent;
		border: none;
		color: #ffffff;
		font-size: 12px;
		outline: none;
		height: 28px;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 4px;
		color: #a1a1aa;
		background: transparent;
		border: none;
		cursor: pointer;
		transition: all 0.2s;
	}

	.icon-btn:hover {
		background: rgba(255, 255, 255, 0.1);
		color: #ffffff;
	}

	.icon-btn.success:hover {
		color: #22c55e;
	}

	.icon-btn.danger:hover {
		color: #ef4444;
	}
</style>
