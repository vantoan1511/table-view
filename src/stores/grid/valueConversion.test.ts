import type { GridColumn } from '@/types';
import { describe, expect, it } from 'vitest';
import { formatGridCellValue, parseGridInputValue } from './valueConversion';

const column = (dataType: string, overrides: Partial<GridColumn> = {}): GridColumn => ({
  name: 'value',
  dataType,
  ...overrides
});

describe('formatGridCellValue', () => {
  it('formats object values as json text', () => {
    expect(formatGridCellValue({ active: true })).toBe('{"active":true}');
  });

  it('formats null values as empty text for editing', () => {
    expect(formatGridCellValue(null)).toBe('');
  });
});

describe('parseGridInputValue', () => {
  it('converts empty nullable values to null', () => {
    expect(parseGridInputValue('', column('VARCHAR', { isNullable: true }))).toEqual({
      ok: true,
      value: null
    });
  });

  it('rejects empty required values', () => {
    expect(parseGridInputValue('', column('INT4', { isNullable: false }))).toEqual({
      ok: false,
      message: 'value is required'
    });
  });

  it('converts integer values', () => {
    expect(parseGridInputValue('42', column('INT4'))).toEqual({ ok: true, value: 42 });
  });

  it('rejects invalid integer values', () => {
    expect(parseGridInputValue('42.5', column('INT4'))).toEqual({
      ok: false,
      message: 'value must be a whole number'
    });
  });

  it('converts boolean values', () => {
    expect(parseGridInputValue('yes', column('BOOL'))).toEqual({ ok: true, value: true });
    expect(parseGridInputValue('0', column('BOOL'))).toEqual({ ok: true, value: false });
  });

  it('parses json values', () => {
    expect(parseGridInputValue('{"active":true}', column('JSONB'))).toEqual({
      ok: true,
      value: { active: true }
    });
  });
});
