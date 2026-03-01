<script lang="ts">
	import { createEventDispatcher, onDestroy, onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { API_BASE_URL } from '$lib/apis';
import { createTaskComment, deleteTaskComment, getCommentImages, getTaskComments, updateTaskCommentText } from '$lib/db';
	import { user } from '$lib/stores/auth';
	import { taskDefaults } from '$lib/stores/taskDefaults';
	import type { Assignee, ChecklistItem, CommentImage, Project, Sprint, Task, TaskComment } from '$lib/types';
	import { CATEGORIES } from '$lib/types';
	import { _ } from 'svelte-i18n';
import { Calendar, Check, CheckCircle, ChevronLeft, ChevronRight, Copy, Download, Edit2, ExternalLink, FileText, Folder, GitBranch, GitPullRequest, Image as ImageIcon, Link, Maximize, MessageCircle, RotateCcw, RotateCw, Send, Tag, Trash2, X, ZoomIn, ZoomOut } from 'lucide-svelte';
	import AssigneeSelector from './AssigneeSelector.svelte';
	import BranchDialog from './BranchDialog.svelte';
	import ChecklistManager from './ChecklistManager.svelte';
	import CustomDatePicker from './CustomDatePicker.svelte';
	import MarkdownEditor from './MarkdownEditor.svelte';
	import Pagination from './Pagination.svelte';
	import SearchableSelect from './SearchableSelect.svelte';

	const dispatch = createEventDispatcher<{
		submit: Omit<Task, 'id' | 'created_at'>;
		cancel: void;
		close: void;
		addAssignee: { name: string; color: string };
		checklistUpdate: { checklist: ChecklistItem[] };
	}>();

	export let show = false;
	export let editingTask: Task | null = null;
	export let assignees: Assignee[] = [];
	export let projects: Project[] = [];
	export let sprints: Sprint[] = [];
	export let isOwner = true;

	let title = '';
	let project = '';
	let date = new Date().toISOString().split('T')[0];
	let end_date = '';
	let status: Task['status'] = 'todo';
	let category = 'งานหลัก';
	let notes = '';
	let assignee_ids: number[] = [];
	let assignee_id_to_add: number | null = null;
	let sprint_id: string | number | null = null;
	let checklist: ChecklistItem[] = [];
	let showBranchDialog = false;
	let formInitKey = 'closed';
	let copySuccess = false;

	let comments: TaskComment[] = [];
	let commentsLoading = false;
	let commentsError = '';
	let commentContent = '';
	let commentFiles: File[] = [];
	let commentSubmitting = false;
	let commentDeleting = false;
	let showDeleteCommentConfirm = false;
	let pendingDeleteComment: TaskComment | null = null;
	let commentsPage = 1;
	let commentsLimit = 10;
	let commentsTotal = 0;
	let editingCommentId: string | null = null;
	let editingCommentText = '';
	let commentUpdating = false;
	let lastCommentsKey = '';
	let imagePaginationByComment: Record<string, { page: number; limit: number; total: number; images: CommentImage[]; loading: boolean; error: string }> = {};
	let lightboxImages: { src: string; alt: string }[] = [];
	let lightboxSrc = '';
	let lightboxAlt = '';
	let lightboxOpen = false;
	let lightboxZoom = 1;
	let lightboxX = 0;
	let lightboxY = 0;
	let lightboxRotation = 0;
	let lightboxIndex = 0;
	let isLightboxDragging = false;
	let dragStartX = 0;
	let dragStartY = 0;
	let dragStartOffsetX = 0;
	let dragStartOffsetY = 0;
	let copyFeedback = '';

	$: activeSprint = sprints.find((s) => s.status === 'active');
	$: currentProjectRepoUrl = (() => {
		if (!project) return '';
		const matched = projects.find((p) => p.name === project);
		return matched?.repo_url || '';
	})();

	function initializeFormState() {
		if (editingTask) {
			title = editingTask.title || '';
			project = editingTask.project || '';
			date = editingTask.date || new Date().toISOString().split('T')[0];
			end_date = editingTask.end_date || '';
			status = editingTask.status || 'todo';
			category = editingTask.category || 'งานหลัก';
			notes = editingTask.notes || '';
			assignee_ids = (editingTask.assignee_ids || (editingTask.assignee_id ? [editingTask.assignee_id] : []))
				.map((id) => Number(id))
				.filter((id) => !Number.isNaN(id));
			sprint_id = editingTask.sprint_id || null;
			checklist = editingTask.checklist ? [...editingTask.checklist] : [];
		} else {
			title = '';
			project = $taskDefaults.project || '';
			date = new Date().toISOString().split('T')[0];
			end_date = '';
			status = 'todo';
			category = $taskDefaults.category || 'งานหลัก';
			notes = '';
			assignee_ids = $taskDefaults.assignee_id ? [$taskDefaults.assignee_id] : [];
			sprint_id = activeSprint?.id || null;
			checklist = [];
		}

		assignee_id_to_add = null;
		showBranchDialog = false;
		commentContent = '';
		commentFiles = [];
		comments = [];
		commentsError = '';
		commentsPage = 1;
		commentsTotal = 0;
		imagePaginationByComment = {};
		lastCommentsKey = '';
	}

	function getCommentImageUrl(fileKey: string): string {
		return `${API_BASE_URL.replace(/\/api$/, '')}/api/files/${fileKey}`;
	}

	async function loadCommentImages(comment: TaskComment) {
		if (!editingTask?.id || !comment.id) return;
		const key = String(comment.id);
		const current = imagePaginationByComment[key] || {
			page: 1,
			limit: 6,
			total: comment.images.length || 0,
			images: [],
			loading: false,
			error: ''
		};
		imagePaginationByComment[key] = { ...current, loading: true, error: '' };
		try {
			const response = await getCommentImages(editingTask.id, comment.id, {
				page: current.page,
				limit: current.limit
			});
			imagePaginationByComment[key] = {
				...current,
				images: response.images,
				page: response.page,
				limit: response.limit,
				total: response.total,
				loading: false,
				error: ''
			};
		} catch (error) {
			imagePaginationByComment[key] = {
				...current,
				loading: false,
				error: error instanceof Error ? error.message : 'Failed to load images'
			};
		}
	}

	async function loadComments(page = commentsPage) {
		if (!editingTask?.id) return;
		commentsLoading = true;
		commentsError = '';
		try {
			const response = await getTaskComments(editingTask.id, { page, limit: commentsLimit });
			comments = response.comments;
			commentsTotal = response.total;
			commentsPage = response.page;
			for (const comment of comments) {
				const key = String(comment.id || '');
				const prev = imagePaginationByComment[key];
				imagePaginationByComment[key] = {
					page: prev?.page || 1,
					limit: prev?.limit || 6,
					total: comment.images.length || 0,
					images: [],
					loading: false,
					error: ''
				};
				await loadCommentImages(comment);
			}
		} catch (error) {
			commentsError = error instanceof Error ? error.message : 'Failed to load comments';
		} finally {
			commentsLoading = false;
		}
	}

	function prevCommentImagesPage(comment: TaskComment) {
		if (!comment.id) return;
		const key = String(comment.id);
		const state = imagePaginationByComment[key];
		if (!state || state.page <= 1) return;
		imagePaginationByComment[key] = { ...state, page: state.page - 1 };
		void loadCommentImages(comment);
	}

	function nextCommentImagesPage(comment: TaskComment) {
		if (!comment.id) return;
		const key = String(comment.id);
		const state = imagePaginationByComment[key];
		if (!state) return;
		const totalPages = Math.max(1, Math.ceil(state.total / state.limit));
		if (state.page >= totalPages) return;
		imagePaginationByComment[key] = { ...state, page: state.page + 1 };
		void loadCommentImages(comment);
	}

	function handleCommentFileChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const files = Array.from(input.files || []).filter((file) => file.type.startsWith('image/'));
		commentFiles = files.slice(0, 10);
	}

	function removeCommentFile(index: number) {
		commentFiles = commentFiles.filter((_, i) => i !== index);
	}

	function formatCommentAuthor(comment: TaskComment): string {
		const uid = $user?.id || $user?.user_id;
		if (uid && comment.created_by === uid) {
			const profile = $user?.profile;
			const fullName = [profile?.first_name, profile?.last_name].filter(Boolean).join(' ').trim();
			return profile?.nickname || fullName || $user?.email?.split('@')[0] || 'Unknown';
		}
		return comment.created_by || 'Unknown';
	}

	function formatCommentTime(value?: string): string {
		if (!value) return '';
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return value;
		return new Intl.DateTimeFormat('th-TH', {
			year: 'numeric',
			month: 'short',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit'
		}).format(date);
	}

	function formatCommentRelativeTime(value?: string): string {
		if (!value) return '';
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return formatCommentTime(value);
		const diffMs = Date.now() - date.getTime();
		const diffSec = Math.max(0, Math.floor(diffMs / 1000));
		if (diffSec < 60) return 'just now';
		const diffMin = Math.floor(diffSec / 60);
		if (diffMin < 60) return `${diffMin}m ago`;
		const diffHr = Math.floor(diffMin / 60);
		if (diffHr < 24) return `${diffHr}h ago`;
		const diffDay = Math.floor(diffHr / 24);
		if (diffDay < 7) return `${diffDay}d ago`;
		return formatCommentTime(value);
	}

	function getAuthorInitials(comment: TaskComment): string {
		const name = formatCommentAuthor(comment).trim();
		if (!name) return 'U';
		return name
			.split(/\s+/)
			.map((p) => p[0])
			.join('')
			.slice(0, 2)
			.toUpperCase();
	}

	function getImageState(commentId: string | undefined) {
		return imagePaginationByComment[String(commentId || '')];
	}

	function openCommentImageLightbox(comment: TaskComment, imageIndex: number) {
		const images = getImageState(comment.id)?.images || [];
		if (images.length === 0) return;
		lightboxImages = images.map((img) => ({
			src: getCommentImageUrl(img.file_key),
			alt: img.filename || 'image'
		}));
		lightboxIndex = Math.min(Math.max(imageIndex, 0), lightboxImages.length - 1);
		lightboxSrc = lightboxImages[lightboxIndex].src;
		lightboxAlt = lightboxImages[lightboxIndex].alt;
		lightboxZoom = 1;
		lightboxX = 0;
		lightboxY = 0;
		lightboxRotation = 0;
		copyFeedback = '';
		lightboxOpen = true;
	}

	function closeLightbox() {
		lightboxOpen = false;
		isLightboxDragging = false;
	}

	function zoomIn() { lightboxZoom = Math.min(lightboxZoom + 0.25, 5); }
	function zoomOut() { lightboxZoom = Math.max(lightboxZoom - 0.25, 0.25); }
	function resetView() { lightboxZoom = 1; lightboxX = 0; lightboxY = 0; lightboxRotation = 0; }
	function rotateRight() { lightboxRotation = (lightboxRotation + 90) % 360; }

	function fitToScreen() {
		lightboxX = 0;
		lightboxY = 0;
		const img = new window.Image();
		img.onload = () => {
			const scaleH = (window.innerHeight * 0.9) / img.naturalHeight;
			lightboxZoom = Math.min(scaleH, 5);
		};
		img.src = lightboxSrc;
	}

	function navigatePrev() {
		if (lightboxImages.length <= 1) return;
		lightboxIndex = (lightboxIndex - 1 + lightboxImages.length) % lightboxImages.length;
		lightboxSrc = lightboxImages[lightboxIndex].src;
		lightboxAlt = lightboxImages[lightboxIndex].alt;
		resetView();
	}

	function navigateNext() {
		if (lightboxImages.length <= 1) return;
		lightboxIndex = (lightboxIndex + 1) % lightboxImages.length;
		lightboxSrc = lightboxImages[lightboxIndex].src;
		lightboxAlt = lightboxImages[lightboxIndex].alt;
		resetView();
	}

	function openInNewTab() {
		window.open(lightboxSrc, '_blank', 'noopener,noreferrer');
	}

	async function copyToClipboard() {
		try {
			const res = await fetch(lightboxSrc);
			const blob = await res.blob();
			await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })]);
			copyFeedback = 'Copied!';
			setTimeout(() => copyFeedback = '', 2000);
		} catch {
			copyFeedback = 'Failed';
			setTimeout(() => copyFeedback = '', 2000);
		}
	}

	async function downloadImage() {
		try {
			const a = document.createElement('a');
			const res = await fetch(lightboxSrc);
			const blob = await res.blob();
			a.href = URL.createObjectURL(blob);
			a.download = (lightboxAlt && lightboxAlt !== 'image' ? lightboxAlt : 'image') + '.png';
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
		} catch (e) {
			console.error('Download failed:', e);
		}
	}

	function handleLightboxWheel(event: WheelEvent) {
		event.preventDefault();
		if (event.deltaY < 0) zoomIn(); else zoomOut();
	}

	function handleLightboxMouseDown(event: MouseEvent) {
		if (event.button !== 0) return;
		isLightboxDragging = true;
		dragStartX = event.clientX;
		dragStartY = event.clientY;
		dragStartOffsetX = lightboxX;
		dragStartOffsetY = lightboxY;
	}

	function handleLightboxMouseMove(event: MouseEvent) {
		if (!isLightboxDragging) return;
		lightboxX = dragStartOffsetX + (event.clientX - dragStartX);
		lightboxY = dragStartOffsetY + (event.clientY - dragStartY);
	}

	function handleLightboxMouseUp() {
		isLightboxDragging = false;
	}

	function handleLightboxKeydown(event: KeyboardEvent) {
		if (!lightboxOpen) return;
		switch (event.key) {
			case 'Escape': closeLightbox(); break;
			case '+':
			case '=': zoomIn(); break;
			case '-': zoomOut(); break;
			case '0': resetView(); break;
			case 'r':
			case 'R': rotateRight(); break;
			case 'f':
			case 'F': fitToScreen(); break;
			case 'ArrowLeft': navigatePrev(); break;
			case 'ArrowRight': navigateNext(); break;
			case 'c':
			case 'C':
				if (!event.metaKey && !event.ctrlKey) void copyToClipboard();
				break;
		}
	}

	async function handleCommentSubmit() {
		if (!editingTask?.id) return;
		if (!commentContent.trim() && commentFiles.length === 0) return;
		commentSubmitting = true;
		commentsError = '';
		try {
			await createTaskComment(editingTask.id, { content: commentContent, files: commentFiles });
			commentContent = '';
			commentFiles = [];
			await loadComments(1);
		} catch (error) {
			commentsError = error instanceof Error ? error.message : 'Failed to submit comment';
		} finally {
			commentSubmitting = false;
		}
	}

	function openDeleteCommentConfirm(comment: TaskComment) {
		pendingDeleteComment = comment;
		showDeleteCommentConfirm = true;
	}

	function closeDeleteCommentConfirm() {
		if (commentDeleting) return;
		showDeleteCommentConfirm = false;
		pendingDeleteComment = null;
	}

	function forceCloseDeleteCommentConfirm() {
		showDeleteCommentConfirm = false;
		pendingDeleteComment = null;
	}

	async function handleDeleteComment() {
		if (!editingTask?.id || !pendingDeleteComment?.id) return;
		commentsError = '';
		commentDeleting = true;
		try {
			await deleteTaskComment(editingTask.id, pendingDeleteComment.id);
			const nextTotal = Math.max(0, commentsTotal - 1);
			const nextPages = Math.max(1, Math.ceil(nextTotal / commentsLimit));
			const nextPage = Math.min(commentsPage, nextPages);
			await loadComments(nextPage);
			forceCloseDeleteCommentConfirm();
		} catch (error) {
			commentsError = error instanceof Error ? error.message : 'Failed to delete comment';
		} finally {
			commentDeleting = false;
		}
	}

	function startEditComment(comment: TaskComment) {
		editingCommentId = String(comment.id || '');
		editingCommentText = comment.content || '';
	}

	function cancelEditComment() {
		editingCommentId = null;
		editingCommentText = '';
	}

	async function saveEditComment(comment: TaskComment) {
		if (!editingTask?.id || !comment.id) return;
		commentUpdating = true;
		commentsError = '';
		try {
			await updateTaskCommentText(editingTask.id, comment.id, editingCommentText);
			cancelEditComment();
			await loadComments(commentsPage);
		} catch (error) {
			commentsError = error instanceof Error ? error.message : 'Failed to update comment';
		} finally {
			commentUpdating = false;
		}
	}

	function copyShareLink() {
		if (!editingTask?.id) return;
		const url = new URL(window.location.href);
		url.searchParams.set('task', String(editingTask.id));
		navigator.clipboard.writeText(url.toString());
		copySuccess = true;
		setTimeout(() => {
			copySuccess = false;
		}, 2000);
	}

	function getPullRequestUrl(): string {
		if (!currentProjectRepoUrl) return '';
		const base = currentProjectRepoUrl.replace(/\/+$/, '').replace(/\.git$/, '');
		if (base.includes('github.com')) return `${base}/compare?expand=1`;
		if (base.includes('gitlab.com') || base.includes('gitlab')) return `${base}/-/merge_requests/new`;
		if (base.includes('bitbucket.org') || base.includes('bitbucket')) return `${base}/pull-requests/new`;
		return base;
	}

	function openPullRequest() {
		const url = getPullRequestUrl();
		if (url) window.open(url, '_blank', 'noopener,noreferrer');
	}

	$: {
		const nextFormInitKey = show ? `${editingTask?.id ?? 'new'}:${editingTask?.created_at ?? ''}` : 'closed';
		if (show && nextFormInitKey !== formInitKey) {
			initializeFormState();
		}
		formInitKey = nextFormInitKey;
	}

	$: if (show && editingTask?.id) {
		const key = `${editingTask.id}:${commentsPage}:${commentsLimit}`;
		if (key !== lastCommentsKey) {
			lastCommentsKey = key;
			void loadComments(commentsPage);
		}
	}

	function handleSubmit() {
		if (!title.trim()) return;
		if (assignee_id_to_add !== null && !assignee_ids.includes(assignee_id_to_add)) {
			assignee_ids = [...assignee_ids, assignee_id_to_add];
		}
		if (!editingTask) {
			taskDefaults.set({
				project: project.trim(),
				assignee_id: assignee_ids.length > 0 ? assignee_ids[0] : null,
				category
			});
		}

		dispatch('submit', {
			title: title.trim(),
			project: project.trim(),
			duration_minutes: 0,
			date,
			end_date: end_date || undefined,
			status,
			category,
			notes: notes.trim(),
			assignee_ids: assignee_ids.length > 0 ? assignee_ids : undefined,
			assignee_id: assignee_ids.length > 0 ? assignee_ids[0] : null,
			sprint_id,
			checklist: checklist.length > 0 ? checklist : undefined
		});
	}

	function handleClose() {
		dispatch('close');
	}

	function handleCancel() {
		dispatch('cancel');
	}

	function handleAddAssignee(event: CustomEvent<{ name: string; color: string }>) {
		dispatch('addAssignee', event.detail);
	}

	function handleChecklistUpdate(event: CustomEvent<{ checklist: ChecklistItem[] }>) {
		checklist = event.detail.checklist;
		if (editingTask) {
			dispatch('checklistUpdate', { checklist });
		}
	}

	function openBranchDialog() {
		showBranchDialog = true;
	}

	function closeBranchDialog() {
		showBranchDialog = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			if (lightboxOpen) {
				closeLightbox();
				return;
			}
			if (showBranchDialog) {
				closeBranchDialog();
				return;
			}
			handleClose();
		}
	}

	onMount(() => {
		document.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		if (browser) {
			document.removeEventListener('keydown', handleKeydown);
		}
	});
