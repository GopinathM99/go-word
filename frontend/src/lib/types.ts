// IPC Types matching Rust definitions

export interface Position {
  nodeId: string;
  offset: number;
}

export interface Selection {
  anchor: Position;
  focus: Position;
}

export interface Viewport {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Color type matching Rust Color struct
export interface Color {
  r: number;
  g: number;
  b: number;
  a: number;
}

// Helper to convert Color to CSS string
export function colorToCss(color: Color): string {
  return `rgba(${color.r}, ${color.g}, ${color.b}, ${color.a / 255})`;
}

// GlyphRun render item - text rendering
export interface GlyphRunData {
  text: string;
  font_family: string;
  font_size: number;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  color: Color;
  x: number;
  y: number;
  hyperlink: HyperlinkRenderInfo | null;
  para_index?: number;
  line_start_char_offset?: number;
}

// Hyperlink information for rendering
export interface HyperlinkRenderInfo {
  node_id: string;
  target: string;
  tooltip: string | null;
  link_type: HyperlinkType;
}

// Type of hyperlink
export type HyperlinkType = 'External' | 'Internal' | 'Email';

// Rectangle render item
export interface RectangleData {
  bounds: Rect;
  fill: Color | null;
  stroke: Color | null;
  stroke_width: number;
}

// Caret render item
export interface CaretData {
  x: number;
  y: number;
  height: number;
  color: Color;
  line_text?: string;
  char_offset_in_line?: number;
  para_index?: number;
}

// Selection render item
export interface SelectionData {
  rects: Rect[];
  color: Color;
}

// Line render item
export interface LineData {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  color: Color;
  width: number;
}

// Image render item
export interface ImageRenderData {
  node_id: string;
  resource_id: string;
  bounds: Rect;
  rotation: number;
  alt_text: string | null;
  title: string | null;
  selected: boolean;
}

// =============================================================================
// Shape Render Types
// =============================================================================

/**
 * Shape type for rendering
 */
export type ShapeRenderType =
  | { kind: 'Rectangle' }
  | { kind: 'RoundedRectangle'; corner_radius: number }
  | { kind: 'Oval' }
  | { kind: 'Line' }
  | { kind: 'Arrow' }
  | { kind: 'DoubleArrow' }
  | { kind: 'Triangle' }
  | { kind: 'Diamond' }
  | { kind: 'Pentagon' }
  | { kind: 'Hexagon' }
  | { kind: 'Star'; points: number; inner_radius_ratio: number }
  | { kind: 'Callout'; tail_position: [number, number]; tail_width: number }
  | { kind: 'TextBox' }
  | { kind: 'RightArrowBlock' }
  | { kind: 'LeftArrowBlock' }
  | { kind: 'UpArrowBlock' }
  | { kind: 'DownArrowBlock' };

/**
 * Shape fill for rendering
 */
export type ShapeFillRender =
  | { type: 'Solid'; color: Color }
  | { type: 'Gradient'; colors: [Color, number][]; angle: number }
  | { type: 'None' };

/**
 * Dash style for shape strokes
 */
export type DashStyleRender = 'Solid' | 'Dash' | 'Dot' | 'DashDot' | 'DashDotDot';

/**
 * Shape stroke for rendering
 */
export interface ShapeStrokeRender {
  color: Color;
  width: number;
  dash_style: DashStyleRender;
}

/**
 * Shadow effect for rendering
 */
export interface ShadowRender {
  color: Color;
  offset_x: number;
  offset_y: number;
  blur_radius: number;
}

/**
 * Shape render data
 */
export interface ShapeRenderData {
  node_id: string;
  shape_type: ShapeRenderType;
  bounds: Rect;
  rotation: number;
  fill: ShapeFillRender | null;
  stroke: ShapeStrokeRender | null;
  shadow: ShadowRender | null;
  opacity: number;
  selected: boolean;
  flip_horizontal: boolean;
  flip_vertical: boolean;
}

// RenderItem - tagged union matching Rust enum with #[serde(tag = "type")]
export type RenderItem =
  | { type: 'GlyphRun' } & GlyphRunData
  | { type: 'Rectangle' } & RectangleData
  | { type: 'Caret' } & CaretData
  | { type: 'Selection' } & SelectionData
  | { type: 'Line' } & LineData
  | { type: 'Image' } & ImageRenderData
  | { type: 'Shape' } & ShapeRenderData
  | { type: 'TableCell' } & TableCellRenderInfo
  | { type: 'TableBorder' } & TableBorderRenderInfo;

export interface PageRender {
  page_index: number;
  width: number;
  height: number;
  items: RenderItem[];
}

export interface RenderModel {
  pages: PageRender[];
}

export interface DocumentChange {
  changedNodes: string[];
  dirtyPages: number[];
  selection: Selection | null;
}

export interface DocumentInfo {
  id: string;
  path: string | null;
  dirty: boolean;
  currentPage: number;
  totalPages: number;
  wordCount: number;
  language: string;
}

// Command types
export interface InputEvent {
  type: 'keydown' | 'compositionstart' | 'compositionupdate' | 'compositionend';
  key?: string;
  data?: string;
  modifiers: {
    ctrl: boolean;
    shift: boolean;
    alt: boolean;
    meta: boolean;
  };
}

// =============================================================================
// Editor Command Types - Operations sent to the Rust backend
// =============================================================================

/**
 * Insert text at the current caret position
 */
export interface InsertTextCommand {
  type: 'InsertText';
  text: string;
}

/**
 * Delete a range of text
 */
export interface DeleteRangeCommand {
  type: 'DeleteRange';
  direction: 'backward' | 'forward';
  unit: 'character' | 'word' | 'line';
}

/**
 * Split the current paragraph (handle Enter key)
 */
export interface SplitParagraphCommand {
  type: 'SplitParagraph';
}

/**
 * Navigate the caret
 */
export interface NavigateCommand {
  type: 'Navigate';
  direction: 'left' | 'right' | 'up' | 'down' | 'home' | 'end';
  unit: 'character' | 'word' | 'line' | 'paragraph' | 'document';
  extend: boolean; // true if extending selection (shift held)
}

/**
 * Clipboard operations
 */
export interface ClipboardCommand {
  type: 'Copy' | 'Cut' | 'Paste';
}

/**
 * Undo/Redo operations
 */
export interface HistoryCommand {
  type: 'Undo' | 'Redo';
}

/**
 * Select all content
 */
export interface SelectAllCommand {
  type: 'SelectAll';
}

/**
 * Set cursor position directly (e.g., from mouse click)
 */
export interface SetCursorPositionCommand {
  type: 'SetCursorPosition';
  paragraph: number;
  offset: number;
}

/**
 * Hyperlink target type
 */
export type HyperlinkTargetType = 'external' | 'internal' | 'email';

/**
 * Hyperlink data for creating/editing hyperlinks
 */
export interface HyperlinkData {
  targetType: HyperlinkTargetType;
  url?: string;
  bookmark?: string;
  email?: string;
  subject?: string;
  tooltip?: string;
  displayText?: string;
}

/**
 * Insert a hyperlink at the current selection
 */
export interface InsertHyperlinkCommand {
  type: 'InsertHyperlink';
  data: HyperlinkData;
}

/**
 * Remove a hyperlink (keep text)
 */
export interface RemoveHyperlinkCommand {
  type: 'RemoveHyperlink';
  hyperlinkId?: string;
}

/**
 * Edit an existing hyperlink
 */
export interface EditHyperlinkCommand {
  type: 'EditHyperlink';
  hyperlinkId?: string;
  data: Partial<HyperlinkData>;
}

/**
 * Open hyperlink dialog
 */
export interface OpenHyperlinkDialogCommand {
  type: 'OpenHyperlinkDialog';
}

// =============================================================================
// Image Commands
// =============================================================================

/**
 * Image wrap type
 */
export type ImageWrapType = 'Inline' | 'Square' | 'Tight' | 'Behind' | 'InFront';

/**
 * Image properties for insert/update
 */
export interface ImageProperties {
  width?: number;
  height?: number;
  wrapType?: ImageWrapType;
  rotation?: number;
  altText?: string;
  title?: string;
  lockAspectRatio?: boolean;
}

/**
 * Insert an image at the current selection
 */
export interface InsertImageCommand {
  type: 'InsertImage';
  resourceId: string;
  originalWidth: number;
  originalHeight: number;
  properties?: ImageProperties;
}

/**
 * Delete an image
 */
export interface DeleteImageCommand {
  type: 'DeleteImage';
  imageId: string;
}

/**
 * Resize an image
 */
export interface ResizeImageCommand {
  type: 'ResizeImage';
  imageId: string;
  width: number;
  height: number;
}

/**
 * Set image wrap type
 */
export interface SetImageWrapCommand {
  type: 'SetImageWrap';
  imageId: string;
  wrapType: ImageWrapType;
}

/**
 * Update image properties
 */
export interface UpdateImagePropertiesCommand {
  type: 'UpdateImageProperties';
  imageId: string;
  properties: Partial<ImageProperties>;
}

/**
 * Open image properties dialog
 */
export interface OpenImageDialogCommand {
  type: 'OpenImageDialog';
  imageId?: string;
}

// =============================================================================
// Shape Commands
// =============================================================================

/**
 * Shape type names for commands
 */
export type ShapeTypeName =
  | 'Rectangle'
  | 'RoundedRectangle'
  | 'Oval'
  | 'Line'
  | 'Arrow'
  | 'DoubleArrow'
  | 'Triangle'
  | 'Diamond'
  | 'Pentagon'
  | 'Hexagon'
  | 'Star'
  | 'Callout'
  | 'TextBox'
  | 'RightArrowBlock'
  | 'LeftArrowBlock'
  | 'UpArrowBlock'
  | 'DownArrowBlock';

/**
 * Shape wrap type (same as image)
 */
export type ShapeWrapType = 'Inline' | 'Square' | 'Tight' | 'Behind' | 'InFront';

/**
 * Shape fill type for commands
 */
export type ShapeFillType =
  | { type: 'Solid'; color: Color }
  | { type: 'Gradient'; colors: [Color, number][]; angle: number }
  | { type: 'None' };

/**
 * Shape stroke type for commands
 */
export interface ShapeStrokeType {
  color: Color;
  width: number;
  dashStyle: DashStyleRender;
}

/**
 * Shape properties for insert/update
 */
export interface ShapeProperties {
  width?: number;
  height?: number;
  wrapType?: ShapeWrapType;
  rotation?: number;
  fill?: ShapeFillType | null;
  stroke?: ShapeStrokeType | null;
  name?: string;
  altText?: string;
  opacity?: number;
  flipHorizontal?: boolean;
  flipVertical?: boolean;
}

/**
 * Insert a shape at the current selection
 */
export interface InsertShapeCommand {
  type: 'InsertShape';
  shapeType: ShapeTypeName;
  width: number;
  height: number;
  properties?: ShapeProperties;
}

/**
 * Delete a shape
 */
export interface DeleteShapeCommand {
  type: 'DeleteShape';
  shapeId: string;
}

/**
 * Resize a shape
 */
export interface ResizeShapeCommand {
  type: 'ResizeShape';
  shapeId: string;
  width: number;
  height: number;
}

/**
 * Move a shape (for floating shapes)
 */
export interface MoveShapeCommand {
  type: 'MoveShape';
  shapeId: string;
  offsetX: number;
  offsetY: number;
}

/**
 * Set shape fill
 */
export interface SetShapeFillCommand {
  type: 'SetShapeFill';
  shapeId: string;
  fill: ShapeFillType | null;
}

/**
 * Set shape stroke
 */
export interface SetShapeStrokeCommand {
  type: 'SetShapeStroke';
  shapeId: string;
  stroke: ShapeStrokeType | null;
}

/**
 * Rotate a shape
 */
export interface RotateShapeCommand {
  type: 'RotateShape';
  shapeId: string;
  angle: number;
}

/**
 * Set shape wrap type
 */
export interface SetShapeWrapCommand {
  type: 'SetShapeWrap';
  shapeId: string;
  wrapType: ShapeWrapType;
}

/**
 * Update shape properties
 */
export interface UpdateShapePropertiesCommand {
  type: 'UpdateShapeProperties';
  shapeId: string;
  properties: Partial<ShapeProperties>;
}

/**
 * Open shape properties dialog
 */
export interface OpenShapeDialogCommand {
  type: 'OpenShapeDialog';
  shapeId?: string;
}

/**
 * Open shape gallery
 */
export interface OpenShapeGalleryCommand {
  type: 'OpenShapeGallery';
}

/**
 * Shape-related commands
 */
export type ShapeCommand =
  | InsertShapeCommand
  | DeleteShapeCommand
  | ResizeShapeCommand
  | MoveShapeCommand
  | SetShapeFillCommand
  | SetShapeStrokeCommand
  | RotateShapeCommand
  | SetShapeWrapCommand
  | UpdateShapePropertiesCommand
  | OpenShapeDialogCommand
  | OpenShapeGalleryCommand;

/**
 * Union type of all possible editor commands
 */
export type EditorCommand =
  | InsertTextCommand
  | DeleteRangeCommand
  | SplitParagraphCommand
  | NavigateCommand
  | ClipboardCommand
  | HistoryCommand
  | SelectAllCommand
  | InsertHyperlinkCommand
  | RemoveHyperlinkCommand
  | EditHyperlinkCommand
  | OpenHyperlinkDialogCommand
  | InsertBookmarkCommand
  | DeleteBookmarkCommand
  | RenameBookmarkCommand
  | GoToBookmarkCommand
  | OpenBookmarkDialogCommand
  | SetParagraphAlignmentCommand
  | OpenParagraphDialogCommand
  | OpenSymbolDialogCommand
  | SetParagraphDirectionCommand
  | ToggleParagraphDirectionCommand
  | InsertImageCommand
  | DeleteImageCommand
  | ResizeImageCommand
  | SetImageWrapCommand
  | UpdateImagePropertiesCommand
  | OpenImageDialogCommand
  | ShapeCommand
  | SetCursorPositionCommand;

// =============================================================================
// IME Composition State
// =============================================================================

export interface CompositionState {
  isComposing: boolean;
  compositionText: string;
  compositionStart: Position | null;
}

// =============================================================================
// Application Settings Types
// =============================================================================

/**
 * Theme options for the application
 */
export type Theme = 'light' | 'dark' | 'system';

/**
 * Available UI languages
 */
export type Language = 'en' | 'es' | 'fr' | 'de' | 'zh' | 'ja' | 'ko' | 'pt' | 'ru' | 'ar';

/**
 * General application settings
 */
export interface GeneralSettings {
  language: Language;
  theme: Theme;
  recent_files_count: number;
}

/**
 * Text editing settings
 */
export interface EditingSettings {
  autosave_enabled: boolean;
  autosave_interval_seconds: number;
  default_font_family: string;
  default_font_size: number;
  show_spelling_errors: boolean;
  show_grammar_errors: boolean;
}

/**
 * Privacy and telemetry settings
 */
export interface PrivacySettings {
  telemetry_enabled: boolean;
  crash_reports_enabled: boolean;
}

/**
 * Complete application settings
 */
export interface AppSettings {
  general: GeneralSettings;
  editing: EditingSettings;
  privacy: PrivacySettings;
}

/**
 * Default application settings
 */
export const DEFAULT_SETTINGS: AppSettings = {
  general: {
    language: 'en',
    theme: 'system',
    recent_files_count: 10,
  },
  editing: {
    autosave_enabled: true,
    autosave_interval_seconds: 60,
    default_font_family: 'Times New Roman',
    default_font_size: 12,
    show_spelling_errors: true,
    show_grammar_errors: true,
  },
  privacy: {
    telemetry_enabled: false,
    crash_reports_enabled: true,
  },
};

/**
 * Language display names
 */
export const LANGUAGE_OPTIONS: Record<Language, string> = {
  en: 'English',
  es: 'Espanol',
  fr: 'Francais',
  de: 'Deutsch',
  zh: 'Chinese',
  ja: 'Japanese',
  ko: 'Korean',
  pt: 'Portugues',
  ru: 'Russian',
  ar: 'Arabic',
};

/**
 * Common font families available
 */
export const FONT_FAMILIES = [
  'Times New Roman',
  'Arial',
  'Calibri',
  'Georgia',
  'Verdana',
  'Tahoma',
  'Trebuchet MS',
  'Courier New',
  'Comic Sans MS',
  'Impact',
];

// =============================================================================
// Font Substitution Types - Matching Rust backend types
// =============================================================================

/**
 * Reason why a font was substituted
 */
export type SubstitutionReason =
  | 'NotInstalled'
  | 'VariantNotAvailable'
  | 'ScriptNotSupported'
  | 'FallbackToDefault';

/**
 * Font weight type
 */
export type FontWeightType = 'Normal' | 'Bold';

/**
 * Font style type
 */
export type FontStyleType = 'Normal' | 'Italic';

/**
 * Record of a single font substitution
 */
export interface FontSubstitutionRecord {
  /** Original requested font */
  requested_font: string;
  /** Font that was actually used */
  actual_font: string;
  /** Weight requested */
  requested_weight: FontWeightType;
  /** Style requested */
  requested_style: FontStyleType;
  /** Reason for substitution */
  reason: SubstitutionReason;
  /** Number of times this substitution occurred */
  occurrence_count: number;
}

/**
 * Summary of all font substitutions in a document
 */
export interface FontSubstitutionSummary {
  /** List of all substitutions */
  substitutions: FontSubstitutionRecord[];
  /** Total number of fonts that were substituted */
  total_substituted: number;
  /** Total number of fonts that were found */
  total_found: number;
}

// =============================================================================
// Style System Types
// =============================================================================

/**
 * Style type - paragraph, character, table, or numbering
 */
export type StyleType = 'paragraph' | 'character' | 'table' | 'numbering';

/**
 * Text alignment options
 */
export type TextAlignment = 'left' | 'center' | 'right' | 'justify';

/**
 * Line spacing configuration
 */
export type LineSpacing =
  | { type: 'Multiple'; value: number }
  | { type: 'Exact'; value: number }
  | { type: 'AtLeast'; value: number };

/**
 * Paragraph formatting properties
 */
export interface ParagraphProperties {
  alignment?: TextAlignment;
  indentLeft?: number;
  indentRight?: number;
  indentFirstLine?: number;
  spaceBefore?: number;
  spaceAfter?: number;
  lineSpacing?: LineSpacing;
}

/**
 * Character formatting properties
 */
export interface CharacterProperties {
  fontFamily?: string;
  fontSize?: number;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  color?: string;
  highlight?: string;
}

/**
 * Style definition
 */
export interface Style {
  id: string;
  name: string;
  styleType: StyleType;
  basedOn?: string;
  nextStyle?: string;
  builtIn: boolean;
  hidden: boolean;
  priority: number;
  paragraphProps: ParagraphProperties;
  characterProps: CharacterProperties;
}

/**
 * Resolved style with all inherited properties applied
 */
export interface ResolvedStyle {
  styleId: string;
  paragraphProps: ParagraphProperties;
  characterProps: CharacterProperties;
  inheritanceChain: string[];
}

/**
 * Source of a property value
 */
export type PropertySource =
  | { type: 'DirectFormatting' }
  | { type: 'Style'; styleId: string }
  | { type: 'Default' };

/**
 * Computed property with source tracking
 */
export interface ComputedProperty<T> {
  value: T;
  source: PropertySource;
}

/**
 * Computed character properties with sources for inspector
 */
export interface ComputedCharacterProperties {
  fontFamily: ComputedProperty<string>;
  fontSize: ComputedProperty<number>;
  bold: ComputedProperty<boolean>;
  italic: ComputedProperty<boolean>;
  underline: ComputedProperty<boolean>;
  color: ComputedProperty<string>;
}

/**
 * Computed paragraph properties with sources for inspector
 */
export interface ComputedParagraphProperties {
  alignment: ComputedProperty<TextAlignment>;
  indentLeft: ComputedProperty<number>;
  indentRight: ComputedProperty<number>;
  indentFirstLine: ComputedProperty<number>;
  spaceBefore: ComputedProperty<number>;
  spaceAfter: ComputedProperty<number>;
  lineSpacing: ComputedProperty<LineSpacing>;
}

/**
 * Style inspector data for the current selection
 */
export interface StyleInspectorData {
  paragraphStyleId?: string;
  characterStyleId?: string;
  paragraphProps: ComputedParagraphProperties;
  characterProps: ComputedCharacterProperties;
  hasDirectParagraphFormatting: boolean;
  hasDirectCharacterFormatting: boolean;
}

/**
 * Get display name for property source
 */
export function getPropertySourceName(source: PropertySource): string {
  switch (source.type) {
    case 'DirectFormatting':
      return 'Direct Formatting';
    case 'Style':
      return source.styleId;
    case 'Default':
      return 'Default';
  }
}

/**
 * Check if a property has direct formatting
 */
export function hasDirectFormatting(source: PropertySource): boolean {
  return source.type === 'DirectFormatting';
}

/**
 * Format line spacing for display
 */
export function formatLineSpacing(spacing: LineSpacing): string {
  switch (spacing.type) {
    case 'Multiple':
      if (spacing.value === 1.0) return 'Single';
      if (spacing.value === 1.5) return '1.5 lines';
      if (spacing.value === 2.0) return 'Double';
      return `${spacing.value}x`;
    case 'Exact':
      return `${spacing.value}pt`;
    case 'AtLeast':
      return `At least ${spacing.value}pt`;
  }
}

// =============================================================================
// Bookmark Types
// =============================================================================

/**
 * Bookmark data from the backend
 */
export interface BookmarkData {
  /** Bookmark ID */
  id: string;
  /** Bookmark name (unique within document) */
  name: string;
  /** Whether this is a point bookmark (vs range) */
  isPoint: boolean;
  /** Preview text near the bookmark */
  preview: string | null;
  /** Paragraph ID containing the bookmark start */
  paragraphId: string;
  /** Character offset in the paragraph */
  offset: number;
}

/**
 * Insert a bookmark at the current selection
 */
export interface InsertBookmarkCommand {
  type: 'InsertBookmark';
  name: string;
}

/**
 * Delete a bookmark
 */
export interface DeleteBookmarkCommand {
  type: 'DeleteBookmark';
  name: string;
}

/**
 * Rename a bookmark
 */
export interface RenameBookmarkCommand {
  type: 'RenameBookmark';
  oldName: string;
  newName: string;
}

/**
 * Navigate to a bookmark
 */
export interface GoToBookmarkCommand {
  type: 'GoToBookmark';
  name: string;
}

/**
 * Open bookmark dialog
 */
export interface OpenBookmarkDialogCommand {
  type: 'OpenBookmarkDialog';
}

// =============================================================================
// Paragraph Formatting Commands
// =============================================================================

/**
 * Set paragraph alignment
 */
export interface SetParagraphAlignmentCommand {
  type: 'SetParagraphAlignment';
  alignment: TextAlignment;
}

/**
 * Open paragraph dialog
 */
export interface OpenParagraphDialogCommand {
  type: 'OpenParagraphDialog';
}

/**
 * Open symbol dialog
 */
export interface OpenSymbolDialogCommand {
  type: 'OpenSymbolDialog';
}

// =============================================================================
// RTL/LTR Text Direction Commands
// =============================================================================

/**
 * Text direction for paragraphs (RTL/LTR/Auto)
 */
export type ParagraphDirection = 'ltr' | 'rtl' | 'auto';

/**
 * Set paragraph text direction
 */
export interface SetParagraphDirectionCommand {
  type: 'SetParagraphDirection';
  direction: ParagraphDirection;
}

/**
 * Toggle paragraph direction (LTR <-> RTL)
 */
export interface ToggleParagraphDirectionCommand {
  type: 'ToggleParagraphDirection';
}

/**
 * Validate a bookmark name
 * Returns an error message if invalid, null if valid
 */
export function validateBookmarkName(name: string): string | null {
  if (!name || !name.trim()) {
    return 'Bookmark name cannot be empty';
  }

  if (name.length > 40) {
    return 'Bookmark name cannot exceed 40 characters';
  }

  const firstChar = name.charAt(0);
  if (!/^[a-zA-Z]$/.test(firstChar)) {
    return 'Bookmark name must start with a letter';
  }

  if (!/^[a-zA-Z][a-zA-Z0-9_]*$/.test(name)) {
    return 'Bookmark name can only contain letters, numbers, and underscores';
  }

  return null;
}

// =============================================================================
// Table Types
// =============================================================================

/**
 * Table cell render information
 */
export interface TableCellRenderInfo {
  cell_id: string;
  bounds: Rect;
  background: Color | null;
  selected: boolean;
}

/**
 * Table border render information
 */
export interface TableBorderRenderInfo {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  color: Color;
  width: number;
  style: string;
}

/**
 * Width type for table columns
 */
export type WidthType = 'Fixed' | 'Auto' | 'Percent';

/**
 * Table width specification
 */
export interface TableWidth {
  value: number;
  width_type: WidthType;
}

/**
 * Table border style
 */
export type TableBorderStyle =
  | 'None'
  | 'Single'
  | 'Double'
  | 'Dotted'
  | 'Dashed'
  | 'Thick';

/**
 * Cell border definition
 */
export interface TableBorder {
  style: TableBorderStyle;
  width: number;
  color: string;
}

/**
 * Cell borders (all four sides)
 */
export interface CellBorders {
  top?: TableBorder;
  bottom?: TableBorder;
  left?: TableBorder;
  right?: TableBorder;
}

/**
 * Cell padding
 */
export interface CellPadding {
  top: number;
  bottom: number;
  left: number;
  right: number;
}

/**
 * Vertical alignment in a cell
 */
export type CellVerticalAlign = 'Top' | 'Center' | 'Bottom';

/**
 * Insert table command
 */
export interface InsertTableCommand {
  type: 'InsertTable';
  rows: number;
  cols: number;
  width?: number;
}

/**
 * Delete table command
 */
export interface DeleteTableCommand {
  type: 'DeleteTable';
  tableId: string;
}

/**
 * Insert row command
 */
export interface InsertRowCommand {
  type: 'InsertRow';
  tableId: string;
  rowIndex: number;
  above: boolean;
}

/**
 * Delete row command
 */
export interface DeleteRowCommand {
  type: 'DeleteRow';
  tableId: string;
  rowId: string;
}

/**
 * Insert column command
 */
export interface InsertColumnCommand {
  type: 'InsertColumn';
  tableId: string;
  columnIndex: number;
  left: boolean;
}

/**
 * Delete column command
 */
export interface DeleteColumnCommand {
  type: 'DeleteColumn';
  tableId: string;
  columnIndex: number;
}

/**
 * Set cell borders command
 */
export interface SetCellBordersCommand {
  type: 'SetCellBorders';
  cellIds: string[];
  borders: CellBorders;
}

/**
 * Set cell shading command
 */
export interface SetCellShadingCommand {
  type: 'SetCellShading';
  cellIds: string[];
  color: string | null;
}

/**
 * Merge cells command
 */
export interface MergeCellsCommand {
  type: 'MergeCells';
  tableId: string;
  rowIndex: number;
  startCol: number;
  endCol: number;
}

/**
 * Split cell command
 */
export interface SplitCellCommand {
  type: 'SplitCell';
  tableId: string;
  rowIndex: number;
  colIndex: number;
  splitCount: number;
}

/**
 * Open table dialog command
 */
export interface OpenTableDialogCommand {
  type: 'OpenTableDialog';
}

/**
 * Table-related commands
 */
export type TableCommand =
  | InsertTableCommand
  | DeleteTableCommand
  | InsertRowCommand
  | DeleteRowCommand
  | InsertColumnCommand
  | DeleteColumnCommand
  | SetCellBordersCommand
  | SetCellShadingCommand
  | MergeCellsCommand
  | SplitCellCommand
  | OpenTableDialogCommand;

// =============================================================================
// List/Numbering Types
// =============================================================================

/**
 * Number format for list items
 */
export type NumberFormat =
  | 'Decimal'
  | 'DecimalZero'
  | 'LowerLetter'
  | 'UpperLetter'
  | 'LowerRoman'
  | 'UpperRoman'
  | 'Bullet'
  | 'None'
  | 'Ordinal'
  | 'CardinalText'
  | 'OrdinalText';

/**
 * Built-in list style IDs
 */
export const LIST_STYLE_IDS = {
  BULLET: 1,
  NUMBERED: 2,
  LEGAL: 3,
} as const;

/**
 * List style definition for the gallery
 */
export interface ListStyleDefinition {
  id: number;
  name: string;
  format: NumberFormat;
  preview: string[];
  isBullet: boolean;
}

/**
 * Built-in list styles
 */
export const BULLET_STYLES: ListStyleDefinition[] = [
  {
    id: 1,
    name: 'Filled Round',
    format: 'Bullet',
    preview: ['\u2022', '\u25E6', '\u25AA'],
    isBullet: true,
  },
  {
    id: 101,
    name: 'Hollow Round',
    format: 'Bullet',
    preview: ['\u25CB', '\u25CB', '\u25CB'],
    isBullet: true,
  },
  {
    id: 102,
    name: 'Square',
    format: 'Bullet',
    preview: ['\u25A0', '\u25A1', '\u25AA'],
    isBullet: true,
  },
  {
    id: 103,
    name: 'Check',
    format: 'Bullet',
    preview: ['\u2713', '\u2713', '\u2713'],
    isBullet: true,
  },
  {
    id: 104,
    name: 'Arrow',
    format: 'Bullet',
    preview: ['\u27A4', '\u27A4', '\u27A4'],
    isBullet: true,
  },
];

export const NUMBERED_STYLES: ListStyleDefinition[] = [
  {
    id: 2,
    name: 'Numbered (1, 2, 3)',
    format: 'Decimal',
    preview: ['1.', '2.', '3.'],
    isBullet: false,
  },
  {
    id: 201,
    name: 'Lowercase Letters',
    format: 'LowerLetter',
    preview: ['a.', 'b.', 'c.'],
    isBullet: false,
  },
  {
    id: 202,
    name: 'Uppercase Letters',
    format: 'UpperLetter',
    preview: ['A.', 'B.', 'C.'],
    isBullet: false,
  },
  {
    id: 203,
    name: 'Lowercase Roman',
    format: 'LowerRoman',
    preview: ['i.', 'ii.', 'iii.'],
    isBullet: false,
  },
  {
    id: 204,
    name: 'Uppercase Roman',
    format: 'UpperRoman',
    preview: ['I.', 'II.', 'III.'],
    isBullet: false,
  },
  {
    id: 3,
    name: 'Legal Style',
    format: 'Decimal',
    preview: ['1.', '1.1.', '1.1.1.'],
    isBullet: false,
  },
];

/**
 * Toggle bullet list command
 */
export interface ToggleBulletListCommand {
  type: 'ToggleBulletList';
}

/**
 * Toggle numbered list command
 */
export interface ToggleNumberedListCommand {
  type: 'ToggleNumberedList';
}

/**
 * Increase list indent command
 */
export interface IncreaseListIndentCommand {
  type: 'IncreaseListIndent';
}

/**
 * Decrease list indent command
 */
export interface DecreaseListIndentCommand {
  type: 'DecreaseListIndent';
}

/**
 * Change list type command
 */
export interface ChangeListTypeCommand {
  type: 'ChangeListType';
  numId: number;
}

/**
 * Remove from list command
 */
export interface RemoveFromListCommand {
  type: 'RemoveFromList';
}

/**
 * Restart numbering command
 */
export interface RestartNumberingCommand {
  type: 'RestartNumbering';
  startValue?: number;
}

/**
 * Set list level command
 */
export interface SetListLevelCommand {
  type: 'SetListLevel';
  level: number;
}

/**
 * List-related commands
 */
export type ListCommand =
  | ToggleBulletListCommand
  | ToggleNumberedListCommand
  | IncreaseListIndentCommand
  | DecreaseListIndentCommand
  | ChangeListTypeCommand
  | RemoveFromListCommand
  | RestartNumberingCommand
  | SetListLevelCommand;

// =============================================================================
// Autosave and Recovery Types
// =============================================================================

/**
 * Recovery file information from the backend
 */
export interface RecoveryFile {
  /** Unique identifier for this recovery file */
  id: string;
  /** Document ID from the original document */
  documentId: string;
  /** Timestamp when the recovery file was created (Unix timestamp in ms) */
  timestamp: number;
  /** Original file path (if known) */
  originalPath: string | null;
  /** Human-readable description of when the file was created */
  timeDescription: string;
  /** Size of the recovery file in bytes */
  fileSize: number;
}

/**
 * Autosave configuration
 */
export interface AutosaveConfig {
  /** Whether autosave is enabled */
  enabled: boolean;
  /** Interval between autosaves in seconds */
  intervalSecs: number;
  /** Maximum number of autosave versions to keep */
  maxVersions: number;
}

/**
 * Autosave status
 */
export interface AutosaveStatus {
  /** Whether autosave is enabled */
  enabled: boolean;
  /** Whether there are unsaved changes */
  hasUnsavedChanges: boolean;
  /** Whether a save is currently in progress */
  isSaving: boolean;
  /** Timestamp of last successful save (Unix timestamp in ms) */
  lastSaveTime: number | null;
  /** Error message from last save attempt (if any) */
  lastError: string | null;
  /** Time until next scheduled autosave (in seconds) */
  nextSaveInSecs: number | null;
}

/**
 * Save state for display
 */
export type SaveState = 'saved' | 'saving' | 'unsaved' | 'error';

/**
 * Get the save state from autosave status
 */
export function getSaveState(status: AutosaveStatus): SaveState {
  if (status.lastError) {
    return 'error';
  }
  if (status.isSaving) {
    return 'saving';
  }
  if (status.hasUnsavedChanges) {
    return 'unsaved';
  }
  return 'saved';
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  } else if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  } else {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
}

