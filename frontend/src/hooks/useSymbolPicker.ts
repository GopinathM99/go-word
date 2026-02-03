/**
 * useSymbolPicker - Symbol picker state management hook
 *
 * Features:
 * - State management for recent symbols (localStorage)
 * - Category filtering
 * - Search functionality by character name
 * - Integration with editor to insert symbol
 */

import { useState, useCallback, useEffect, useMemo } from 'react';

// =============================================================================
// Types
// =============================================================================

export interface SymbolInfo {
  /** The Unicode character */
  char: string;
  /** Unicode code point (e.g., "U+00B1") */
  code: string;
  /** Character name */
  name: string;
  /** Category this symbol belongs to */
  category: SymbolCategory;
}

export type SymbolCategory =
  | 'mathematical'
  | 'arrows'
  | 'currency'
  | 'greek'
  | 'punctuation'
  | 'letterlike'
  | 'geometric'
  | 'technical'
  | 'dingbats'
  | 'emoji';

export interface SymbolCategoryInfo {
  id: SymbolCategory;
  name: string;
  description: string;
}

export interface UnicodeBlock {
  name: string;
  start: number;
  end: number;
}

export interface UseSymbolPickerOptions {
  /** Maximum number of recent symbols to track */
  maxRecentSymbols?: number;
  /** Callback when symbol is selected for insertion */
  onInsertSymbol?: (symbol: string) => void;
  /** LocalStorage key for recent symbols */
  storageKey?: string;
}

export interface UseSymbolPickerReturn {
  /** All available symbols */
  symbols: SymbolInfo[];
  /** Filtered symbols based on current search/category */
  filteredSymbols: SymbolInfo[];
  /** Recent symbols */
  recentSymbols: string[];
  /** Current search query */
  searchQuery: string;
  /** Set search query */
  setSearchQuery: (query: string) => void;
  /** Current selected category (null = all) */
  selectedCategory: SymbolCategory | null;
  /** Set selected category */
  setSelectedCategory: (category: SymbolCategory | null) => void;
  /** Available categories */
  categories: SymbolCategoryInfo[];
  /** Insert a symbol */
  insertSymbol: (symbol: string) => void;
  /** Clear recent symbols */
  clearRecentSymbols: () => void;
  /** Get symbol info by character */
  getSymbolInfo: (char: string) => SymbolInfo | undefined;
  /** Unicode blocks for browsing */
  unicodeBlocks: UnicodeBlock[];
  /** Get characters in a Unicode block */
  getBlockCharacters: (block: UnicodeBlock) => SymbolInfo[];
}

// =============================================================================
// Constants - Symbol Data
// =============================================================================

const STORAGE_KEY_DEFAULT = 'word-processor-recent-symbols';
const MAX_RECENT_DEFAULT = 20;

export const SYMBOL_CATEGORIES: SymbolCategoryInfo[] = [
  { id: 'mathematical', name: 'Mathematical', description: 'Math operators and symbols' },
  { id: 'arrows', name: 'Arrows', description: 'Directional arrows' },
  { id: 'currency', name: 'Currency', description: 'Currency symbols' },
  { id: 'greek', name: 'Greek', description: 'Greek alphabet letters' },
  { id: 'punctuation', name: 'Punctuation', description: 'Special punctuation marks' },
  { id: 'letterlike', name: 'Letterlike', description: 'Letter-like symbols' },
  { id: 'geometric', name: 'Geometric', description: 'Geometric shapes' },
  { id: 'technical', name: 'Technical', description: 'Technical symbols' },
  { id: 'dingbats', name: 'Dingbats', description: 'Decorative symbols' },
  { id: 'emoji', name: 'Emoji', description: 'Common emoji symbols' },
];

