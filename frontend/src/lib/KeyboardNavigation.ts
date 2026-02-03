/**
 * KeyboardNavigation.ts
 *
 * Provides full keyboard navigation for the document editor.
 * Implements WCAG 2.1 keyboard accessibility requirements.
 */

import { useEffect, useCallback, useRef } from 'react';

// =============================================================================
// Types
// =============================================================================

export enum FocusRegion {
  Toolbar = 'toolbar',
  Canvas = 'canvas',
  SidePanel = 'side-panel',
  Dialog = 'dialog',
  StatusBar = 'status-bar',
  Menu = 'menu',
}

export interface FocusTarget {
  region: FocusRegion;
  element: HTMLElement | null;
  subIndex?: number;
}

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean;
  description: string;
  action: () => void;
  category: 'navigation' | 'editing' | 'formatting' | 'file' | 'view';
}

export interface FocusManagerOptions {
  onRegionChange?: (region: FocusRegion, previousRegion: FocusRegion | null) => void;
  onFocusRestore?: () => void;
  trapFocusInDialogs?: boolean;
}

// =============================================================================
// FocusManager Class
// =============================================================================

export class FocusManager {
  private currentFocus: FocusTarget | null = null;
  private savedFocus: FocusTarget[] = [];
  private regionElements: Map<FocusRegion, HTMLElement | null> = new Map();
  private regionOrder: FocusRegion[] = [
    FocusRegion.Toolbar,
    FocusRegion.Canvas,
    FocusRegion.SidePanel,
    FocusRegion.StatusBar,
  ];
  private options: FocusManagerOptions;
  private focusableSelectors: string =
    'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), ' +
    'textarea:not([disabled]), [tabindex]:not([tabindex="-1"]), [contenteditable]';

  constructor(options: FocusManagerOptions = {}) {
    this.options = {
      trapFocusInDialogs: true,
      ...options,
    };
  }

  /**
   * Register a region element
   */
  registerRegion(region: FocusRegion, element: HTMLElement | null): void {
    this.regionElements.set(region, element);
  }

  /**
   * Unregister a region element
   */
  unregisterRegion(region: FocusRegion): void {
    this.regionElements.delete(region);
  }

  /**
   * Get current focus region
   */
  getCurrentRegion(): FocusRegion | null {
    return this.currentFocus?.region ?? null;
  }

  /**
   * Focus a specific region
   */
  focusRegion(region: FocusRegion): boolean {
    const element = this.regionElements.get(region);
    if (!element) return false;

    const previousRegion = this.currentFocus?.region ?? null;

    // Find first focusable element in the region
    const focusable = this.getFocusableElements(element);
    const target = focusable[0] ?? element;

    this.currentFocus = {
      region,
      element: target instanceof HTMLElement ? target : null,
      subIndex: 0,
    };

    if (target instanceof HTMLElement) {
      target.focus();
    }

    // Notify of region change
    if (previousRegion !== region && this.options.onRegionChange) {
      this.options.onRegionChange(region, previousRegion);
    }

    return true;
  }

  /**
   * Move focus to the next region in tab order
   */
  focusNextRegion(): boolean {
    const currentRegion = this.currentFocus?.region;
    let nextIndex = 0;

    if (currentRegion) {
      const currentIndex = this.regionOrder.indexOf(currentRegion);
      nextIndex = (currentIndex + 1) % this.regionOrder.length;
    }

    // Find next available region
    for (let i = 0; i < this.regionOrder.length; i++) {
      const region = this.regionOrder[(nextIndex + i) % this.regionOrder.length];
      if (this.regionElements.has(region) && this.regionElements.get(region)) {
        return this.focusRegion(region);
      }
    }

    return false;
  }

  /**
   * Move focus to the previous region in tab order
   */
  focusPreviousRegion(): boolean {
    const currentRegion = this.currentFocus?.region;
    let prevIndex = this.regionOrder.length - 1;

    if (currentRegion) {
      const currentIndex = this.regionOrder.indexOf(currentRegion);
      prevIndex = (currentIndex - 1 + this.regionOrder.length) % this.regionOrder.length;
    }

    // Find previous available region
    for (let i = 0; i < this.regionOrder.length; i++) {
      const region = this.regionOrder[(prevIndex - i + this.regionOrder.length) % this.regionOrder.length];
      if (this.regionElements.has(region) && this.regionElements.get(region)) {
        return this.focusRegion(region);
      }
    }

    return false;
  }

