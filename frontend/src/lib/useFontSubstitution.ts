import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FontSubstitutionSummary } from './types';

/**
 * Font resolution result from the backend
 */
export interface FontResolution {
  family: string;
  weight: string;
  style: string;
  was_substituted: boolean;
  warning: SubstitutionWarning | null;
}

/**
 * Substitution warning details
 */
export interface SubstitutionWarning {
  requested: string;
  substituted: string;
  reason: string;
}

/**
 * Hook for managing font substitution state and interactions
 */
export function useFontSubstitution() {
  const [summary, setSummary] = useState<FontSubstitutionSummary | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [neverShowForDocument, setNeverShowForDocument] = useState(false);
  const [availableFonts, setAvailableFonts] = useState<string[]>([]);

  /**
   * Fetch the current font substitution summary from the backend
   */
  const fetchSummary = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<FontSubstitutionSummary>('get_font_substitutions');
      setSummary(result);
      setDismissed(false);
    } catch (e) {
      console.error('Failed to fetch font substitutions:', e);
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Clear the font substitution summary on the backend
   */
  const clearSubstitutions = useCallback(async () => {
    try {
      await invoke('clear_font_substitutions');
      setSummary(null);
    } catch (e) {
      console.error('Failed to clear font substitutions:', e);
    }
  }, []);

  /**
   * Check if a specific font is available
   */
  const checkFontAvailable = useCallback(async (family: string): Promise<boolean> => {
    try {
      return await invoke<boolean>('is_font_available', { family });
    } catch (e) {
      console.error('Failed to check font availability:', e);
      return false;
    }
  }, []);

  /**
   * Resolve a font with fallback
   */
  const resolveFont = useCallback(async (
    family: string,
    weight?: string,
    style?: string
  ): Promise<FontResolution | null> => {
    try {
      return await invoke<FontResolution>('resolve_font', { family, weight, style });
    } catch (e) {
      console.error('Failed to resolve font:', e);
      return null;
    }
  }, []);

  /**
   * Fetch the list of available fonts
   */
  const fetchAvailableFonts = useCallback(async () => {
    try {
      const fonts = await invoke<string[]>('get_available_fonts');
      setAvailableFonts(fonts);
      return fonts;
    } catch (e) {
      console.error('Failed to fetch available fonts:', e);
      return [];
    }
  }, []);

  /**
   * Dismiss the notification temporarily
   */
  const dismiss = useCallback(() => {
    setDismissed(true);
  }, []);

  /**
   * Don't show notification again for this document
   */
  const dontShowAgain = useCallback(() => {
    setNeverShowForDocument(true);
    setDismissed(true);
  }, []);

  /**
   * Handle install fonts action
   * Opens system font installer or shows instructions
   */
  const handleInstallFonts = useCallback(async (fonts: string[]) => {
    // For now, just log the fonts that need to be installed
    // In a real implementation, this could open a font download page
    // or provide instructions for installing fonts
    console.log('Fonts to install:', fonts);

    // Could open a dialog or external link here
    // await invoke('open_font_installer', { fonts });
  }, []);

  // Determine if the notice should be shown
  const shouldShowNotice = !dismissed &&
    !neverShowForDocument &&
    summary !== null &&
    summary.substitutions.length > 0;

  return {
    // State
    summary: shouldShowNotice ? summary : null,
    isLoading,
    error,
    availableFonts,

    // Actions
    fetchSummary,
    clearSubstitutions,
    checkFontAvailable,
    resolveFont,
    fetchAvailableFonts,
    dismiss,
    dontShowAgain,
    handleInstallFonts,
  };
}

export default useFontSubstitution;