export const COMMON_SYMBOLS: SymbolInfo[] = [
  // Mathematical symbols
  { char: '\u00B1', code: 'U+00B1', name: 'Plus-Minus Sign', category: 'mathematical' },
  { char: '\u00D7', code: 'U+00D7', name: 'Multiplication Sign', category: 'mathematical' },
  { char: '\u00F7', code: 'U+00F7', name: 'Division Sign', category: 'mathematical' },
  { char: '\u221A', code: 'U+221A', name: 'Square Root', category: 'mathematical' },
  { char: '\u221E', code: 'U+221E', name: 'Infinity', category: 'mathematical' },
  { char: '\u03C0', code: 'U+03C0', name: 'Greek Small Letter Pi', category: 'mathematical' },
  { char: '\u2211', code: 'U+2211', name: 'N-Ary Summation (Sigma)', category: 'mathematical' },
  { char: '\u222B', code: 'U+222B', name: 'Integral', category: 'mathematical' },
  { char: '\u2202', code: 'U+2202', name: 'Partial Differential', category: 'mathematical' },
  { char: '\u2260', code: 'U+2260', name: 'Not Equal To', category: 'mathematical' },
  { char: '\u2264', code: 'U+2264', name: 'Less-Than or Equal To', category: 'mathematical' },
  { char: '\u2265', code: 'U+2265', name: 'Greater-Than or Equal To', category: 'mathematical' },
  { char: '\u2248', code: 'U+2248', name: 'Almost Equal To', category: 'mathematical' },
  { char: '\u2261', code: 'U+2261', name: 'Identical To', category: 'mathematical' },
  { char: '\u00B2', code: 'U+00B2', name: 'Superscript Two', category: 'mathematical' },
  { char: '\u00B3', code: 'U+00B3', name: 'Superscript Three', category: 'mathematical' },
  { char: '\u00BC', code: 'U+00BC', name: 'Vulgar Fraction One Quarter', category: 'mathematical' },
  { char: '\u00BD', code: 'U+00BD', name: 'Vulgar Fraction One Half', category: 'mathematical' },
  { char: '\u00BE', code: 'U+00BE', name: 'Vulgar Fraction Three Quarters', category: 'mathematical' },
  { char: '\u2200', code: 'U+2200', name: 'For All', category: 'mathematical' },
  { char: '\u2203', code: 'U+2203', name: 'There Exists', category: 'mathematical' },
  { char: '\u2205', code: 'U+2205', name: 'Empty Set', category: 'mathematical' },
  { char: '\u2208', code: 'U+2208', name: 'Element Of', category: 'mathematical' },
  { char: '\u2209', code: 'U+2209', name: 'Not an Element Of', category: 'mathematical' },
  { char: '\u2229', code: 'U+2229', name: 'Intersection', category: 'mathematical' },
  { char: '\u222A', code: 'U+222A', name: 'Union', category: 'mathematical' },
  { char: '\u2282', code: 'U+2282', name: 'Subset Of', category: 'mathematical' },
  { char: '\u2283', code: 'U+2283', name: 'Superset Of', category: 'mathematical' },

  // Arrows
  { char: '\u2192', code: 'U+2192', name: 'Rightwards Arrow', category: 'arrows' },
  { char: '\u2190', code: 'U+2190', name: 'Leftwards Arrow', category: 'arrows' },
  { char: '\u2191', code: 'U+2191', name: 'Upwards Arrow', category: 'arrows' },
  { char: '\u2193', code: 'U+2193', name: 'Downwards Arrow', category: 'arrows' },
  { char: '\u2194', code: 'U+2194', name: 'Left Right Arrow', category: 'arrows' },
  { char: '\u2195', code: 'U+2195', name: 'Up Down Arrow', category: 'arrows' },
  { char: '\u21D2', code: 'U+21D2', name: 'Rightwards Double Arrow', category: 'arrows' },
  { char: '\u21D0', code: 'U+21D0', name: 'Leftwards Double Arrow', category: 'arrows' },
  { char: '\u21D1', code: 'U+21D1', name: 'Upwards Double Arrow', category: 'arrows' },
  { char: '\u21D3', code: 'U+21D3', name: 'Downwards Double Arrow', category: 'arrows' },
  { char: '\u21D4', code: 'U+21D4', name: 'Left Right Double Arrow', category: 'arrows' },
  { char: '\u21B5', code: 'U+21B5', name: 'Downwards Arrow with Corner Leftwards', category: 'arrows' },
  { char: '\u21AA', code: 'U+21AA', name: 'Rightwards Arrow with Hook', category: 'arrows' },
  { char: '\u21A9', code: 'U+21A9', name: 'Leftwards Arrow with Hook', category: 'arrows' },
  { char: '\u2196', code: 'U+2196', name: 'North West Arrow', category: 'arrows' },
  { char: '\u2197', code: 'U+2197', name: 'North East Arrow', category: 'arrows' },
  { char: '\u2198', code: 'U+2198', name: 'South East Arrow', category: 'arrows' },
  { char: '\u2199', code: 'U+2199', name: 'South West Arrow', category: 'arrows' },

  // Currency
  { char: '\u20AC', code: 'U+20AC', name: 'Euro Sign', category: 'currency' },
  { char: '\u00A3', code: 'U+00A3', name: 'Pound Sign', category: 'currency' },
  { char: '\u00A5', code: 'U+00A5', name: 'Yen Sign', category: 'currency' },
  { char: '\u20B9', code: 'U+20B9', name: 'Indian Rupee Sign', category: 'currency' },
  { char: '\u20BF', code: 'U+20BF', name: 'Bitcoin Sign', category: 'currency' },
  { char: '\u00A2', code: 'U+00A2', name: 'Cent Sign', category: 'currency' },
  { char: '\u20A9', code: 'U+20A9', name: 'Won Sign', category: 'currency' },
  { char: '\u20AB', code: 'U+20AB', name: 'Dong Sign', category: 'currency' },
  { char: '\u20BD', code: 'U+20BD', name: 'Ruble Sign', category: 'currency' },
  { char: '\u20B1', code: 'U+20B1', name: 'Peso Sign', category: 'currency' },
  { char: '\u20B4', code: 'U+20B4', name: 'Hryvnia Sign', category: 'currency' },
  { char: '\u0024', code: 'U+0024', name: 'Dollar Sign', category: 'currency' },

  // Greek Letters
  { char: '\u03B1', code: 'U+03B1', name: 'Greek Small Letter Alpha', category: 'greek' },
  { char: '\u03B2', code: 'U+03B2', name: 'Greek Small Letter Beta', category: 'greek' },
  { char: '\u03B3', code: 'U+03B3', name: 'Greek Small Letter Gamma', category: 'greek' },
  { char: '\u03B4', code: 'U+03B4', name: 'Greek Small Letter Delta', category: 'greek' },
  { char: '\u03B5', code: 'U+03B5', name: 'Greek Small Letter Epsilon', category: 'greek' },
  { char: '\u03B6', code: 'U+03B6', name: 'Greek Small Letter Zeta', category: 'greek' },
  { char: '\u03B7', code: 'U+03B7', name: 'Greek Small Letter Eta', category: 'greek' },
  { char: '\u03B8', code: 'U+03B8', name: 'Greek Small Letter Theta', category: 'greek' },
  { char: '\u03B9', code: 'U+03B9', name: 'Greek Small Letter Iota', category: 'greek' },
  { char: '\u03BA', code: 'U+03BA', name: 'Greek Small Letter Kappa', category: 'greek' },
  { char: '\u03BB', code: 'U+03BB', name: 'Greek Small Letter Lambda', category: 'greek' },
  { char: '\u03BC', code: 'U+03BC', name: 'Greek Small Letter Mu', category: 'greek' },
  { char: '\u03BD', code: 'U+03BD', name: 'Greek Small Letter Nu', category: 'greek' },
  { char: '\u03BE', code: 'U+03BE', name: 'Greek Small Letter Xi', category: 'greek' },
  { char: '\u03BF', code: 'U+03BF', name: 'Greek Small Letter Omicron', category: 'greek' },
  { char: '\u03C1', code: 'U+03C1', name: 'Greek Small Letter Rho', category: 'greek' },
  { char: '\u03C3', code: 'U+03C3', name: 'Greek Small Letter Sigma', category: 'greek' },
  { char: '\u03C4', code: 'U+03C4', name: 'Greek Small Letter Tau', category: 'greek' },
  { char: '\u03C5', code: 'U+03C5', name: 'Greek Small Letter Upsilon', category: 'greek' },
  { char: '\u03C6', code: 'U+03C6', name: 'Greek Small Letter Phi', category: 'greek' },
  { char: '\u03C7', code: 'U+03C7', name: 'Greek Small Letter Chi', category: 'greek' },
  { char: '\u03C8', code: 'U+03C8', name: 'Greek Small Letter Psi', category: 'greek' },
  { char: '\u03C9', code: 'U+03C9', name: 'Greek Small Letter Omega', category: 'greek' },
  { char: '\u0394', code: 'U+0394', name: 'Greek Capital Letter Delta', category: 'greek' },
  { char: '\u03A3', code: 'U+03A3', name: 'Greek Capital Letter Sigma', category: 'greek' },
  { char: '\u03A9', code: 'U+03A9', name: 'Greek Capital Letter Omega', category: 'greek' },
  { char: '\u03A6', code: 'U+03A6', name: 'Greek Capital Letter Phi', category: 'greek' },
  { char: '\u03A8', code: 'U+03A8', name: 'Greek Capital Letter Psi', category: 'greek' },

  // Punctuation
  { char: '\u2022', code: 'U+2022', name: 'Bullet', category: 'punctuation' },
  { char: '\u00B7', code: 'U+00B7', name: 'Middle Dot', category: 'punctuation' },
  { char: '\u2020', code: 'U+2020', name: 'Dagger', category: 'punctuation' },
  { char: '\u2021', code: 'U+2021', name: 'Double Dagger', category: 'punctuation' },
  { char: '\u00A7', code: 'U+00A7', name: 'Section Sign', category: 'punctuation' },
  { char: '\u00B6', code: 'U+00B6', name: 'Pilcrow (Paragraph) Sign', category: 'punctuation' },
  { char: '\u2026', code: 'U+2026', name: 'Horizontal Ellipsis', category: 'punctuation' },
  { char: '\u2013', code: 'U+2013', name: 'En Dash', category: 'punctuation' },
  { char: '\u2014', code: 'U+2014', name: 'Em Dash', category: 'punctuation' },
  { char: '\u2018', code: 'U+2018', name: 'Left Single Quotation Mark', category: 'punctuation' },
  { char: '\u2019', code: 'U+2019', name: 'Right Single Quotation Mark', category: 'punctuation' },
  { char: '\u201C', code: 'U+201C', name: 'Left Double Quotation Mark', category: 'punctuation' },
  { char: '\u201D', code: 'U+201D', name: 'Right Double Quotation Mark', category: 'punctuation' },
  { char: '\u00AB', code: 'U+00AB', name: 'Left-Pointing Double Angle Quotation Mark', category: 'punctuation' },
  { char: '\u00BB', code: 'U+00BB', name: 'Right-Pointing Double Angle Quotation Mark', category: 'punctuation' },
  { char: '\u00A9', code: 'U+00A9', name: 'Copyright Sign', category: 'punctuation' },
  { char: '\u00AE', code: 'U+00AE', name: 'Registered Sign', category: 'punctuation' },
  { char: '\u2122', code: 'U+2122', name: 'Trade Mark Sign', category: 'punctuation' },
  { char: '\u00B0', code: 'U+00B0', name: 'Degree Sign', category: 'punctuation' },
  { char: '\u2032', code: 'U+2032', name: 'Prime (Minutes)', category: 'punctuation' },
  { char: '\u2033', code: 'U+2033', name: 'Double Prime (Seconds)', category: 'punctuation' },

  // Letterlike Symbols
  { char: '\u2113', code: 'U+2113', name: 'Script Small L', category: 'letterlike' },
  { char: '\u2116', code: 'U+2116', name: 'Numero Sign', category: 'letterlike' },
  { char: '\u2117', code: 'U+2117', name: 'Sound Recording Copyright', category: 'letterlike' },
  { char: '\u2118', code: 'U+2118', name: 'Script Capital P (Weierstrass)', category: 'letterlike' },
  { char: '\u211C', code: 'U+211C', name: 'Black-Letter Capital R (Real Part)', category: 'letterlike' },
  { char: '\u2111', code: 'U+2111', name: 'Black-Letter Capital I (Imaginary Part)', category: 'letterlike' },
  { char: '\u00AA', code: 'U+00AA', name: 'Feminine Ordinal Indicator', category: 'letterlike' },
  { char: '\u00BA', code: 'U+00BA', name: 'Masculine Ordinal Indicator', category: 'letterlike' },

  // Geometric Shapes
  { char: '\u25A0', code: 'U+25A0', name: 'Black Square', category: 'geometric' },
  { char: '\u25A1', code: 'U+25A1', name: 'White Square', category: 'geometric' },
  { char: '\u25B2', code: 'U+25B2', name: 'Black Up-Pointing Triangle', category: 'geometric' },
  { char: '\u25B3', code: 'U+25B3', name: 'White Up-Pointing Triangle', category: 'geometric' },
  { char: '\u25BC', code: 'U+25BC', name: 'Black Down-Pointing Triangle', category: 'geometric' },
  { char: '\u25BD', code: 'U+25BD', name: 'White Down-Pointing Triangle', category: 'geometric' },
  { char: '\u25C6', code: 'U+25C6', name: 'Black Diamond', category: 'geometric' },
  { char: '\u25C7', code: 'U+25C7', name: 'White Diamond', category: 'geometric' },
  { char: '\u25CB', code: 'U+25CB', name: 'White Circle', category: 'geometric' },
  { char: '\u25CF', code: 'U+25CF', name: 'Black Circle', category: 'geometric' },
  { char: '\u25D0', code: 'U+25D0', name: 'Circle with Left Half Black', category: 'geometric' },
  { char: '\u25D1', code: 'U+25D1', name: 'Circle with Right Half Black', category: 'geometric' },
  { char: '\u2605', code: 'U+2605', name: 'Black Star', category: 'geometric' },
  { char: '\u2606', code: 'U+2606', name: 'White Star', category: 'geometric' },

  // Technical Symbols
  { char: '\u2318', code: 'U+2318', name: 'Place of Interest (Command Key)', category: 'technical' },
  { char: '\u2325', code: 'U+2325', name: 'Option Key', category: 'technical' },
  { char: '\u21E7', code: 'U+21E7', name: 'Upwards White Arrow (Shift)', category: 'technical' },
  { char: '\u2303', code: 'U+2303', name: 'Up Arrowhead (Control)', category: 'technical' },
  { char: '\u232B', code: 'U+232B', name: 'Erase to the Left (Backspace)', category: 'technical' },
  { char: '\u2326', code: 'U+2326', name: 'Erase to the Right (Delete)', category: 'technical' },
  { char: '\u21A9', code: 'U+21A9', name: 'Leftwards Arrow with Hook (Return)', category: 'technical' },
  { char: '\u21E5', code: 'U+21E5', name: 'Rightwards Arrow to Bar (Tab)', category: 'technical' },
  { char: '\u238B', code: 'U+238B', name: 'Broken Circle with Northwest Arrow (Escape)', category: 'technical' },

  // Dingbats
  { char: '\u2713', code: 'U+2713', name: 'Check Mark', category: 'dingbats' },
  { char: '\u2714', code: 'U+2714', name: 'Heavy Check Mark', category: 'dingbats' },
  { char: '\u2717', code: 'U+2717', name: 'Ballot X', category: 'dingbats' },
  { char: '\u2718', code: 'U+2718', name: 'Heavy Ballot X', category: 'dingbats' },
  { char: '\u2764', code: 'U+2764', name: 'Heavy Black Heart', category: 'dingbats' },
  { char: '\u2665', code: 'U+2665', name: 'Black Heart Suit', category: 'dingbats' },
  { char: '\u2666', code: 'U+2666', name: 'Black Diamond Suit', category: 'dingbats' },
  { char: '\u2663', code: 'U+2663', name: 'Black Club Suit', category: 'dingbats' },
  { char: '\u2660', code: 'U+2660', name: 'Black Spade Suit', category: 'dingbats' },
  { char: '\u266A', code: 'U+266A', name: 'Eighth Note', category: 'dingbats' },
  { char: '\u266B', code: 'U+266B', name: 'Beamed Eighth Notes', category: 'dingbats' },
  { char: '\u263A', code: 'U+263A', name: 'White Smiling Face', category: 'dingbats' },
  { char: '\u263C', code: 'U+263C', name: 'White Sun with Rays', category: 'dingbats' },
  { char: '\u2602', code: 'U+2602', name: 'Umbrella', category: 'dingbats' },
  { char: '\u2744', code: 'U+2744', name: 'Snowflake', category: 'dingbats' },
  { char: '\u2702', code: 'U+2702', name: 'Black Scissors', category: 'dingbats' },
  { char: '\u270E', code: 'U+270E', name: 'Lower Right Pencil', category: 'dingbats' },
  { char: '\u270F', code: 'U+270F', name: 'Pencil', category: 'dingbats' },
  { char: '\u2709', code: 'U+2709', name: 'Envelope', category: 'dingbats' },
  { char: '\u260E', code: 'U+260E', name: 'Black Telephone', category: 'dingbats' },

  // Common Emoji (text-style)
  { char: '\u2615', code: 'U+2615', name: 'Hot Beverage', category: 'emoji' },
  { char: '\u2708', code: 'U+2708', name: 'Airplane', category: 'emoji' },
  { char: '\u231A', code: 'U+231A', name: 'Watch', category: 'emoji' },
  { char: '\u231B', code: 'U+231B', name: 'Hourglass', category: 'emoji' },
  { char: '\u2328', code: 'U+2328', name: 'Keyboard', category: 'emoji' },
  { char: '\u23F0', code: 'U+23F0', name: 'Alarm Clock', category: 'emoji' },
  { char: '\u23F3', code: 'U+23F3', name: 'Hourglass with Flowing Sand', category: 'emoji' },
  { char: '\u2600', code: 'U+2600', name: 'Black Sun with Rays', category: 'emoji' },
  { char: '\u2601', code: 'U+2601', name: 'Cloud', category: 'emoji' },
  { char: '\u2614', code: 'U+2614', name: 'Umbrella with Rain Drops', category: 'emoji' },
  { char: '\u26A1', code: 'U+26A1', name: 'High Voltage Sign', category: 'emoji' },
  { char: '\u2728', code: 'U+2728', name: 'Sparkles', category: 'emoji' },
  { char: '\u2B50', code: 'U+2B50', name: 'White Medium Star', category: 'emoji' },
];