/**
 * Format timestamp for display
 */
export function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleString();
}

// =============================================================================
// View Mode Types
// =============================================================================

/**
 * View mode for the document editor
 */
export type ViewModeType = 'print-layout' | 'web-layout' | 'read-mode';

/**
 * View mode display information
 */
export interface ViewModeInfo {
  id: ViewModeType;
  label: string;
  description: string;
  icon: 'print' | 'web' | 'read';
  shortcut?: string;
}

// =============================================================================
// Document Statistics Types
// =============================================================================

/**
 * Document-wide statistics
 */
export interface DocumentStatsType {
  /** Total page count */
  pageCount: number;
  /** Total word count */
  wordCount: number;
  /** Total character count including spaces */
  characterCount: number;
  /** Total character count excluding spaces */
  characterCountNoSpaces: number;
  /** Total paragraph count */
  paragraphCount: number;
  /** Total line count (approximate) */
  lineCount: number;
  /** Estimated reading time in minutes */
  readingTimeMinutes: number;
}

/**
 * Selection statistics
 */
export interface SelectionStatsType {
  /** Selected word count */
  wordCount: number;
  /** Selected character count including spaces */
  characterCount: number;
  /** Selected character count excluding spaces */
  characterCountNoSpaces: number;
  /** Number of paragraphs in selection */
  paragraphCount: number;
}

