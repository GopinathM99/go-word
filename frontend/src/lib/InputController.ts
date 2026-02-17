/**
 * InputController - Handles keyboard and IME input for the document editor
 *
 * This module provides:
 * - Keyboard event handling for printable characters and control keys
 * - IME composition support for international text input
 * - Modifier key combinations for common operations (copy, paste, undo, etc.)
 * - Command generation for the Rust backend
 */

import {
  Selection,
  CompositionState,
  InsertTextCommand,
  DeleteRangeCommand,
  SplitParagraphCommand,
  NavigateCommand,
  ClipboardCommand,
  HistoryCommand,
  SelectAllCommand,
  OpenHyperlinkDialogCommand,
  OpenBookmarkDialogCommand,
  SetParagraphAlignmentCommand,
  OpenParagraphDialogCommand,
  OpenSymbolDialogCommand,
  SetCursorPositionCommand,
  EditorCommand,
} from './types';

// Re-export command types for convenience
export type {
  EditorCommand,
  InsertTextCommand,
  DeleteRangeCommand,
  SplitParagraphCommand,
  NavigateCommand,
  ClipboardCommand,
  HistoryCommand,
  SelectAllCommand,
  OpenHyperlinkDialogCommand,
  OpenBookmarkDialogCommand,
  SetParagraphAlignmentCommand,
  OpenParagraphDialogCommand,
  OpenSymbolDialogCommand,
  SetCursorPositionCommand,
  CompositionState,
};

/**
 * Union type of all possible commands
 */
export type Command =
  | InsertTextCommand
  | DeleteRangeCommand
  | SplitParagraphCommand
  | NavigateCommand
  | ClipboardCommand
  | HistoryCommand
  | SelectAllCommand
  | OpenHyperlinkDialogCommand
  | OpenBookmarkDialogCommand
  | SetParagraphAlignmentCommand
  | OpenParagraphDialogCommand
  | OpenSymbolDialogCommand
  | SetCursorPositionCommand;

// =============================================================================
// Input Controller Configuration
// =============================================================================

export interface InputControllerConfig {
  onCommand: (command: Command) => void;
  onCompositionChange?: (state: CompositionState) => void;
  selection: Selection | null;
}

// =============================================================================
// Input Controller Class
// =============================================================================

export class InputController {
  private config: InputControllerConfig;
  private compositionState: CompositionState;

  constructor(config: InputControllerConfig) {
    this.config = config;
    this.compositionState = {
      isComposing: false,
      compositionText: '',
      compositionStart: null,
    };
  }

