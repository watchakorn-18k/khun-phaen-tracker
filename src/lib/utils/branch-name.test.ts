import { describe, it, expect } from 'vitest';
import { getWorkItemPrefix, getComputedBranchName } from './branch-name';

describe('branch-name utils', () => {
  describe('getWorkItemPrefix', () => {
    it('should return project short name with task number', () => {
      expect(getWorkItemPrefix('PROJ', 123)).toBe('PROJ-123');
    });

    it('should return project short name only if task number is missing', () => {
      expect(getWorkItemPrefix('PROJ', null)).toBe('PROJ');
    });

    it('should handle workspace short name with spaces', () => {
      expect(getWorkItemPrefix('MY PROJ', 123)).toBe('MYPR-123');
    });

    it('should return task prefix if short name is missing', () => {
      expect(getWorkItemPrefix('', 123)).toBe('task-123');
    });
  });

  describe('getComputedBranchName', () => {
    it('should generate feature branch with project prefix', () => {
      const options = {
        gitFlowType: 'feature' as const,
        workspaceShortName: 'PROJ',
        taskNumber: 123,
        title: 'Fix Login System'
      };
      expect(getComputedBranchName(options)).toBe('feature/PROJ-123-fix-login-system');
    });

    it('should handle missing title by using task number', () => {
      const options = {
        gitFlowType: 'feature' as const,
        workspaceShortName: 'PROJ',
        taskNumber: 123,
        title: ''
      };
      expect(getComputedBranchName(options)).toBe('feature/PROJ-123');
    });

    it('should prioritize translated title', () => {
      const options = {
        gitFlowType: 'bugfix' as const,
        workspaceShortName: 'PROJ',
        taskNumber: 456,
        title: 'แก้บั๊ก',
        translatedTitle: 'Fix Bug'
      };
      expect(getComputedBranchName(options)).toBe('bugfix/PROJ-456-fix-bug');
    });
  });
});