// =============================================================================
// Status Bar Types
// =============================================================================

/**
 * Spell check status
 */
export type SpellCheckStatusType = 'idle' | 'checking' | 'has-errors' | 'no-errors' | 'disabled';

/**
 * Proofing language information
 */
export interface ProofingLanguageType {
  code: string;
  name: string;
  shortName: string;
}

/**
 * Text direction for BiDi support
 */
export type TextDirectionType = 'ltr' | 'rtl' | 'auto';

// =============================================================================
// Find/Replace Types
// =============================================================================

/**
 * Options for find operations
 */
export interface FindOptions {
  /** Case-sensitive search */
  caseSensitive: boolean;
  /** Match whole words only */
  wholeWord: boolean;
  /** Use regex pattern */
  useRegex: boolean;
  /** Wrap around to beginning when reaching end */
  wrapAround: boolean;
  /** Search backwards */
  searchBackwards?: boolean;
}

/**
 * Default find options
 */
export const DEFAULT_FIND_OPTIONS: FindOptions = {
  caseSensitive: false,
  wholeWord: false,
  useRegex: false,
  wrapAround: true,
  searchBackwards: false,
};

/**
 * Result of a find operation
 */
export interface FindResultData {
  /** The paragraph containing the match */
  nodeId: string;
  /** Start offset within the paragraph */
  startOffset: number;
  /** End offset within the paragraph */
  endOffset: number;
  /** The matched text */
  matchedText: string;
  /** Context around the match */
  context: string | null;
  /** Match index (1-based) */
  matchIndex: number;
}