  /**
   * Update configuration (e.g., when selection changes)
   */
  updateConfig(config: Partial<InputControllerConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Check if currently in IME composition mode
   */
  isComposing(): boolean {
    return this.compositionState.isComposing;
  }

  /**
   * Get the current composition state
   */
  getCompositionState(): CompositionState {
    return { ...this.compositionState };
  }

  // ===========================================================================
  // Keyboard Event Handling
  // ===========================================================================

  /**
   * Handle keydown events
   * Returns true if the event was handled and should be prevented
   */
  handleKeyDown(event: KeyboardEvent): boolean {
    // Skip during IME composition - let composition events handle it
    if (this.compositionState.isComposing) {
      return false;
    }

    const { key, ctrlKey, metaKey, shiftKey, altKey } = event;

    // Detect platform modifier (Cmd on Mac, Ctrl on others)
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    const modifierKey = isMac ? metaKey : ctrlKey;

    // Handle Ctrl+Shift for RTL/LTR toggle (without any other key)
    // This is a common Windows convention for text direction switching
    if (modifierKey && shiftKey && (key === 'Shift' || key === 'Control' || key === 'Meta')) {
      // Wait for both keys to be pressed together without other keys
      return false; // Don't prevent, let it combine
    }

    // Handle modifier key combinations
    if (modifierKey) {
      return this.handleModifierKeyCombo(key, shiftKey, altKey);
    }

    // Handle navigation keys
    if (this.isNavigationKey(key)) {
      return this.handleNavigationKey(key, shiftKey, ctrlKey || metaKey, altKey);
    }

    // Handle special keys
    switch (key) {
      case 'Backspace':
        this.handleBackspace(ctrlKey || metaKey, altKey);
        return true;

      case 'Delete':
        this.handleDelete(ctrlKey || metaKey, altKey);
        return true;

      case 'Enter':
        this.handleEnter(shiftKey);
        return true;

      case 'Tab':
        this.handleTab(shiftKey);
        return true;

      case 'Escape':
        // Escape can be used to cancel operations
        return false;

      default:
        // Handle printable characters
        if (this.isPrintableKey(event)) {
          this.handlePrintableCharacter(key);
          return true;
        }
        return false;
    }
  }

  /**
   * Check if a key is a navigation key
   */
  private isNavigationKey(key: string): boolean {
    return [
      'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown',
      'Home', 'End', 'PageUp', 'PageDown'
    ].includes(key);
  }

  /**
   * Check if the keypress produces a printable character
   */
  private isPrintableKey(event: KeyboardEvent): boolean {
    const { key, ctrlKey, metaKey, altKey } = event;

    // Skip if any modifier is held (except shift)
    if (ctrlKey || metaKey || altKey) {
      return false;
    }

    // Single character keys are printable
    if (key.length === 1) {
      return true;
    }

    // Space is printable
    if (key === ' ' || key === 'Space') {
      return true;
    }

    return false;
  }

  /**
   * Handle modifier key combinations (Ctrl/Cmd + key)
   */
  private handleModifierKeyCombo(key: string, shiftKey: boolean, altKey: boolean): boolean {
    const lowerKey = key.toLowerCase();

    switch (lowerKey) {
      case 'c':
        this.config.onCommand({ type: 'Copy' });
        return true;

      case 'x':
        this.config.onCommand({ type: 'Cut' });
        return true;

      case 'v':
        this.config.onCommand({ type: 'Paste' });
        return true;

      case 'z':
        if (shiftKey) {
          // Ctrl+Shift+Z = Redo
          this.config.onCommand({ type: 'Redo' });
        } else {
          // Ctrl+Z = Undo
          this.config.onCommand({ type: 'Undo' });
        }
        return true;

      case 'y':
        // Ctrl+Y = Redo
        this.config.onCommand({ type: 'Redo' });
        return true;

      case 'a':
        // Ctrl+A = Select All
        this.config.onCommand({ type: 'SelectAll' });
        return true;

      case 'k':
        // Ctrl+K = Insert/Edit Hyperlink
        this.config.onCommand({ type: 'OpenHyperlinkDialog' });
        return true;

      case 'g':
        // Ctrl+G = Go To (opens bookmark dialog)
        this.config.onCommand({ type: 'OpenBookmarkDialog' });
        return true;

      case 's':
        if (shiftKey) {
          // Ctrl+Shift+S = Open Symbol Dialog
          this.config.onCommand({ type: 'OpenSymbolDialog' });
          return true;
        }
        // Ctrl+S = Save (let it pass through for browser/system handling)
        return false;

      // Paragraph alignment shortcuts
      case 'l':
        // Ctrl+L = Align Left
        this.config.onCommand({ type: 'SetParagraphAlignment', alignment: 'left' });
        return true;

      case 'e':
        // Ctrl+E = Center
        this.config.onCommand({ type: 'SetParagraphAlignment', alignment: 'center' });
        return true;

      case 'r':
        // Ctrl+R = Align Right
        this.config.onCommand({ type: 'SetParagraphAlignment', alignment: 'right' });
        return true;

      case 'j':
        // Ctrl+J = Justify
        this.config.onCommand({ type: 'SetParagraphAlignment', alignment: 'justify' });
        return true;

      case 'backspace':
        // Ctrl+Backspace = Delete word backward
        this.handleBackspace(true, altKey);
        return true;

      case 'delete':
        // Ctrl+Delete = Delete word forward
        this.handleDelete(true, altKey);
        return true;

      default:
        // Let other modifier combos pass through (e.g., Ctrl+S for save)
        return false;
    }
  }

  /**
   * Handle navigation keys (arrows, home, end, etc.)
   */
  private handleNavigationKey(
    key: string,
    shiftKey: boolean,
    modifierKey: boolean,
    altKey: boolean
  ): boolean {
    let direction: NavigateCommand['direction'];
    let unit: NavigateCommand['unit'] = 'character';

    // Determine direction
    switch (key) {
      case 'ArrowLeft':
        direction = 'left';
        break;
      case 'ArrowRight':
        direction = 'right';
        break;
      case 'ArrowUp':
        direction = 'up';
        break;
      case 'ArrowDown':
        direction = 'down';
        break;
      case 'Home':
        direction = 'home';
        break;
      case 'End':
        direction = 'end';
        break;
      case 'PageUp':
        direction = 'up';
        unit = 'paragraph'; // Page navigation
        break;
      case 'PageDown':
        direction = 'down';
        unit = 'paragraph'; // Page navigation
        break;
      default:
        return false;
    }

    // Determine unit based on modifiers
    if (modifierKey) {
      if (direction === 'left' || direction === 'right') {
        unit = 'word';
      } else if (direction === 'home' || direction === 'end') {
        unit = 'document';
      } else {
        unit = 'paragraph';
      }
    } else if (altKey) {
      // Alt + arrow = word navigation on Mac
      unit = 'word';
    } else if (direction === 'home' || direction === 'end') {
      unit = 'line';
    }

    this.config.onCommand({
      type: 'Navigate',
      direction,
      unit,
      extend: shiftKey,
    });

    return true;
  }

  /**
   * Handle Backspace key
   */
  private handleBackspace(modifierKey: boolean, altKey: boolean): void {
    let unit: DeleteRangeCommand['unit'] = 'character';

    if (modifierKey) {
      unit = 'word';
    } else if (altKey) {
      // Alt+Backspace = delete word on Mac
      unit = 'word';
    }

    this.config.onCommand({
      type: 'DeleteRange',
      direction: 'backward',
      unit,
    });
  }

  /**
   * Handle Delete key
   */
  private handleDelete(modifierKey: boolean, altKey: boolean): void {
    let unit: DeleteRangeCommand['unit'] = 'character';

    if (modifierKey) {
      unit = 'word';
    } else if (altKey) {
      unit = 'word';
    }

    this.config.onCommand({
      type: 'DeleteRange',
      direction: 'forward',
      unit,
    });
  }

  /**
   * Handle Enter key
   */
  private handleEnter(shiftKey: boolean): void {
    if (shiftKey) {
      // Shift+Enter = soft line break (insert newline without paragraph split)
      this.config.onCommand({
        type: 'InsertText',
        text: '\n',
      });
    } else {
      // Regular Enter = split paragraph
      this.config.onCommand({
        type: 'SplitParagraph',
      });
    }
  }

  /**
   * Handle Tab key
   */
  private handleTab(shiftKey: boolean): void {
    if (shiftKey) {
      // Shift+Tab could be used for outdent in lists
      // For now, just insert a tab character in reverse?
      // This is typically handled differently - skip for now
      return;
    }

    // Insert tab character
    this.config.onCommand({
      type: 'InsertText',
      text: '\t',
    });
  }

  /**
   * Handle printable character input
   */
  private handlePrintableCharacter(char: string): void {
    this.config.onCommand({
      type: 'InsertText',
      text: char,
    });
  }

  // ===========================================================================
  // IME Composition Handling
  // ===========================================================================

  /**
   * Handle compositionstart event
   * Called when the user begins IME composition
   */
  handleCompositionStart(_event: CompositionEvent): void {
    this.compositionState = {
      isComposing: true,
      compositionText: '',
      compositionStart: this.config.selection?.anchor ?? null,
    };

    this.notifyCompositionChange();
  }

  /**
   * Handle compositionupdate event
   * Called as the user types during IME composition
   */
  handleCompositionUpdate(event: CompositionEvent): void {
    this.compositionState = {
      ...this.compositionState,
      compositionText: event.data || '',
    };

    this.notifyCompositionChange();
  }

  /**
   * Handle compositionend event
   * Called when the user commits or cancels the composition
   */
  handleCompositionEnd(event: CompositionEvent): void {
    const finalText = event.data || '';

    // Reset composition state
    this.compositionState = {
      isComposing: false,
      compositionText: '',
      compositionStart: null,
    };

    this.notifyCompositionChange();

    // Commit the final text as a single InsertText command
    if (finalText.length > 0) {
      this.config.onCommand({
        type: 'InsertText',
        text: finalText,
      });
    }
  }

  /**
   * Cancel the current composition
   * Can be called externally (e.g., when user clicks elsewhere)
   */
  cancelComposition(): void {
    if (this.compositionState.isComposing) {
      this.compositionState = {
        isComposing: false,
        compositionText: '',
        compositionStart: null,
      };

      this.notifyCompositionChange();
    }
  }

  /**
   * Notify listener of composition state changes
   */
  private notifyCompositionChange(): void {
    if (this.config.onCompositionChange) {
      this.config.onCompositionChange({ ...this.compositionState });
    }
  }

  // ===========================================================================
  // Input Event Handling (for 'input' events)
  // ===========================================================================

  /**
   * Handle input events
   * This can be used for 'beforeinput' events which provide more information
   * about the intended input
   */
  handleInput(event: InputEvent): boolean {
    // During composition, let composition events handle it
    if (this.compositionState.isComposing) {
      return false;
    }

    const inputType = event.inputType;
    const data = event.data;

    switch (inputType) {
      case 'insertText':
        if (data) {
          this.config.onCommand({
            type: 'InsertText',
            text: data,
          });
          return true;
        }
        break;

      case 'insertLineBreak':
        this.config.onCommand({
          type: 'InsertText',
          text: '\n',
        });
        return true;

      case 'insertParagraph':
        this.config.onCommand({
          type: 'SplitParagraph',
        });
        return true;

      case 'deleteContentBackward':
        this.config.onCommand({
          type: 'DeleteRange',
          direction: 'backward',
          unit: 'character',
        });
        return true;

      case 'deleteContentForward':
        this.config.onCommand({
          type: 'DeleteRange',
          direction: 'forward',
          unit: 'character',
        });
        return true;

      case 'deleteWordBackward':
        this.config.onCommand({
          type: 'DeleteRange',
          direction: 'backward',
          unit: 'word',
        });
        return true;

      case 'deleteWordForward':
        this.config.onCommand({
          type: 'DeleteRange',
          direction: 'forward',
          unit: 'word',
        });
        return true;

      case 'deleteSoftLineBackward':
      case 'deleteHardLineBackward':
        this.config.onCommand({
          type: 'DeleteRange',
          direction: 'backward',
          unit: 'line',
        });
        return true;

      case 'deleteSoftLineForward':
      case 'deleteHardLineForward':
        this.config.onCommand({
          type: 'DeleteRange',
          direction: 'forward',
          unit: 'line',
        });
        return true;

      // Handle cut/copy/paste through input events
      case 'insertFromPaste':
        // Paste is handled via clipboard API
        return false;

      case 'deleteByCut':
        this.config.onCommand({ type: 'Cut' });
        return true;

      case 'historyUndo':
        this.config.onCommand({ type: 'Undo' });
        return true;

      case 'historyRedo':
        this.config.onCommand({ type: 'Redo' });
        return true;
    }

    return false;
  }
}

// =============================================================================
// React Hook for InputController
// =============================================================================

import { useRef, useCallback, useEffect } from 'react';

export interface UseInputControllerOptions {
  onCommand: (command: Command) => void;
  onCompositionChange?: (state: CompositionState) => void;
  selection: Selection | null;
}

export interface UseInputControllerReturn {
  handleKeyDown: (event: React.KeyboardEvent) => void;
  handleCompositionStart: (event: React.CompositionEvent) => void;
  handleCompositionUpdate: (event: React.CompositionEvent) => void;
  handleCompositionEnd: (event: React.CompositionEvent) => void;
  handleInput: (event: React.FormEvent<HTMLElement>) => void;
  isComposing: () => boolean;
  compositionState: CompositionState;
}

export function useInputController(options: UseInputControllerOptions): UseInputControllerReturn {
  const controllerRef = useRef<InputController | null>(null);

  // Initialize controller
  if (!controllerRef.current) {
    controllerRef.current = new InputController({
      onCommand: options.onCommand,
      onCompositionChange: options.onCompositionChange,
      selection: options.selection,
    });
  }

  // Update config when dependencies change
  useEffect(() => {
    controllerRef.current?.updateConfig({
      onCommand: options.onCommand,
      onCompositionChange: options.onCompositionChange,
      selection: options.selection,
    });
  }, [options.onCommand, options.onCompositionChange, options.selection]);

  const handleKeyDown = useCallback((event: React.KeyboardEvent) => {
    if (controllerRef.current?.handleKeyDown(event.nativeEvent)) {
      event.preventDefault();
      event.stopPropagation();
    }
  }, []);

  const handleCompositionStart = useCallback((event: React.CompositionEvent) => {
    controllerRef.current?.handleCompositionStart(event.nativeEvent);
  }, []);

  const handleCompositionUpdate = useCallback((event: React.CompositionEvent) => {
    controllerRef.current?.handleCompositionUpdate(event.nativeEvent);
  }, []);

  const handleCompositionEnd = useCallback((event: React.CompositionEvent) => {
    controllerRef.current?.handleCompositionEnd(event.nativeEvent);
  }, []);

  const handleInput = useCallback((event: React.FormEvent<HTMLElement>) => {
    const inputEvent = event.nativeEvent as InputEvent;
    if (controllerRef.current?.handleInput(inputEvent)) {
      event.preventDefault();
    }
  }, []);

  const isComposing = useCallback(() => {
    return controllerRef.current?.isComposing() ?? false;
  }, []);

  const compositionState = controllerRef.current?.getCompositionState() ?? {
    isComposing: false,
    compositionText: '',
    compositionStart: null,
  };

  return {
    handleKeyDown,
    handleCompositionStart,
    handleCompositionUpdate,
    handleCompositionEnd,
    handleInput,
    isComposing,
    compositionState,
  };
}
