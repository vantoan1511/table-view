import { describe, it, expect } from 'vitest';
import { useSchemaCache } from './useSchemaCache';

describe('useSchemaCache', () => {
  it('initializes with empty state', () => {
    const cache = useSchemaCache();
    expect(cache.schemasByConnection.value).toEqual({});
    expect(cache.loadingByConnection.value).toEqual({});
    expect(cache.perDbSchemas.value).toEqual({});
    expect(cache.loadingDbByConnection.value).toEqual({});
    expect(cache.errorDbByConnection.value).toEqual({});
  });

  it('emptySchema returns a correctly shaped object', () => {
    const cache = useSchemaCache();
    const empty = cache.emptySchema();
    expect(empty).toEqual({
      tables: [],
      views: [],
      functions: [],
      schemas: [],
      databases: undefined
    });
  });

  it('hasSchemaLoaded returns correct state', () => {
    const cache = useSchemaCache();
    expect(cache.hasSchemaLoaded('conn-1')).toBe(false);

    cache.schemasByConnection.value['conn-1'] = cache.emptySchema();
    expect(cache.hasSchemaLoaded('conn-1')).toBe(true);
  });

  it('getDbSchema and getDbError return correct data', () => {
    const cache = useSchemaCache();
    const connectionId = 'conn-1';
    const dbName = 'test_db';

    expect(cache.getDbSchema(connectionId, dbName)).toBe(null);
    expect(cache.getDbError(connectionId, dbName)).toBe(null);

    const mockSchema = cache.emptySchema();
    cache.perDbSchemas.value[connectionId] = { [dbName]: mockSchema };
    cache.errorDbByConnection.value[connectionId] = { [dbName]: 'Some error' };

    expect(cache.getDbSchema(connectionId, dbName)).toEqual(mockSchema);
    expect(cache.getDbError(connectionId, dbName)).toBe('Some error');
  });
});
