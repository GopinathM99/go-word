import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DocumentInfo, Selection, RenderModel, DocumentChange, Viewport } from './types';

export function useDocument() {
  const [documentId, setDocumentId] = useState<string | null>(null);
  const [document, setDocument] = useState<DocumentInfo | null>(null);
  const [selection, setSelection] = useState<Selection | null>(null);
  const [renderModel, setRenderModel] = useState<RenderModel | null>(null);

  // Create a new document on mount
  useEffect(() => {
    const initDocument = async () => {
      try {
        const id = await invoke<string>('create_document');
        setDocumentId(id);
        setDocument({
          id,
          path: null,
          dirty: false,
          currentPage: 1,
          totalPages: 1,
          wordCount: 0,
          language: 'English',
        });

        // Get initial layout
        const viewport: Viewport = {
          x: 0,
          y: 0,
          width: window.innerWidth,
          height: window.innerHeight,
        };
        const model = await invoke<RenderModel>('get_layout', { docId: id, viewport });
        setRenderModel(model);
      } catch (e) {
        console.error('Failed to create document:', e);
      }
    };

    initDocument();
  }, []);

  // Execute a command
  const executeCommand = useCallback(async (command: string, params?: Record<string, unknown>) => {
    if (!documentId) return;

    try {
      switch (command) {
        case 'undo': {
          const change = await invoke<DocumentChange>('undo', { docId: documentId });
          if (change.selection) setSelection(change.selection);
          break;
        }
        case 'redo': {
          const change = await invoke<DocumentChange>('redo', { docId: documentId });
          if (change.selection) setSelection(change.selection);
          break;
        }
        case 'save': {
          if (document?.path) {
            await invoke('save_document', { docId: documentId, path: document.path });
            setDocument(prev => prev ? { ...prev, dirty: false } : null);
          }
          break;
        }
        default: {
          // Send command to Rust core
          const commandPayload = JSON.stringify({ type: command, ...params });
          const change = await invoke<DocumentChange>('apply_command', {
            docId: documentId,
            command: commandPayload,
          });
          if (change.selection) setSelection(change.selection);
          break;
        }
      }

      // Refresh render model after command
      const viewport: Viewport = {
        x: 0,
        y: 0,
        width: window.innerWidth,
        height: window.innerHeight,
      };
      const model = await invoke<RenderModel>('get_layout', { docId: documentId, viewport });
      setRenderModel(model);
    } catch (e) {
      console.error('Command failed:', e);
    }
  }, [documentId, document?.path]);

  return {
    document,
    selection,
    renderModel,
    executeCommand,
  };
}
