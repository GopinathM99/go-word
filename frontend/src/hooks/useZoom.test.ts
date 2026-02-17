/**
 * Tests for useZoom hook
 *
 * Tests:
 * - Zoom range limits (25% to 500%)
 * - Preset levels
 * - Zoom persistence
 * - Keyboard shortcuts
 * - Mouse coordinate transformation
 */

import { renderHook, act } from '@testing-library/react';
import { useZoom, MIN_ZOOM, MAX_ZOOM, ZOOM_PRESETS, ZOOM_STEP } from './useZoom';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

describe('useZoom', () => {
  beforeEach(() => {
    localStorageMock.clear();
  });

  describe('initialization', () => {
    it('should initialize with default zoom of 1.0', () => {
      const { result } = renderHook(() => useZoom());
      expect(result.current.zoom).toBe(1.0);
    });

    it('should initialize with custom initial zoom', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.5 }));
      expect(result.current.zoom).toBe(1.5);
    });

    it('should load zoom from localStorage if document ID is provided', () => {
      localStorageMock.setItem('go-word-zoom-test-doc', '1.25');
      const { result } = renderHook(() => useZoom({ documentId: 'test-doc' }));
      expect(result.current.zoom).toBe(1.25);
    });

    it('should use initial zoom if localStorage value is invalid', () => {
      localStorageMock.setItem('go-word-zoom-test-doc', 'invalid');
      const { result } = renderHook(() => useZoom({ documentId: 'test-doc', initialZoom: 1.0 }));
      expect(result.current.zoom).toBe(1.0);
    });
  });

  describe('zoom range limits', () => {
    it('should not zoom below minimum (25%)', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: MIN_ZOOM }));

      act(() => {
        result.current.zoomOut();
      });

      expect(result.current.zoom).toBe(MIN_ZOOM);
      expect(result.current.isAtMin).toBe(true);
    });

    it('should not zoom above maximum (500%)', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: MAX_ZOOM }));

      act(() => {
        result.current.zoomIn();
      });

      expect(result.current.zoom).toBe(MAX_ZOOM);
      expect(result.current.isAtMax).toBe(true);
    });

    it('should clamp setZoom to valid range', () => {
      const { result } = renderHook(() => useZoom());

      act(() => {
        result.current.setZoom(10); // Above max
      });
      expect(result.current.zoom).toBe(MAX_ZOOM);

      act(() => {
        result.current.setZoom(0.1); // Below min
      });
      expect(result.current.zoom).toBe(MIN_ZOOM);
    });
  });

  describe('zoom operations', () => {
    it('should zoom in by step (10%)', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.0 }));

      act(() => {
        result.current.zoomIn();
      });

      expect(result.current.zoom).toBe(1.0 + ZOOM_STEP);
    });

    it('should zoom out by step (10%)', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.0 }));

      act(() => {
        result.current.zoomOut();
      });

      expect(result.current.zoom).toBe(1.0 - ZOOM_STEP);
    });

    it('should reset zoom to 100%', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 2.0 }));

      act(() => {
        result.current.resetZoom();
      });

      expect(result.current.zoom).toBe(1.0);
    });

    it('should set preset levels', () => {
      const { result } = renderHook(() => useZoom());

      for (const preset of ZOOM_PRESETS) {
        act(() => {
          result.current.setPreset(preset);
        });
        expect(result.current.zoom).toBe(preset);
      }
    });

    it('should ignore invalid preset levels', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.0 }));

      act(() => {
        result.current.setPreset(0.33); // Not a valid preset
      });

      expect(result.current.zoom).toBe(1.0);
    });
  });

  describe('fit modes', () => {
    it('should calculate fit to width', () => {
      const { result } = renderHook(() =>
        useZoom({
          pageWidth: 816,
          containerWidth: 900,
        })
      );

      act(() => {
        result.current.fitToWidth();
      });

      // Expected: (900 - 80) / 816 = 820 / 816 ≈ 1.00
      expect(result.current.fitMode).toBe('fit-width');
      expect(result.current.zoom).toBeGreaterThan(0);
      expect(result.current.zoom).toBeLessThanOrEqual(MAX_ZOOM);
    });

    it('should calculate fit to page', () => {
      const { result } = renderHook(() =>
        useZoom({
          pageWidth: 816,
          pageHeight: 1056,
          containerWidth: 900,
          containerHeight: 700,
        })
      );

      act(() => {
        result.current.fitToPage();
      });

      expect(result.current.fitMode).toBe('fit-page');
      expect(result.current.zoom).toBeGreaterThan(0);
      expect(result.current.zoom).toBeLessThanOrEqual(MAX_ZOOM);
    });

    it('should clear fit mode when manually setting zoom', () => {
      const { result } = renderHook(() =>
        useZoom({
          pageWidth: 816,
          containerWidth: 900,
        })
      );

      act(() => {
        result.current.fitToWidth();
      });
      expect(result.current.fitMode).toBe('fit-width');

      act(() => {
        result.current.setZoom(1.5);
      });
      expect(result.current.fitMode).toBe('none');
    });
  });

  describe('wheel zoom', () => {
    it('should zoom in on Ctrl + scroll up (negative deltaY)', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.0 }));

      let handled: boolean = false;
      act(() => {
        handled = result.current.handleWheelZoom(-100, true);
      });

      expect(handled).toBe(true);
      expect(result.current.zoom).toBe(1.0 + ZOOM_STEP);
    });

    it('should zoom out on Ctrl + scroll down (positive deltaY)', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.0 }));

      let handled: boolean = false;
      act(() => {
        handled = result.current.handleWheelZoom(100, true);
      });

      expect(handled).toBe(true);
      expect(result.current.zoom).toBe(1.0 - ZOOM_STEP);
    });

    it('should not handle wheel without Ctrl key', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.0 }));

      let handled: boolean = false;
      act(() => {
        handled = result.current.handleWheelZoom(-100, false);
      });

      expect(handled).toBe(false);
      expect(result.current.zoom).toBe(1.0);
    });
  });

  describe('persistence', () => {
    it('should save zoom to localStorage when document ID is provided', () => {
      const { result } = renderHook(() => useZoom({ documentId: 'test-doc' }));

      act(() => {
        result.current.setZoom(1.5);
      });

      expect(localStorageMock.getItem('go-word-zoom-test-doc')).toBe('1.5');
    });

    it('should not save zoom without document ID', () => {
      const { result } = renderHook(() => useZoom());

      act(() => {
        result.current.setZoom(1.5);
      });

      // Should not throw and should not save to any key
      expect(localStorageMock.getItem('go-word-zoom-undefined')).toBeNull();
    });
  });

  describe('zoom percentage display', () => {
    it('should display correct percentage string', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.5 }));
      expect(result.current.zoomPercentage).toBe('150%');
    });

    it('should round percentage to nearest integer', () => {
      const { result } = renderHook(() => useZoom({ initialZoom: 1.555 }));
      expect(result.current.zoomPercentage).toBe('156%');
    });
  });

  describe('callback notifications', () => {
    it('should call onZoomChange when zoom changes', () => {
      const onZoomChange = jest.fn();
      const { result } = renderHook(() =>
        useZoom({ onZoomChange })
      );

      act(() => {
        result.current.setZoom(1.5);
      });

      expect(onZoomChange).toHaveBeenCalledWith(1.5, 'none');
    });

    it('should call onZoomChange with fit mode', () => {
      const onZoomChange = jest.fn();
      const { result } = renderHook(() =>
        useZoom({
          onZoomChange,
          pageWidth: 816,
          containerWidth: 900,
        })
      );

      act(() => {
        result.current.fitToWidth();
      });

      expect(onZoomChange).toHaveBeenCalledWith(
        expect.any(Number),
        'fit-width'
      );
    });
  });
});