/**
 * All results from a find all operation
 */
export interface FindAllResultsData {
  /** All matches found */
  matches: FindResultData[];
  /** Total count */
  totalCount: number;
  /** Current match index (0-based) */
  currentIndex: number | null;
}

/**
 * Find command
 */
export interface FindNextCommand {
  type: 'FindNext';
  pattern: string;
  options: FindOptions;
}

/**
 * Find previous command
 */
export interface FindPreviousCommand {
  type: 'FindPrevious';
  pattern: string;
  options: FindOptions;
}

/**
 * Replace command
 */
export interface ReplaceCurrentCommand {
  type: 'ReplaceCurrent';
  replacement: string;
}

/**
 * Replace all command
 */
export interface ReplaceAllMatchesCommand {
  type: 'ReplaceAll';
  pattern: string;
  replacement: string;
  options: FindOptions;
}

/**
 * Find/Replace related commands
 */
export type FindReplaceCommand =
  | FindNextCommand
  | FindPreviousCommand
  | ReplaceCurrentCommand
  | ReplaceAllMatchesCommand;

// =============================================================================
// Spellcheck Types
// =============================================================================

/**
 * Supported languages for spell checking
 */
export type SpellcheckLanguage = 'en-US' | 'en-GB' | 'es-ES' | 'fr-FR' | 'de-DE';

