import { HighlightStyle } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import { EditorView } from 'codemirror'

export const sqlHighlightStyle = HighlightStyle.define([
  { tag: t.keyword, color: '#818CF8', fontWeight: 'bold' },
  { tag: t.string, color: '#34D399' },
  { tag: t.number, color: '#FBBF24' },
  { tag: t.comment, color: '#6C7086', fontStyle: 'italic' },
  { tag: t.operator, color: '#CDD6F4' },
  { tag: t.meta, color: '#CDD6F4' },
  { tag: t.typeName, color: '#818CF8' },
  { tag: t.propertyName, color: '#CDD6F4' },
  { tag: t.className, color: '#CDD6F4' },
  { tag: t.labelName, color: '#CDD6F4' },
  { tag: t.namespace, color: '#CDD6F4' },
  { tag: t.macroName, color: '#CDD6F4' },
  { tag: t.literal, color: '#34D399' },
  { tag: t.bool, color: '#FBBF24' },
  { tag: t.null, color: '#FBBF24' },
  { tag: t.name, color: '#CDD6F4' },
  { tag: t.heading, color: '#CDD6F4', fontWeight: 'bold' },
  { tag: t.invalid, color: '#FB7185' },
])

export const editorTheme = EditorView.theme({
  '&': {
    fontSize: '12px',
    fontFamily: 'var(--font-mono)',
    backgroundColor: 'var(--color-surface)',
    color: 'var(--color-text-primary)',
  },
  '.cm-content': {
    padding: '8px 0',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--color-muted)',
    borderRight: '1px solid var(--color-border)',
    color: 'var(--color-text-tertiary)',
    fontSize: '11px',
  },
  '.cm-activeLine': {
    backgroundColor: 'var(--color-active)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--color-active)',
  },
  '&.cm-focused .cm-cursor': {
    borderLeftColor: 'var(--color-primary)',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    backgroundColor: 'var(--color-primary-light)',
  },
}, { dark: true })