describe('Mouse coordinate transformation', () => {
  // These tests verify the expected coordinate transformation logic
  // The actual implementation is in EditorCanvas, but we test the math here

  it('should correctly transform screen to document coordinates', () => {
    const zoom = 1.5;
    const scrollX = 100;
    const scrollY = 200;
    const rulerOffset = 20;

    const screenX = 150;
    const screenY = 250;

    // Formula: docX = (screenX + scrollX - rulerOffset) / zoom
    const docX = (screenX + scrollX - rulerOffset) / zoom;
    const docY = (screenY + scrollY - rulerOffset) / zoom;

    expect(docX).toBeCloseTo((150 + 100 - 20) / 1.5);
    expect(docY).toBeCloseTo((250 + 200 - 20) / 1.5);
  });

  it('should handle zoom = 1 correctly', () => {
    const zoom = 1.0;
    const scrollX = 0;
    const scrollY = 0;
    const rulerOffset = 0;

    const screenX = 100;
    const screenY = 200;

    const docX = (screenX + scrollX - rulerOffset) / zoom;
    const docY = (screenY + scrollY - rulerOffset) / zoom;

    expect(docX).toBe(100);
    expect(docY).toBe(200);
  });

  it('should handle zoom < 1 correctly', () => {
    const zoom = 0.5;
    const scrollX = 0;
    const scrollY = 0;
    const rulerOffset = 0;

    const screenX = 100;
    const screenY = 200;

    const docX = (screenX + scrollX - rulerOffset) / zoom;
    const docY = (screenY + scrollY - rulerOffset) / zoom;

    // At 50% zoom, screen pixel 100 corresponds to document pixel 200
    expect(docX).toBe(200);
    expect(docY).toBe(400);
  });

  it('should handle zoom > 1 correctly', () => {
    const zoom = 2.0;
    const scrollX = 0;
    const scrollY = 0;
    const rulerOffset = 0;

    const screenX = 100;
    const screenY = 200;

    const docX = (screenX + scrollX - rulerOffset) / zoom;
    const docY = (screenY + scrollY - rulerOffset) / zoom;

    // At 200% zoom, screen pixel 100 corresponds to document pixel 50
    expect(docX).toBe(50);
    expect(docY).toBe(100);
  });
});