  /**
   * Move focus to next element within current region
   */
  focusNext(): boolean {
    if (!this.currentFocus?.element) return false;

    const region = this.regionElements.get(this.currentFocus.region);
    if (!region) return false;

    const focusable = this.getFocusableElements(region);
    const currentIndex = this.currentFocus.subIndex ?? 0;
    const nextIndex = (currentIndex + 1) % focusable.length;

    const target = focusable[nextIndex];
    if (target instanceof HTMLElement) {
      target.focus();
      this.currentFocus = {
        ...this.currentFocus,
        element: target,
        subIndex: nextIndex,
      };
      return true;
    }

    return false;
  }

  /**
   * Move focus to previous element within current region
   */
  focusPrevious(): boolean {
    if (!this.currentFocus?.element) return false;

    const region = this.regionElements.get(this.currentFocus.region);
    if (!region) return false;

    const focusable = this.getFocusableElements(region);
    const currentIndex = this.currentFocus.subIndex ?? 0;
    const prevIndex = (currentIndex - 1 + focusable.length) % focusable.length;

    const target = focusable[prevIndex];
    if (target instanceof HTMLElement) {
      target.focus();
      this.currentFocus = {
        ...this.currentFocus,
        element: target,
        subIndex: prevIndex,
      };
      return true;
    }

    return false;
  }

  /**
   * Save current focus state for later restoration
   */
  saveFocus(): void {
    if (this.currentFocus) {
      this.savedFocus.push({ ...this.currentFocus });
    }
  }

  /**
   * Restore previously saved focus
   */
  restoreFocus(): boolean {
    const saved = this.savedFocus.pop();
    if (!saved) return false;

    if (saved.element && document.body.contains(saved.element)) {
      saved.element.focus();
      this.currentFocus = saved;
      this.options.onFocusRestore?.();
      return true;
    }

    // Element no longer exists, try to focus region
    return this.focusRegion(saved.region);
  }

  /**
   * Get all focusable elements within a container
   */
  private getFocusableElements(container: HTMLElement): Element[] {
    return Array.from(container.querySelectorAll(this.focusableSelectors))
      .filter(el => {
        // Check if element is visible
        if (el instanceof HTMLElement) {
          return el.offsetWidth > 0 && el.offsetHeight > 0;
        }
        return false;
      });
  }

  /**
   * Create a focus trap for dialogs
   */
  createFocusTrap(dialogElement: HTMLElement): () => void {
    const focusable = this.getFocusableElements(dialogElement);
    const firstFocusable = focusable[0] as HTMLElement | undefined;
    const lastFocusable = focusable[focusable.length - 1] as HTMLElement | undefined;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        // Shift + Tab
        if (document.activeElement === firstFocusable) {
          e.preventDefault();
          lastFocusable?.focus();
        }
      } else {
        // Tab
        if (document.activeElement === lastFocusable) {
          e.preventDefault();
          firstFocusable?.focus();
        }
      }
    };

    dialogElement.addEventListener('keydown', handleKeyDown);

    // Focus first element
    firstFocusable?.focus();

    // Return cleanup function
    return () => {
      dialogElement.removeEventListener('keydown', handleKeyDown);
    };
  }

  /**
   * Track focus changes
   */
  handleFocusChange(element: HTMLElement): void {
    // Determine which region the focused element is in
    for (const [region, regionElement] of this.regionElements) {
      if (regionElement?.contains(element)) {
        const previousRegion = this.currentFocus?.region;
        const focusable = this.getFocusableElements(regionElement);
        const index = focusable.indexOf(element);

        this.currentFocus = {
          region,
          element,
          subIndex: index >= 0 ? index : 0,
        };

        if (previousRegion !== region && this.options.onRegionChange) {
          this.options.onRegionChange(region, previousRegion ?? null);
        }

        return;
      }
    }
  }
}

// =============================================================================
// KeyboardNavigationManager Class
// =============================================================================

export class KeyboardNavigationManager {
  private shortcuts: Map<string, KeyboardShortcut> = new Map();
  private focusManager: FocusManager;
  private enabled: boolean = true;

  constructor(focusManager: FocusManager) {
    this.focusManager = focusManager;
    this.registerDefaultShortcuts();
  }

