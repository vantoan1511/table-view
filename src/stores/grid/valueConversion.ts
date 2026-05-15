import type { GridColumn, CellValue } from '@/types';

type ParseSuccess = { ok: true; value: CellValue | Record<string, unknown> | unknown[] };
type ParseFailure = { ok: false; message: string };

export type ParseGridInputResult = ParseSuccess | ParseFailure;

const integerTypes = new Set([
  'int2',
  'int4',
  'int8',
  'smallint',
  'integer',
  'bigint',
  'tinyint',
  'mediumint',
  'serial',
  'bigserial',
  'smallserial',
  '21',
  '23',
  '20'
]);

const numericTypes = new Set([
  'float4',
  'float8',
  'real',
  'double precision',
  'numeric',
  'decimal',
  'number',
  'float',
  'double',
  '700',
  '701',
  '1700'
]);

const booleanTypes = new Set(['bool', 'boolean', '16']);
const jsonTypes = new Set(['json', 'jsonb', '114', '3802']);

const normalizeDataType = (dataType: string) => dataType.trim().toLowerCase();

export const parseGridInputValue = (
  rawValue: unknown,
  column: GridColumn
): ParseGridInputResult => {
  if (rawValue === null) return { ok: true, value: null };

  const text = String(rawValue).trim();
  const type = normalizeDataType(column.dataType);

  if (text === '') {
    if (column.isNullable !== false && !column.isPrimaryKey) {
      return { ok: true, value: null };
    }
    return { ok: false, message: `${column.name} is required` };
  }

  if (integerTypes.has(type)) {
    if (!/^[+-]?\d+$/.test(text)) {
      return { ok: false, message: `${column.name} must be a whole number` };
    }
    const value = Number(text);
    if (!Number.isSafeInteger(value)) {
      return { ok: false, message: `${column.name} is outside the safe integer range` };
    }
    return { ok: true, value };
  }

  if (numericTypes.has(type)) {
    const value = Number(text);
    if (!Number.isFinite(value)) {
      return { ok: false, message: `${column.name} must be a valid number` };
    }
    return { ok: true, value };
  }

  if (booleanTypes.has(type)) {
    const normalized = text.toLowerCase();
    if (['true', 't', 'yes', 'y', '1'].includes(normalized)) {
      return { ok: true, value: true };
    }
    if (['false', 'f', 'no', 'n', '0'].includes(normalized)) {
      return { ok: true, value: false };
    }
    return { ok: false, message: `${column.name} must be true or false` };
  }

  if (jsonTypes.has(type)) {
    try {
      return { ok: true, value: JSON.parse(text) };
    } catch {
      return { ok: false, message: `${column.name} must be valid JSON` };
    }
  }

  return { ok: true, value: String(rawValue) };
};

export const formatGridCellValue = (value: unknown): string => {
  if (value === null || value === undefined) return '';
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
};