export const UNICODE_BLOCKS: UnicodeBlock[] = [
  { name: 'Basic Latin', start: 0x0020, end: 0x007F },
  { name: 'Latin-1 Supplement', start: 0x00A0, end: 0x00FF },
  { name: 'Latin Extended-A', start: 0x0100, end: 0x017F },
  { name: 'Latin Extended-B', start: 0x0180, end: 0x024F },
  { name: 'Greek and Coptic', start: 0x0370, end: 0x03FF },
  { name: 'Cyrillic', start: 0x0400, end: 0x04FF },
  { name: 'General Punctuation', start: 0x2000, end: 0x206F },
  { name: 'Superscripts and Subscripts', start: 0x2070, end: 0x209F },
  { name: 'Currency Symbols', start: 0x20A0, end: 0x20CF },
  { name: 'Letterlike Symbols', start: 0x2100, end: 0x214F },
  { name: 'Number Forms', start: 0x2150, end: 0x218F },
  { name: 'Arrows', start: 0x2190, end: 0x21FF },
  { name: 'Mathematical Operators', start: 0x2200, end: 0x22FF },
  { name: 'Miscellaneous Technical', start: 0x2300, end: 0x23FF },
  { name: 'Box Drawing', start: 0x2500, end: 0x257F },
  { name: 'Block Elements', start: 0x2580, end: 0x259F },
  { name: 'Geometric Shapes', start: 0x25A0, end: 0x25FF },
  { name: 'Miscellaneous Symbols', start: 0x2600, end: 0x26FF },
  { name: 'Dingbats', start: 0x2700, end: 0x27BF },
  { name: 'Miscellaneous Mathematical Symbols-A', start: 0x27C0, end: 0x27EF },
  { name: 'Supplemental Arrows-A', start: 0x27F0, end: 0x27FF },
  { name: 'Braille Patterns', start: 0x2800, end: 0x28FF },
  { name: 'Supplemental Arrows-B', start: 0x2900, end: 0x297F },
  { name: 'Miscellaneous Mathematical Symbols-B', start: 0x2980, end: 0x29FF },
  { name: 'Supplemental Mathematical Operators', start: 0x2A00, end: 0x2AFF },
];

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Get code point string from character
 */