  /**
   * Register default keyboard shortcuts
   */
  private registerDefaultShortcuts(): void {
    // Navigation shortcuts
    this.registerShortcut({
      key: 'F6',
      description: 'Cycle between panes',
      action: () => this.focusManager.focusNextRegion(),
      category: 'navigation',
    });

    this.registerShortcut({
      key: 'F6',
      shift: true,
      description: 'Cycle between panes (reverse)',
      action: () => this.focusManager.focusPreviousRegion(),
      category: 'navigation',
    });

    this.registerShortcut({
      key: 'F6',
      ctrl: true,
      description: 'Cycle panels',
      action: () => this.focusManager.focusNextRegion(),
      category: 'navigation',
    });

    this.registerShortcut({
      key: 'Escape',
      description: 'Close dialogs, cancel operations',
      action: () => {
        // This will be overridden by dialog-specific handlers
        this.focusManager.restoreFocus();
      },
      category: 'navigation',
    });
  }

  /**
   * Generate a key for the shortcut map
   */
  private getShortcutKey(shortcut: Partial<KeyboardShortcut>): string {
    const parts: string[] = [];
    if (shortcut.ctrl) parts.push('Ctrl');
    if (shortcut.shift) parts.push('Shift');
    if (shortcut.alt) parts.push('Alt');
    if (shortcut.meta) parts.push('Meta');
    if (shortcut.key) parts.push(shortcut.key.toLowerCase());
    return parts.join('+');
  }

  /**
   * Register a keyboard shortcut
   */
  registerShortcut(shortcut: KeyboardShortcut): void {
    const key = this.getShortcutKey(shortcut);
    this.shortcuts.set(key, shortcut);
  }

  /**
   * Unregister a keyboard shortcut
   */
  unregisterShortcut(shortcut: Partial<KeyboardShortcut>): void {
    const key = this.getShortcutKey(shortcut);
    this.shortcuts.delete(key);
  }

  /**
   * Handle a keyboard event
   */
  handleKeyDown(event: KeyboardEvent): boolean {
    if (!this.enabled) return false;

    const key = this.getShortcutKey({
      key: event.key,
      ctrl: event.ctrlKey,
      shift: event.shiftKey,
      alt: event.altKey,
      meta: event.metaKey,
    });

    const shortcut = this.shortcuts.get(key);
    if (shortcut) {
      event.preventDefault();
      shortcut.action();
      return true;
    }

    return false;
  }

  /**
   * Enable or disable keyboard navigation
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
  }

  /**
   * Get all registered shortcuts
   */
  getShortcuts(): KeyboardShortcut[] {
    return Array.from(this.shortcuts.values());
  }

  /**
   * Get shortcuts by category
   */
  getShortcutsByCategory(category: string): KeyboardShortcut[] {
    return this.getShortcuts().filter(s => s.category === category);
  }
}

// =============================================================================
// React Hooks
// =============================================================================

/**
 * Hook for managing focus within the application
 */
export function useFocusManager(options: FocusManagerOptions = {}) {
  const focusManagerRef = useRef<FocusManager | null>(null);

  if (!focusManagerRef.current) {
    focusManagerRef.current = new FocusManager(options);
  }

  const registerRegion = useCallback((region: FocusRegion, element: HTMLElement | null) => {
    focusManagerRef.current?.registerRegion(region, element);
  }, []);

  const unregisterRegion = useCallback((region: FocusRegion) => {
    focusManagerRef.current?.unregisterRegion(region);
  }, []);

  const focusRegion = useCallback((region: FocusRegion) => {
    return focusManagerRef.current?.focusRegion(region) ?? false;
  }, []);

  const focusNextRegion = useCallback(() => {
    return focusManagerRef.current?.focusNextRegion() ?? false;
  }, []);

  const focusPreviousRegion = useCallback(() => {
    return focusManagerRef.current?.focusPreviousRegion() ?? false;
  }, []);

  const focusNext = useCallback(() => {
    return focusManagerRef.current?.focusNext() ?? false;
  }, []);

  const focusPrevious = useCallback(() => {
    return focusManagerRef.current?.focusPrevious() ?? false;
  }, []);

  const saveFocus = useCallback(() => {
    focusManagerRef.current?.saveFocus();
  }, []);

  const restoreFocus = useCallback(() => {
    return focusManagerRef.current?.restoreFocus() ?? false;
  }, []);

  const createFocusTrap = useCallback((dialogElement: HTMLElement) => {
    return focusManagerRef.current?.createFocusTrap(dialogElement) ?? (() => {});
  }, []);

  const getCurrentRegion = useCallback(() => {
    return focusManagerRef.current?.getCurrentRegion() ?? null;
  }, []);

  return {
    focusManager: focusManagerRef.current,
    registerRegion,
    unregisterRegion,
    focusRegion,
    focusNextRegion,
    focusPreviousRegion,
    focusNext,
    focusPrevious,
    saveFocus,
    restoreFocus,
    createFocusTrap,
    getCurrentRegion,
  };
}

/**
 * Hook for keyboard navigation
 */
