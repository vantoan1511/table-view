import { defineStore } from 'pinia';
import { ref } from 'vue';

import { BridgeService } from '@/services/bridge';
import type { SchemaDetails, TableDetails, TableRelation } from '@/types';
import { useToastStore } from './toast';

export const useDiagramStore = defineStore('diagram', () => {
  const toastStore = useToastStore();

  const diagrams = ref<Record<string, SchemaDetails>>({});
  const loading = ref<Record<string, boolean>>({});
  const errors = ref<Record<string, string | null>>({});

  const getCacheKey = (connectionId: string, schemaName: string, dbName?: string): string => {
    return `${connectionId}-${dbName || 'default'}-${schemaName}`;
  };

  const fetchSchemaDetails = async (
    connectionId: string,
    schemaName: string,
    dbName?: string,
    force = false
  ): Promise<SchemaDetails | null> => {
    const key = getCacheKey(connectionId, schemaName, dbName);
    if (!force && diagrams.value[key]) {
      return diagrams.value[key] || null;
    }

    loading.value[key] = true;
    errors.value[key] = null;

    try {
      const payload = await BridgeService.request(
        'dbBridge.getSchemaDetails',
        'dbBridge.getSchemaDetailsResult',
        {
          connectionId,
          schemaName,
          targetDatabase: dbName
        }
      );

      if (!payload) {
        throw new Error('Empty response from database bridge');
      }

      const relations: TableRelation[] = (payload.relations || []).map((r: any) => ({
        constraintName: r.constraintName || r.constraint_name || '',
        sourceTable: r.sourceTable || r.source_table || '',
        sourceColumn: r.sourceColumn || r.source_column || '',
        targetTable: r.targetTable || r.target_table || '',
        targetColumn: r.targetColumn || r.target_column || ''
      }));

      const fkMap = new Map<string, { targetTable: string; targetColumn: string }>();
      for (const rel of relations) {
        if (rel.sourceTable && rel.sourceColumn) {
          fkMap.set(`${rel.sourceTable}.${rel.sourceColumn}`, {
            targetTable: rel.targetTable,
            targetColumn: rel.targetColumn
          });
        }
      }

      const tables: TableDetails[] = (payload.tables || []).map((t: any) => ({
        name: t.name,
        columns: (t.columns || []).map((c: any) => {
          const colName = c.name;
          const fk = fkMap.get(`${t.name}.${colName}`);
          return {
            name: colName,
            dataType: c.dataType || c.data_type || 'unknown',
            nullable: c.nullable ?? true,
            isPrimaryKey: c.isPrimaryKey || c.is_primary_key || false,
            default: c.default || undefined,
            foreignKey: fk
              ? { targetTable: fk.targetTable, targetColumn: fk.targetColumn }
              : undefined
          };
        })
      }));

      // Ensure proper structure
      const schemaDetails: SchemaDetails = {
        tables,
        relations
      };

      diagrams.value[key] = schemaDetails;
      return schemaDetails;
    } catch (err: any) {
      console.error('[diagram] Failed to load schema details:', err);
      const msg = err.message || 'Could not retrieve schema details.';
      errors.value[key] = msg;

      toastStore.addToast({
        severity: 'error',
        title: 'Diagram Load Failed',
        message: msg
      });
      return null;
    } finally {
      loading.value[key] = false;
    }
  };

  const clearCache = (connectionId: string, schemaName: string, dbName?: string) => {
    const key = getCacheKey(connectionId, schemaName, dbName);
    delete diagrams.value[key];
    delete loading.value[key];
    delete errors.value[key];
  };

  return {
    diagrams,
    loading,
    errors,
    fetchSchemaDetails,
    clearCache,
    getCacheKey
  };
});