</script>

{#if show}
	<div class="fixed inset-0 bg-black/20 backdrop-blur-sm z-[20000] pointer-events-none !m-0"></div>
	<div class="fixed inset-0 z-[20000] overflow-y-auto !m-0" on:click|self={handleClose} on:keydown|self={(e) => e.key === 'Escape' && handleClose()} role="button" tabindex="-1">
		<div class="flex min-h-full items-center justify-center p-4">
			<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl {editingTask?.id ? 'max-w-6xl' : 'max-w-2xl'} w-full animate-modal-in relative max-h-[90vh] flex flex-col">
				<div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
					<h2 class="text-lg font-semibold text-gray-800 dark:text-white flex items-center gap-2">
						<CheckCircle size={20} class="text-primary" />
						{editingTask ? $_('taskForm__edit_task_title') : $_('taskForm__add_task_title')}
					</h2>
					<div class="flex items-center gap-2">
						{#if editingTask?.id}
							<button type="button" on:click={copyShareLink} class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium {copySuccess ? 'text-green-600 dark:text-green-400 border-green-300 dark:border-green-600 bg-green-50 dark:bg-green-900/10' : 'text-blue-600 dark:text-blue-400 border-blue-300 dark:border-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/10'} border rounded-lg transition-all" title="Copy share link">
								{#if copySuccess}
									<Check size={14} />
									<span>Copied!</span>
								{:else}
									<Link size={14} />
									<span>Share</span>
								{/if}
							</button>
						{/if}
						{#if currentProjectRepoUrl}
							<button type="button" on:click={openPullRequest} class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-emerald-600 dark:text-emerald-400 border border-emerald-300 dark:border-emerald-600 rounded-lg hover:bg-emerald-50 dark:hover:bg-emerald-900/30 transition-colors" title="Open Pull Request">
								<GitPullRequest size={14} />
								<span>Pull Request</span>
								<ExternalLink size={12} class="opacity-60" />
							</button>
						{/if}
						<button type="button" on:click={openBranchDialog} class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-gray-600 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" title="Branch">
							<GitBranch size={14} />
							<span>Branch</span>
						</button>
						<button type="button" on:click={handleClose} class="p-1.5 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
							<X size={20} />
						</button>
					</div>
				</div>

				<form on:submit|preventDefault={handleSubmit} class="flex flex-col flex-1 min-h-0">
					<div class="p-6 overflow-y-auto flex-1 min-h-0 custom-scrollbar">
						<div class={editingTask?.id ? 'grid grid-cols-1 xl:grid-cols-5 gap-4' : 'space-y-4'}>
							<div class={editingTask?.id ? 'xl:col-span-3 space-y-4' : 'space-y-4'}>
								<div>
							<label for="title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{$_('taskForm__task_title_label')} <span class="text-danger">*</span></label>
							<input id="title" type="text" bind:value={title} placeholder={$_('taskForm__task_title_placeholder')} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none dark:bg-gray-700 dark:text-white" required />
								</div>

						<div class="grid grid-cols-2 gap-4">
							<div>
								<label for="project" class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 flex items-center gap-1"><Folder size={14} />{$_('taskForm__project_label')}</label>
								<SearchableSelect id="project" bind:value={project} options={[{ value: '', label: '-- ' + $_('taskForm__unassigned') + ' --' }, ...projects.map(proj => ({ value: proj.name, label: proj.name }))]} placeholder={$_('taskForm__project_placeholder')} />
							</div>
							<div>
								<label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 flex items-center gap-1"><Tag size={14} />{$_('taskForm__category_label')}</label>
								<SearchableSelect bind:value={category} options={CATEGORIES.map(cat => ({ value: cat, label: cat }))} placeholder={$_('taskForm__category_placeholder')} showSearch={false} />
							</div>
						</div>

						<div class="grid grid-cols-2 gap-4">
							<div>
								<label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 flex items-center gap-1"><Calendar size={14} />{$_('taskForm__date_label')}</label>
								<CustomDatePicker bind:value={date} placeholder={$_('taskForm__date_placeholder')} on:select={(e) => date = e.detail} />
							</div>
							<div>
								<label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 flex items-center gap-1"><Calendar size={14} />วันสิ้นสุด (Optional)</label>
								<CustomDatePicker bind:value={end_date} placeholder="เลือกวันสิ้นสุด..." on:select={(e) => end_date = e.detail} />
							</div>
						</div>

						<AssigneeSelector {assignees} bind:assignee_ids bind:assignee_id_to_add readonly={!isOwner} on:addAssignee={handleAddAssignee} />
						<ChecklistManager bind:checklist autoDispatch={!!editingTask} on:update={handleChecklistUpdate} />

						<div>
							<label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 flex items-center gap-1"><FileText size={14} />{$_('taskForm__notes_label')}</label>
							<MarkdownEditor bind:value={notes} placeholder={$_('taskForm__notes_placeholder')} rows={4} />
						</div>

							</div>

							{#if editingTask?.id}
								<div class="xl:col-span-2 rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50/70 dark:bg-gray-900/30 p-4 space-y-3 h-fit">
								<div class="flex items-center gap-2 text-sm font-semibold text-gray-800 dark:text-gray-100"><MessageCircle size={16} /><span>Comments and activity</span></div>
								<textarea bind:value={commentContent} placeholder="Write a comment..." rows="3" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none dark:bg-gray-800 dark:text-white"></textarea>
								<div class="flex items-center gap-3">
									<label class="inline-flex items-center gap-2 px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-600 text-sm text-gray-700 dark:text-gray-200 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"><ImageIcon size={14} /><span>Attach images</span><input type="file" accept="image/*" multiple class="hidden" on:change={handleCommentFileChange} /></label>
									<button type="button" on:click={handleCommentSubmit} disabled={commentSubmitting || (!commentContent.trim() && commentFiles.length === 0)} class="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-dark disabled:opacity-50 disabled:cursor-not-allowed"><Send size={14} /><span>{commentSubmitting ? 'Saving...' : 'Save'}</span></button>
								</div>

								{#if commentFiles.length > 0}
									<div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
										{#each commentFiles as file, idx}
											<div class="relative rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-2 text-xs text-gray-600 dark:text-gray-300"><p class="truncate">{file.name}</p><button type="button" on:click={() => removeCommentFile(idx)} class="absolute top-1 right-1 text-red-500">×</button></div>
										{/each}
									</div>
								{/if}

								{#if commentsError}<p class="text-sm text-red-500">{commentsError}</p>{/if}

								{#if commentsLoading}
									<p class="text-sm text-gray-500 dark:text-gray-400">Loading comments...</p>
								{:else if comments.length === 0}
									<p class="text-sm text-gray-500 dark:text-gray-400">No comments yet</p>
								{:else}
									<div class="space-y-3">
										{#each comments as comment}
											<div class="p-1 space-y-2">
												<div class="flex items-center gap-2 text-xs">
													<div class="w-7 h-7 rounded-full bg-primary/15 dark:bg-primary/20 text-primary font-semibold flex items-center justify-center">
														{getAuthorInitials(comment)}
													</div>
													<span class="font-semibold text-gray-800 dark:text-gray-200">{formatCommentAuthor(comment)}</span>
													<span class="text-primary underline underline-offset-2">{formatCommentRelativeTime(comment.created_at)}</span>
												</div>
												{#if editingCommentId === String(comment.id || '')}
													<div class="space-y-2">
														<textarea
															bind:value={editingCommentText}
															rows="3"
															class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary outline-none dark:bg-gray-800 dark:text-white"
														></textarea>
														<div class="flex items-center gap-2">
															<button type="button" on:click={() => saveEditComment(comment)} disabled={commentUpdating} class="px-3 py-1.5 rounded-md bg-primary text-white text-xs font-medium disabled:opacity-50">{commentUpdating ? 'Saving...' : 'Save'}</button>
															<button type="button" on:click={cancelEditComment} disabled={commentUpdating} class="px-3 py-1.5 rounded-md border border-gray-300 dark:border-gray-600 text-xs font-medium">Cancel</button>
														</div>
													</div>
												{:else if comment.content}
													<p class="text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap rounded-lg bg-gray-50 dark:bg-gray-900/60 px-3 py-2">{comment.content}</p>
												{/if}
												{#if getImageState(comment.id)?.total > 0}
													{#if getImageState(comment.id)?.loading}
														<p class="text-xs text-gray-500">Loading images...</p>
													{:else}
														<div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
															{#each getImageState(comment.id)?.images || [] as image, imageIndex}
																<a href={getCommentImageUrl(image.file_key)} target="_blank" rel="noreferrer" on:click|preventDefault={() => openCommentImageLightbox(comment, imageIndex)} class="block rounded-md overflow-hidden border border-gray-200 dark:border-gray-700"><img src={getCommentImageUrl(image.file_key)} alt={image.filename} class="w-full h-24 object-cover" loading="lazy" /></a>
															{/each}
														</div>
														{#if (getImageState(comment.id)?.total || 0) > (getImageState(comment.id)?.limit || 1)}
															<div class="flex items-center justify-between text-xs text-gray-500 mt-2">
																<button type="button" class="px-2 py-1 border rounded disabled:opacity-50" disabled={(getImageState(comment.id)?.page || 1) <= 1} on:click={() => prevCommentImagesPage(comment)}>Prev</button>
																<span>Page {getImageState(comment.id)?.page || 1} / {Math.max(1, Math.ceil((getImageState(comment.id)?.total || 0) / (getImageState(comment.id)?.limit || 1)))}</span>
																<button type="button" class="px-2 py-1 border rounded disabled:opacity-50" disabled={(getImageState(comment.id)?.page || 1) >= Math.ceil((getImageState(comment.id)?.total || 0) / (getImageState(comment.id)?.limit || 1))} on:click={() => nextCommentImagesPage(comment)}>Next</button>
															</div>
														{/if}
													{/if}
												{/if}
												<div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
													<button
														type="button"
														on:click={() => startEditComment(comment)}
														class="inline-flex items-center gap-1 hover:text-primary"
														title="Edit comment text"
													>
														<Edit2 size={12} />
														<span>Edit</span>
													</button>
													<span>•</span>
													<button
														type="button"
														on:click={() => openDeleteCommentConfirm(comment)}
														class="inline-flex items-center gap-1 text-danger hover:opacity-80"
														title="Delete comment"
													>
														<Trash2 size={12} />
														<span>Delete</span>
													</button>
												</div>
											</div>
										{/each}
									</div>

									{#if commentsTotal > commentsLimit}
										<Pagination totalItems={commentsTotal} bind:pageSize={commentsLimit} bind:currentPage={commentsPage} pageSizeOptions={[10, 20, 50]} />
									{/if}
								{/if}
								</div>
							{/if}
						</div>
					</div>

					<div class="flex justify-end gap-3 px-6 py-4 border-t border-gray-200 dark:border-gray-700 shrink-0">
						<button type="button" on:click={handleClose} class="min-w-[96px] px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors font-medium">{$_('taskForm__btn_cancel')}</button>
						<button type="submit" class="min-w-[96px] bg-primary hover:bg-primary-dark text-white py-2 px-4 rounded-lg font-medium transition-colors">{editingTask ? $_('taskForm__btn_save') : $_('taskForm__btn_add')}</button>
					</div>
				</form>
			</div>
		</div>
	</div>

	<BranchDialog bind:show={showBranchDialog} {title} on:close={closeBranchDialog} />

	{#if lightboxOpen}
		<div
			class="fixed inset-0 z-[20020] bg-black/80 backdrop-blur-sm flex items-center justify-center !m-0"
			on:click|self={closeLightbox}
			on:wheel|preventDefault={handleLightboxWheel}
			on:keydown={handleLightboxKeydown}
			tabindex="-1"
			role="dialog"
			aria-modal="true"
			aria-label="Image viewer"
		>
			<div class="absolute top-4 right-4 flex items-center gap-1.5 z-10">
				<span class="text-white/70 text-sm font-mono bg-black/40 px-2 py-1 rounded">{Math.round(lightboxZoom * 100)}%</span>
				{#if lightboxRotation !== 0}
					<span class="text-white/70 text-sm font-mono bg-black/40 px-2 py-1 rounded">{lightboxRotation}°</span>
				{/if}
				{#if lightboxImages.length > 1}
					<span class="text-white/70 text-sm bg-black/40 px-2 py-1 rounded">{lightboxIndex + 1}/{lightboxImages.length}</span>
				{/if}
				<div class="w-px h-5 bg-white/20"></div>
				<button type="button" on:click={zoomIn} class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors" title="Zoom in (+)">
					<ZoomIn size={18} />
				</button>
				<button type="button" on:click={zoomOut} class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors" title="Zoom out (-)">
					<ZoomOut size={18} />
				</button>
				<button type="button" on:click={fitToScreen} class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors" title="Fit to screen (F)">
					<Maximize size={18} />
				</button>
				<button type="button" on:click={rotateRight} class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors" title="Rotate (R)">
					<RotateCw size={18} />
				</button>
				<button type="button" on:click={resetView} class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors" title="Reset (0)">
					<RotateCcw size={18} />
				</button>
				<div class="w-px h-5 bg-white/20"></div>
				<button type="button" on:click={() => void copyToClipboard()} class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors relative" title="Copy (C)">
					<Copy size={18} />
					{#if copyFeedback}
						<span class="absolute -bottom-7 left-1/2 -translate-x-1/2 text-xs bg-green-500 text-white px-2 py-0.5 rounded whitespace-nowrap">{copyFeedback}</span>
					{/if}
				</button>
				<button type="button" on:click={downloadImage} class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors" title="Download">
					<Download size={18} />
				</button>
				<button type="button" on:click={openInNewTab} class="p-2 bg-black/40 hover:bg-black/60 text-white rounded-lg transition-colors" title="Open in new tab">
					<ExternalLink size={18} />
				</button>
				<div class="w-px h-5 bg-white/20"></div>
				<button type="button" on:click={closeLightbox} class="p-2 bg-black/40 hover:bg-red-600/80 text-white rounded-lg transition-colors" title="Close (Esc)">
					<X size={18} />
				</button>
			</div>

			{#if lightboxImages.length > 1}
				<button
					type="button"
					on:click={navigatePrev}
					class="absolute left-4 top-1/2 -translate-y-1/2 p-3 bg-black/40 hover:bg-black/60 text-white rounded-full transition-colors z-10"
					title="Previous (←)"
				>
					<ChevronLeft size={24} />
				</button>
				<button
					type="button"
					on:click={navigateNext}
					class="absolute right-4 top-1/2 -translate-y-1/2 p-3 bg-black/40 hover:bg-black/60 text-white rounded-full transition-colors z-10"
					title="Next (→)"
				>
					<ChevronRight size={24} />
				</button>
			{/if}

			<img
				src={lightboxSrc}
				alt={lightboxAlt}
				class="max-w-[90vw] max-h-[85vh] object-contain select-none {isLightboxDragging ? 'cursor-grabbing' : 'cursor-grab transition-transform duration-150'}"
				style="transform: scale({lightboxZoom}) translate({lightboxX / lightboxZoom}px, {lightboxY / lightboxZoom}px) rotate({lightboxRotation}deg);"
				on:mousedown={handleLightboxMouseDown}
				on:mousemove={handleLightboxMouseMove}
				on:mouseup={handleLightboxMouseUp}
				on:mouseleave={handleLightboxMouseUp}
				draggable="false"
			/>

			{#if lightboxAlt && lightboxAlt !== 'image'}
				<div class="absolute bottom-4 left-1/2 -translate-x-1/2 bg-black/50 text-white/80 text-sm px-4 py-1.5 rounded-full">
					{lightboxAlt}
				</div>
			{/if}
		</div>
	{/if}

	{#if showDeleteCommentConfirm && pendingDeleteComment}
		<div class="fixed inset-0 z-[20010] bg-black/50 backdrop-blur-sm flex items-center justify-center p-4" on:click|self={closeDeleteCommentConfirm}>
			<div class="w-full max-w-sm rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 shadow-2xl">
				<div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
					<h3 class="text-sm font-semibold">Delete comment?</h3>
					<button
						type="button"
						on:click={closeDeleteCommentConfirm}
						disabled={commentDeleting}
						class="w-7 h-7 inline-flex items-center justify-center rounded-md border border-gray-300 dark:border-gray-600 text-gray-500 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50"
					>
						<X size={14} />
					</button>
				</div>
				<div class="px-4 py-3 space-y-3">
					<p class="text-sm text-gray-600 dark:text-gray-300">Deleting a comment is forever. There is no undo.</p>
					<button
						type="button"
						on:click={handleDeleteComment}
						disabled={commentDeleting}
						class="w-full rounded-md bg-danger hover:opacity-90 text-white text-sm font-semibold py-2.5 disabled:opacity-50"
					>
						{commentDeleting ? 'Deleting...' : 'Delete comment'}
					</button>
				</div>
			</div>
		</div>
	{/if}
{/if}

<style>
	@keyframes modal-in {
		from {
			opacity: 0;
			transform: scale(0.95) translateY(-10px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.animate-modal-in {
		animation: modal-in 0.2s ease-out;
	}

	.custom-scrollbar::-webkit-scrollbar {
		width: 4px;
	}

	.custom-scrollbar::-webkit-scrollbar-track {
		background: transparent;
	}

	.custom-scrollbar::-webkit-scrollbar-thumb {
		background: #d1d5db;
		border-radius: 10px;
	}

	:global(.dark .custom-scrollbar::-webkit-scrollbar-thumb) {
		background: #4b5563;
	}

	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background: #9ca3af;
	}
</style>