/**
 * Spelling error from the backend
 */
export interface SpellingErrorData {
  /** The paragraph containing the error */
  paraId: string;
  /** Start offset in the paragraph */
  startOffset: number;
  /** End offset in the paragraph */
  endOffset: number;
  /** The misspelled word */
  word: string;
  /** Suggested corrections */
  suggestions: string[];
}

/**
 * Spellcheck results from the backend
 */
export interface SpellcheckResultsData {
  /** All spelling errors found */
  errors: SpellingErrorData[];
  /** Current error index (0-based) */
  currentIndex: number | null;
  /** Total word count checked */
  wordsChecked: number;
}

/**
 * Ignore rules for spell checking
 */
export interface SpellcheckIgnoreRules {
  /** Ignore words in ALL CAPS */
  ignoreAllCaps: boolean;
  /** Ignore words containing numbers */
  ignoreWordsWithNumbers: boolean;
  /** Ignore URLs */
  ignoreUrls: boolean;
  /** Ignore email addresses */
  ignoreEmails: boolean;
  /** Ignore file paths */
  ignoreFilePaths: boolean;
  /** Minimum word length to check */
  minWordLength: number;
}

/**
 * Default ignore rules
 */
export const DEFAULT_IGNORE_RULES: SpellcheckIgnoreRules = {
  ignoreAllCaps: true,
  ignoreWordsWithNumbers: true,
  ignoreUrls: true,
  ignoreEmails: true,
  ignoreFilePaths: true,
  minWordLength: 2,
};