function getCodePoint(char: string): string {
  const codePoint = char.codePointAt(0);
  if (codePoint === undefined) return 'U+0000';
  return `U+${codePoint.toString(16).toUpperCase().padStart(4, '0')}`;
}

/**
 * Get character name from Unicode (simplified - uses basic names for common chars)
 */
function getCharacterName(char: string): string {
  // Check if it's in our predefined symbols
  const predefined = COMMON_SYMBOLS.find((s) => s.char === char);
  if (predefined) return predefined.name;

  // Default to code point description
  return `Unicode Character ${getCodePoint(char)}`;
}

/**
 * Load recent symbols from localStorage
 */
function loadRecentSymbols(storageKey: string): string[] {
  try {
    const stored = localStorage.getItem(storageKey);
    if (stored) {
      const parsed = JSON.parse(stored);
      if (Array.isArray(parsed)) {
        return parsed.filter((s) => typeof s === 'string');
      }
    }
  } catch (e) {
    console.warn('Failed to load recent symbols from localStorage:', e);
  }
  return [];
}

/**
 * Save recent symbols to localStorage
 */
function saveRecentSymbols(symbols: string[], storageKey: string): void {
  try {
    localStorage.setItem(storageKey, JSON.stringify(symbols));
  } catch (e) {
    console.warn('Failed to save recent symbols to localStorage:', e);
  }
}

