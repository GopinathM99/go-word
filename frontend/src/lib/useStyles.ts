/**
 * useStyles - Hook for interacting with the style system
 *
 * Provides functions to get styles, apply styles, and manage direct formatting.
 */

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Style,
  ResolvedStyle,
  StyleInspectorData,
  ParagraphProperties,
  CharacterProperties,
  DocumentChange,
} from './types';

interface UseStylesOptions {
  /** Document ID to work with */
  documentId: string | null;
  /** Auto-refresh interval in ms (0 to disable) */
  refreshInterval?: number;
}

interface UseStylesResult {
  /** All available styles */
  styles: Style[];
  /** Loading state */
  loading: boolean;
  /** Error message if any */
  error: string | null;
  /** Style inspector data for current selection */
  inspectorData: StyleInspectorData | null;
  /** Refresh styles from backend */
  refreshStyles: () => Promise<void>;
  /** Get a specific style by ID */
  getStyle: (styleId: string) => Promise<Style | null>;
  /** Get resolved style (with inheritance applied) */
  getResolvedStyle: (styleId: string) => Promise<ResolvedStyle | null>;
  /** Apply a paragraph style to current selection */
  applyParagraphStyle: (styleId: string) => Promise<DocumentChange | null>;
  /** Apply a character style to current selection */
  applyCharacterStyle: (styleId: string) => Promise<DocumentChange | null>;
  /** Apply direct formatting */
  applyDirectFormatting: (
    paragraphProps?: ParagraphProperties,
    characterProps?: CharacterProperties
  ) => Promise<DocumentChange | null>;
  /** Clear direct formatting */
  clearDirectFormatting: (
    clearParagraph: boolean,
    clearCharacter: boolean
  ) => Promise<DocumentChange | null>;
  /** Create a new style */
  createStyle: (style: Partial<Style>) => Promise<Style | null>;
  /** Modify an existing style */
  modifyStyle: (
    styleId: string,
    updates: Partial<Style>
  ) => Promise<Style | null>;
  /** Refresh inspector data */
  refreshInspector: () => Promise<void>;
}