/**
 * Ignore once command
 */
export interface IgnoreSpellingOnceCommand {
  type: 'IgnoreSpellingOnce';
  paraId: string;
  startOffset: number;
  endOffset: number;
}

/**
 * Ignore all command
 */
export interface IgnoreSpellingAllCommand {
  type: 'IgnoreSpellingAll';
  word: string;
}

/**
 * Add to dictionary command
 */
export interface AddToDictionarySpellCommand {
  type: 'AddToDictionary';
  word: string;
  language: SpellcheckLanguage;
}

/**
 * Correct spelling command
 */
export interface CorrectSpellingTextCommand {
  type: 'CorrectSpelling';
  paraId: string;
  startOffset: number;
  endOffset: number;
  correction: string;
}

/**
 * Run spellcheck command
 */
export interface RunSpellcheckCommand {
  type: 'RunSpellcheck';
  language: SpellcheckLanguage;
}

/**
 * Spellcheck-related commands
 */
export type SpellcheckCommand =
  | IgnoreSpellingOnceCommand
  | IgnoreSpellingAllCommand
  | AddToDictionarySpellCommand
  | CorrectSpellingTextCommand
  | RunSpellcheckCommand;

// =============================================================================
// Squiggly Underline Types (for rendering)
// =============================================================================

/**
 * Squiggly style for error markers
 */
export type SquigglyStyleType = 'Spelling' | 'Grammar' | 'Style' | 'Custom';

/**
 * Squiggly render info
 */
export interface SquigglyRenderData {
  /** Position and size */
  bounds: Rect;
  /** Color of the squiggly */
  color: Color;
  /** The node ID this underline belongs to */
  nodeId: string;
  /** Start offset in the text */
  startOffset: number;
  /** End offset in the text */
  endOffset: number;
  /** Optional error message/tooltip */
  message: string | null;
}

/**
 * Find highlight render data
 */
export interface FindHighlightData {
  /** Highlight bounds */
  bounds: Rect;
  /** Highlight color */
  color: Color;
  /** Whether this is the current match */
  isCurrent: boolean;
}