export function useKeyboardNavigation(
  focusManager: FocusManager,
  additionalShortcuts: KeyboardShortcut[] = []
) {
  const navManagerRef = useRef<KeyboardNavigationManager | null>(null);

  if (!navManagerRef.current) {
    navManagerRef.current = new KeyboardNavigationManager(focusManager);
  }

  // Register additional shortcuts
  useEffect(() => {
    const manager = navManagerRef.current;
    if (!manager) return;

    additionalShortcuts.forEach(shortcut => {
      manager.registerShortcut(shortcut);
    });

    return () => {
      additionalShortcuts.forEach(shortcut => {
        manager.unregisterShortcut(shortcut);
      });
    };
  }, [additionalShortcuts]);

  // Set up global keyboard listener
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      navManagerRef.current?.handleKeyDown(event);
    };

    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  const registerShortcut = useCallback((shortcut: KeyboardShortcut) => {
    navManagerRef.current?.registerShortcut(shortcut);
  }, []);

  const unregisterShortcut = useCallback((shortcut: Partial<KeyboardShortcut>) => {
    navManagerRef.current?.unregisterShortcut(shortcut);
  }, []);

  const setEnabled = useCallback((enabled: boolean) => {
    navManagerRef.current?.setEnabled(enabled);
  }, []);

  const getShortcuts = useCallback(() => {
    return navManagerRef.current?.getShortcuts() ?? [];
  }, []);

  return {
    navManager: navManagerRef.current,
    registerShortcut,
    unregisterShortcut,
    setEnabled,
    getShortcuts,
  };
}

/**
 * Hook for focus trap in dialogs
 */
export function useFocusTrap(
  dialogRef: React.RefObject<HTMLElement | null>,
  isOpen: boolean,
  focusManager: ReturnType<typeof useFocusManager>
) {
  useEffect(() => {
    if (!isOpen || !dialogRef.current) return;

    // Save current focus
    focusManager.saveFocus();

    // Create focus trap
    const cleanup = focusManager.createFocusTrap(dialogRef.current);

    return () => {
      cleanup();
      // Restore focus on close
      focusManager.restoreFocus();
    };
  }, [isOpen, dialogRef, focusManager]);
}

/**
 * Hook for skip links
 */
export function useSkipLinks() {
  const skipToMain = useCallback(() => {
    const main = document.querySelector('[role="main"], main, .editor-canvas');
    if (main instanceof HTMLElement) {
      main.focus();
    }
  }, []);

  const skipToToolbar = useCallback(() => {
    const toolbar = document.querySelector('[role="toolbar"], .toolbar');
    if (toolbar instanceof HTMLElement) {
      const firstButton = toolbar.querySelector('button');
      if (firstButton) {
        firstButton.focus();
      } else {
        toolbar.focus();
      }
    }
  }, []);

  const skipToStatusBar = useCallback(() => {
    const statusBar = document.querySelector('.status-bar');
    if (statusBar instanceof HTMLElement) {
      statusBar.focus();
    }
  }, []);

  return {
    skipToMain,
    skipToToolbar,
    skipToStatusBar,
  };
}

// =============================================================================
// Arrow Key Navigation for Toolbar
// =============================================================================

export function useToolbarNavigation(toolbarRef: React.RefObject<HTMLElement | null>) {
  useEffect(() => {
    const toolbar = toolbarRef.current;
    if (!toolbar) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      const buttons = Array.from(
        toolbar.querySelectorAll('button:not([disabled])')
      ) as HTMLButtonElement[];

      if (buttons.length === 0) return;

      const currentIndex = buttons.findIndex(b => b === document.activeElement);

      switch (event.key) {
        case 'ArrowRight':
        case 'ArrowDown': {
          event.preventDefault();
          const nextIndex = currentIndex < buttons.length - 1 ? currentIndex + 1 : 0;
          buttons[nextIndex]?.focus();
          break;
        }

        case 'ArrowLeft':
        case 'ArrowUp': {
          event.preventDefault();
          const prevIndex = currentIndex > 0 ? currentIndex - 1 : buttons.length - 1;
          buttons[prevIndex]?.focus();
          break;
        }

        case 'Home': {
          event.preventDefault();
          buttons[0]?.focus();
          break;
        }

        case 'End': {
          event.preventDefault();
          buttons[buttons.length - 1]?.focus();
          break;
        }
      }
    };

    toolbar.addEventListener('keydown', handleKeyDown);

    return () => {
      toolbar.removeEventListener('keydown', handleKeyDown);
    };
  }, [toolbarRef]);
}