export function useStyles({
  documentId,
  refreshInterval = 0,
}: UseStylesOptions): UseStylesResult {
  const [styles, setStyles] = useState<Style[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [inspectorData, setInspectorData] = useState<StyleInspectorData | null>(null);

  // Refresh styles from backend
  const refreshStyles = useCallback(async () => {
    if (!documentId) return;

    setLoading(true);
    setError(null);

    try {
      const result = await invoke<Style[]>('get_styles', { docId: documentId });
      setStyles(result);
    } catch (e) {
      setError(String(e));
      console.error('Failed to get styles:', e);
    } finally {
      setLoading(false);
    }
  }, [documentId]);

  // Refresh inspector data
  const refreshInspector = useCallback(async () => {
    if (!documentId) return;

    try {
      const result = await invoke<StyleInspectorData>('get_style_inspector', {
        docId: documentId,
      });
      setInspectorData(result);
    } catch (e) {
      console.error('Failed to get inspector data:', e);
    }
  }, [documentId]);

  // Load styles on mount and when documentId changes
  useEffect(() => {
    if (documentId) {
      refreshStyles();
      refreshInspector();
    }
  }, [documentId, refreshStyles, refreshInspector]);

  // Set up refresh interval if specified
  useEffect(() => {
    if (refreshInterval > 0 && documentId) {
      const interval = setInterval(() => {
        refreshInspector();
      }, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [refreshInterval, documentId, refreshInspector]);

  // Get a specific style
  const getStyle = useCallback(
    async (styleId: string): Promise<Style | null> => {
      if (!documentId) return null;

      try {
        const result = await invoke<Style | null>('get_style', {
          docId: documentId,
          styleId,
        });
        return result;
      } catch (e) {
        console.error('Failed to get style:', e);
        return null;
      }
    },
    [documentId]
  );

  // Get resolved style
  const getResolvedStyle = useCallback(
    async (styleId: string): Promise<ResolvedStyle | null> => {
      if (!documentId) return null;

      try {
        const result = await invoke<ResolvedStyle | null>('get_resolved_style', {
          docId: documentId,
          styleId,
        });
        return result;
      } catch (e) {
        console.error('Failed to get resolved style:', e);
        return null;
      }
    },
    [documentId]
  );

  // Apply paragraph style
  const applyParagraphStyle = useCallback(
    async (styleId: string): Promise<DocumentChange | null> => {
      if (!documentId) return null;

      try {
        const result = await invoke<DocumentChange>('apply_paragraph_style', {
          docId: documentId,
          styleId,
        });
        await refreshInspector();
        return result;
      } catch (e) {
        console.error('Failed to apply paragraph style:', e);
        return null;
      }
    },
    [documentId, refreshInspector]
  );

  // Apply character style
  const applyCharacterStyle = useCallback(
    async (styleId: string): Promise<DocumentChange | null> => {
      if (!documentId) return null;

      try {
        const result = await invoke<DocumentChange>('apply_character_style', {
          docId: documentId,
          styleId,
        });
        await refreshInspector();
        return result;
      } catch (e) {
        console.error('Failed to apply character style:', e);
        return null;
      }
    },
    [documentId, refreshInspector]
  );

  // Apply direct formatting
  const applyDirectFormatting = useCallback(
    async (
      paragraphProps?: ParagraphProperties,
      characterProps?: CharacterProperties
    ): Promise<DocumentChange | null> => {
      if (!documentId) return null;

      try {
        const result = await invoke<DocumentChange>('apply_direct_formatting', {
          docId: documentId,
          paragraphProps: paragraphProps || null,
          characterProps: characterProps || null,
        });
        await refreshInspector();
        return result;
      } catch (e) {
        console.error('Failed to apply direct formatting:', e);
        return null;
      }
    },
    [documentId, refreshInspector]
  );

  // Clear direct formatting
  const clearDirectFormatting = useCallback(
    async (
      clearParagraph: boolean,
      clearCharacter: boolean
    ): Promise<DocumentChange | null> => {
      if (!documentId) return null;

      try {
        const result = await invoke<DocumentChange>('clear_direct_formatting', {
          docId: documentId,
          clearParagraph,
          clearCharacter,
        });
        await refreshInspector();
        return result;
      } catch (e) {
        console.error('Failed to clear direct formatting:', e);
        return null;
      }
    },
    [documentId, refreshInspector]
  );

  // Create a new style
  const createStyle = useCallback(
    async (style: Partial<Style>): Promise<Style | null> => {
      if (!documentId) return null;

      try {
        const result = await invoke<Style>('create_style', {
          docId: documentId,
          name: style.name || 'New Style',
          styleType: style.styleType || 'paragraph',
          basedOn: style.basedOn || null,
          paragraphProps: style.paragraphProps || null,
          characterProps: style.characterProps || null,
        });
        await refreshStyles();
        return result;
      } catch (e) {
        console.error('Failed to create style:', e);
        return null;
      }
    },
    [documentId, refreshStyles]
  );

  // Modify an existing style
  const modifyStyle = useCallback(
    async (styleId: string, updates: Partial<Style>): Promise<Style | null> => {
      if (!documentId) return null;

      try {
        const result = await invoke<Style>('modify_style', {
          docId: documentId,
          styleId,
          name: updates.name || null,
          basedOn: updates.basedOn || null,
          paragraphProps: updates.paragraphProps || null,
          characterProps: updates.characterProps || null,
        });
        await refreshStyles();
        await refreshInspector();
        return result;
      } catch (e) {
        console.error('Failed to modify style:', e);
        return null;
      }
    },
    [documentId, refreshStyles, refreshInspector]
  );

  return {
    styles,
    loading,
    error,
    inspectorData,
    refreshStyles,
    getStyle,
    getResolvedStyle,
    applyParagraphStyle,
    applyCharacterStyle,
    applyDirectFormatting,
    clearDirectFormatting,
    createStyle,
    modifyStyle,
    refreshInspector,
  };
}

export default useStyles;