// =============================================================================
// Hook Implementation
// =============================================================================

export function useSymbolPicker(options: UseSymbolPickerOptions = {}): UseSymbolPickerReturn {
  const {
    maxRecentSymbols = MAX_RECENT_DEFAULT,
    onInsertSymbol,
    storageKey = STORAGE_KEY_DEFAULT,
  } = options;

  const [recentSymbols, setRecentSymbols] = useState<string[]>(() =>
    loadRecentSymbols(storageKey)
  );
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<SymbolCategory | null>(null);

  // Save recent symbols to localStorage when they change
  useEffect(() => {
    saveRecentSymbols(recentSymbols, storageKey);
  }, [recentSymbols, storageKey]);

  // Filter symbols based on search and category
  const filteredSymbols = useMemo(() => {
    let filtered = COMMON_SYMBOLS;

    // Filter by category
    if (selectedCategory) {
      filtered = filtered.filter((s) => s.category === selectedCategory);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase().trim();
      filtered = filtered.filter(
        (s) =>
          s.name.toLowerCase().includes(query) ||
          s.char.includes(query) ||
          s.code.toLowerCase().includes(query)
      );
    }

    return filtered;
  }, [selectedCategory, searchQuery]);

  // Insert symbol handler
  const insertSymbol = useCallback(
    (symbol: string) => {
      // Add to recent symbols
      setRecentSymbols((prev) => {
        const filtered = prev.filter((s) => s !== symbol);
        const updated = [symbol, ...filtered].slice(0, maxRecentSymbols);
        return updated;
      });

      // Call the insert callback
      if (onInsertSymbol) {
        onInsertSymbol(symbol);
      }
    },
    [onInsertSymbol, maxRecentSymbols]
  );

  // Clear recent symbols
  const clearRecentSymbols = useCallback(() => {
    setRecentSymbols([]);
  }, []);

  // Get symbol info by character
  const getSymbolInfo = useCallback((char: string): SymbolInfo | undefined => {
    return COMMON_SYMBOLS.find((s) => s.char === char);
  }, []);

  // Get characters in a Unicode block
  const getBlockCharacters = useCallback((block: UnicodeBlock): SymbolInfo[] => {
    const chars: SymbolInfo[] = [];
    for (let codePoint = block.start; codePoint <= block.end; codePoint++) {
      try {
        const char = String.fromCodePoint(codePoint);
        // Skip control characters and non-printable chars
        if (codePoint < 0x20 || (codePoint >= 0x7F && codePoint < 0xA0)) continue;

        chars.push({
          char,
          code: getCodePoint(char),
          name: getCharacterName(char),
          category: 'technical', // Default category for block browsing
        });
      } catch {
        // Skip invalid code points
      }
    }
    return chars;
  }, []);

  return {
    symbols: COMMON_SYMBOLS,
    filteredSymbols,
    recentSymbols,
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    categories: SYMBOL_CATEGORIES,
    insertSymbol,
    clearRecentSymbols,
    getSymbolInfo,
    unicodeBlocks: UNICODE_BLOCKS,
    getBlockCharacters,
  };
}

export default useSymbolPicker;
