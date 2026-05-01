// @vitest-environment jsdom
import { render, fireEvent } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import TaskForm from './TaskForm.svelte';
import type { Task } from '$lib/types';
import { cleanup } from '@testing-library/svelte';
import '../i18n'; // Initialize i18n
import { waitLocale, locale } from 'svelte-i18n';

async function setup() {
    locale.set('th');
    await waitLocale();
}

function createTask(overrides: Partial<Task> = {}): Task {
	return {
		id: 1,
		title: 'Original task',
		project: 'Alpha',
		duration_minutes: 0,
		date: '2026-02-10',
		status: 'todo',
		category: 'งานหลัก',
		notes: 'Initial notes',
		assignee_id: null,
		sprint_id: null,
        created_at: new Date().toISOString(),
		...overrides
	};
}

describe('TaskForm', () => {
	afterEach(() => cleanup());

	it('renders create mode defaults and allows typing', async () => {
        await setup();
		const { getByPlaceholderText, getByText } = render(TaskForm, {
			show: true,
			editingTask: null,
			assignees: [],
			projects: [],
			sprints: []
		});

		const titleInput = getByPlaceholderText('เช่น แก้ไขระบบ login, เขียนเอกสาร...') as HTMLInputElement;
		expect(titleInput.value).toBe('');
		expect(getByText('สร้างงาน')).toBeTruthy();

		await fireEvent.input(titleInput, { target: { value: 'New task' } });
		expect(titleInput.value).toBe('New task');
	});

	it('prefills edit mode values and shows edit action text', async () => {
        await setup();
		const editingTask = createTask();
		const { getByPlaceholderText, getByText } = render(TaskForm, {
			show: true,
			editingTask,
			assignees: [],
			projects: [],
			sprints: []
		});

		const titleInput = getByPlaceholderText('เช่น แก้ไขระบบ login, เขียนเอกสาร...') as HTMLInputElement;
		expect(titleInput.value).toBe('Original task');
		expect((getByPlaceholderText('รายละเอียดเพิ่มเติม...') as HTMLTextAreaElement).value).toBe('Initial notes');
		expect(getByText('บันทึก')).toBeTruthy();
	});

    it('renders due date dropdown at the top', async () => {
        await setup();
        const { getByPlaceholderText, getByText } = render(TaskForm, {
            show: true,
            editingTask: null,
            assignees: [],
            projects: [],
            sprints: []
        });

        const datePicker = getByText('เลือกวันสิ้นสุด...');
        expect(datePicker).toBeTruthy();
        
        // Check if it's before the title input in the DOM
        const titleInput = getByPlaceholderText('เช่น แก้ไขระบบ login, เขียนเอกสาร...');
        const datePickerParent = datePicker.closest('.property-select');
        const titleInputParent = titleInput.closest('div');
        
        // This is a rough check for order
        const allNodes = Array.from(document.querySelectorAll('*'));
        expect(allNodes.indexOf(datePicker)).toBeLessThan(allNodes.indexOf(titleInput));
    });
});

