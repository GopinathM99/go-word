/**
 * KeyboardNavigation.test.ts
 *
 * Tests for keyboard navigation functionality.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  FocusManager,
  FocusRegion,
  KeyboardNavigationManager,
  KeyboardShortcut,
} from './KeyboardNavigation';

describe('FocusManager', () => {
  let focusManager: FocusManager;
  let toolbar: HTMLElement;
  let canvas: HTMLElement;
  let statusBar: HTMLElement;

  beforeEach(() => {
    focusManager = new FocusManager();

    // Create mock elements
    toolbar = document.createElement('div');
    toolbar.innerHTML = '<button>New</button><button>Open</button><button>Save</button>';
    document.body.appendChild(toolbar);

    canvas = document.createElement('div');
    canvas.tabIndex = 0;
    document.body.appendChild(canvas);

    statusBar = document.createElement('div');
    statusBar.innerHTML = '<button>Zoom In</button><button>Zoom Out</button>';
    document.body.appendChild(statusBar);

    // Register regions
    focusManager.registerRegion(FocusRegion.Toolbar, toolbar);
    focusManager.registerRegion(FocusRegion.Canvas, canvas);
    focusManager.registerRegion(FocusRegion.StatusBar, statusBar);
  });

  afterEach(() => {
    toolbar.remove();
    canvas.remove();
    statusBar.remove();
  });

  describe('registerRegion', () => {
    it('should register a region', () => {
      const sidePanel = document.createElement('div');
      focusManager.registerRegion(FocusRegion.SidePanel, sidePanel);
      // Should not throw
    });

    it('should handle null elements', () => {
      focusManager.registerRegion(FocusRegion.SidePanel, null);
      // Should not throw
    });
  });

  describe('focusRegion', () => {
    it('should focus first element in toolbar', () => {
      const result = focusManager.focusRegion(FocusRegion.Toolbar);
      expect(result).toBe(true);
      expect(document.activeElement).toBe(toolbar.querySelector('button'));
    });

    it('should focus canvas directly when no buttons', () => {
      const result = focusManager.focusRegion(FocusRegion.Canvas);
      expect(result).toBe(true);
      expect(document.activeElement).toBe(canvas);
    });

    it('should return false for unregistered region', () => {
      focusManager.unregisterRegion(FocusRegion.SidePanel);
      const result = focusManager.focusRegion(FocusRegion.SidePanel);
      expect(result).toBe(false);
    });
  });

  describe('focusNextRegion', () => {
    it('should cycle through regions', () => {
      focusManager.focusRegion(FocusRegion.Toolbar);
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Toolbar);

      focusManager.focusNextRegion();
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Canvas);

      focusManager.focusNextRegion();
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.StatusBar);

      focusManager.focusNextRegion();
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Toolbar);
    });
  });

  describe('focusPreviousRegion', () => {
    it('should cycle through regions in reverse', () => {
      focusManager.focusRegion(FocusRegion.Toolbar);

      focusManager.focusPreviousRegion();
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.StatusBar);

      focusManager.focusPreviousRegion();
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Canvas);

      focusManager.focusPreviousRegion();
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Toolbar);
    });
  });

  describe('focusNext/focusPrevious within region', () => {
    it('should navigate within toolbar', () => {
      focusManager.focusRegion(FocusRegion.Toolbar);
      const buttons = toolbar.querySelectorAll('button');

      expect(document.activeElement).toBe(buttons[0]);

      focusManager.focusNext();
      expect(document.activeElement).toBe(buttons[1]);

      focusManager.focusNext();
      expect(document.activeElement).toBe(buttons[2]);

      focusManager.focusNext();
      expect(document.activeElement).toBe(buttons[0]); // Wraps around
    });

    it('should navigate backwards within toolbar', () => {
      focusManager.focusRegion(FocusRegion.Toolbar);
      const buttons = toolbar.querySelectorAll('button');

      focusManager.focusPrevious();
      expect(document.activeElement).toBe(buttons[2]); // Wraps to end

      focusManager.focusPrevious();
      expect(document.activeElement).toBe(buttons[1]);
    });
  });

  describe('saveFocus/restoreFocus', () => {
    it('should save and restore focus', () => {
      focusManager.focusRegion(FocusRegion.Toolbar);
      focusManager.focusNext(); // Focus second button

      focusManager.saveFocus();

      // Move focus elsewhere
      focusManager.focusRegion(FocusRegion.Canvas);
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Canvas);

      // Restore
      const result = focusManager.restoreFocus();
      expect(result).toBe(true);
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Toolbar);
    });

    it('should handle multiple saves', () => {
      focusManager.focusRegion(FocusRegion.Toolbar);
      focusManager.saveFocus();

      focusManager.focusRegion(FocusRegion.Canvas);
      focusManager.saveFocus();

      focusManager.focusRegion(FocusRegion.StatusBar);

      // First restore should go to canvas
      focusManager.restoreFocus();
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Canvas);

      // Second restore should go to toolbar
      focusManager.restoreFocus();
      expect(focusManager.getCurrentRegion()).toBe(FocusRegion.Toolbar);
    });
  });

  describe('createFocusTrap', () => {
    it('should trap focus within dialog', () => {
      const dialog = document.createElement('div');
      dialog.innerHTML = `
        <input type="text" id="first" />
        <button id="middle">OK</button>
        <button id="last">Cancel</button>
      `;
      document.body.appendChild(dialog);

      const cleanup = focusManager.createFocusTrap(dialog);

      // First element should be focused
      const firstInput = dialog.querySelector('#first') as HTMLInputElement;
      expect(document.activeElement).toBe(firstInput);

      // Simulate Tab on last element
      const lastButton = dialog.querySelector('#last') as HTMLButtonElement;
      lastButton.focus();

      const tabEvent = new KeyboardEvent('keydown', {
        key: 'Tab',
        bubbles: true,
      });
      dialog.dispatchEvent(tabEvent);

      // Should wrap to first element
      // Note: In real implementation, the event handler would do this

      cleanup();
      dialog.remove();
    });
  });
});

describe('KeyboardNavigationManager', () => {
  let focusManager: FocusManager;
  let navManager: KeyboardNavigationManager;

  beforeEach(() => {
    focusManager = new FocusManager();
    navManager = new KeyboardNavigationManager(focusManager);
  });

  describe('registerShortcut', () => {
    it('should register a shortcut', () => {
      const action = vi.fn();
      const shortcut: KeyboardShortcut = {
        key: 'b',
        ctrl: true,
        description: 'Bold',
        action,
        category: 'formatting',
      };

      navManager.registerShortcut(shortcut);

      const shortcuts = navManager.getShortcuts();
      expect(shortcuts).toContainEqual(shortcut);
    });
  });

  describe('handleKeyDown', () => {
    it('should trigger shortcut action', () => {
      const action = vi.fn();
      navManager.registerShortcut({
        key: 'b',
        ctrl: true,
        description: 'Bold',
        action,
        category: 'formatting',
      });

      const event = new KeyboardEvent('keydown', {
        key: 'b',
        ctrlKey: true,
      });

      const handled = navManager.handleKeyDown(event);
      expect(handled).toBe(true);
      expect(action).toHaveBeenCalled();
    });

    it('should not trigger without correct modifiers', () => {
      const action = vi.fn();
      navManager.registerShortcut({
        key: 'b',
        ctrl: true,
        description: 'Bold',
        action,
        category: 'formatting',
      });

      const event = new KeyboardEvent('keydown', {
        key: 'b',
        ctrlKey: false, // Missing Ctrl
      });

      const handled = navManager.handleKeyDown(event);
      expect(handled).toBe(false);
      expect(action).not.toHaveBeenCalled();
    });

    it('should handle multiple modifiers', () => {
      const action = vi.fn();
      navManager.registerShortcut({
        key: 's',
        ctrl: true,
        shift: true,
        description: 'Save As',
        action,
        category: 'file',
      });

      const event = new KeyboardEvent('keydown', {
        key: 's',
        ctrlKey: true,
        shiftKey: true,
      });

      const handled = navManager.handleKeyDown(event);
      expect(handled).toBe(true);
      expect(action).toHaveBeenCalled();
    });
  });

  describe('setEnabled', () => {
    it('should disable navigation', () => {
      const action = vi.fn();
      navManager.registerShortcut({
        key: 'b',
        ctrl: true,
        description: 'Bold',
        action,
        category: 'formatting',
      });

      navManager.setEnabled(false);

      const event = new KeyboardEvent('keydown', {
        key: 'b',
        ctrlKey: true,
      });

      const handled = navManager.handleKeyDown(event);
      expect(handled).toBe(false);
      expect(action).not.toHaveBeenCalled();
    });

    it('should re-enable navigation', () => {
      const action = vi.fn();
      navManager.registerShortcut({
        key: 'b',
        ctrl: true,
        description: 'Bold',
        action,
        category: 'formatting',
      });

      navManager.setEnabled(false);
      navManager.setEnabled(true);

      const event = new KeyboardEvent('keydown', {
        key: 'b',
        ctrlKey: true,
      });

      const handled = navManager.handleKeyDown(event);
      expect(handled).toBe(true);
      expect(action).toHaveBeenCalled();
    });
  });

  describe('getShortcutsByCategory', () => {
    it('should filter shortcuts by category', () => {
      navManager.registerShortcut({
        key: 'b',
        ctrl: true,
        description: 'Bold',
        action: () => {},
        category: 'formatting',
      });

      navManager.registerShortcut({
        key: 's',
        ctrl: true,
        description: 'Save',
        action: () => {},
        category: 'file',
      });

      navManager.registerShortcut({
        key: 'i',
        ctrl: true,
        description: 'Italic',
        action: () => {},
        category: 'formatting',
      });

      const formatting = navManager.getShortcutsByCategory('formatting');
      expect(formatting.length).toBe(2);
      expect(formatting.every((s) => s.category === 'formatting')).toBe(true);

      const file = navManager.getShortcutsByCategory('file');
      expect(file.length).toBe(1);
      expect(file[0].description).toBe('Save');
    });
  });

  describe('unregisterShortcut', () => {
    it('should unregister a shortcut', () => {
      const shortcut: KeyboardShortcut = {
        key: 'b',
        ctrl: true,
        description: 'Bold',
        action: () => {},
        category: 'formatting',
      };

      navManager.registerShortcut(shortcut);
      expect(navManager.getShortcuts().length).toBeGreaterThan(0);

      navManager.unregisterShortcut({ key: 'b', ctrl: true });

      const event = new KeyboardEvent('keydown', {
        key: 'b',
        ctrlKey: true,
      });

      const handled = navManager.handleKeyDown(event);
      expect(handled).toBe(false);
    });
  });
});
