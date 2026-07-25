import { describe, expect, it } from 'vitest';

import {
  categorizeCommits,
  formatReleaseNotes,
  parseCommitLine
} from '../../scripts/generate-release-notes.js';

describe('generate-release-notes', () => {
  describe('parseCommitLine', () => {
    it('returns null for null, undefined, or empty lines', () => {
      expect(parseCommitLine('')).toBeNull();
      expect(parseCommitLine(null as any)).toBeNull();
      expect(parseCommitLine('   ')).toBeNull();
    });

    it('parses commit line with hash, type, and message', () => {
      const line = 'a35f350 feat: add download progress tracking for app updates';
      const parsed = parseCommitLine(line);
      expect(parsed).toEqual({
        type: 'feat',
        scope: null,
        message: 'add download progress tracking for app updates'
      });
    });

    it('parses commit line with scope', () => {
      const line = '40f85c6 fix(updater): add SpawnedProcessEvent interface';
      const parsed = parseCommitLine(line);
      expect(parsed).toEqual({
        type: 'fix',
        scope: 'updater',
        message: 'add SpawnedProcessEvent interface'
      });
    });

    it('returns null for non-conventional commit lines', () => {
      expect(parseCommitLine('Random commit message without type')).toBeNull();
      expect(parseCommitLine('1234567 Just a message')).toBeNull();
    });
  });

  describe('categorizeCommits', () => {
    it('categorizes feat, fix, and improvement commits while filtering ignored types and merge commits', () => {
      const logLines = [
        'a35f350 feat(ui): add download progress tracking',
        '40f85c6 fix(updater): resolve explicit any warning',
        'ebb2ef5 refactor: eliminate duplicate merge logic',
        '03dded2 style(ui): format PrimeVue dialog props',
        '52738fe chore: correct configs',
        'f7290d4 chore: update manifest-preview.json for preview [skip ci]',
        'Merge pull request #61 from feature'
      ];

      const categorized = categorizeCommits(logLines);

      expect(categorized.features).toEqual(['**ui**: Add download progress tracking']);
      expect(categorized.bugFixes).toEqual(['**updater**: Resolve explicit any warning']);
      expect(categorized.improvements).toEqual([
        'Eliminate duplicate merge logic',
        '**ui**: Format PrimeVue dialog props'
      ]);
    });
  });

  describe('formatReleaseNotes', () => {
    it('formats release notes into 3 distinct sections with icons', () => {
      const categorized = {
        features: ['**ui**: Add new feature'],
        bugFixes: ['Fix memory leak'],
        improvements: ['Optimize queries']
      };

      const markdown = formatReleaseNotes(categorized);

      expect(markdown).toContain('### 🚀 Features\n\n- **ui**: Add new feature');
      expect(markdown).toContain('### 🐛 Bug Fixes\n\n- Fix memory leak');
      expect(markdown).toContain('### ⚡ Improvements\n\n- Optimize queries');
    });

    it('returns fallback statement when all categories are empty', () => {
      const categorized = {
        features: [],
        bugFixes: [],
        improvements: []
      };

      const markdown = formatReleaseNotes(categorized);

      expect(markdown).toBe('General maintenance, performance, and stability improvements.');
    });
  });
});
