/**
 * Mail Merge Components
 *
 * Components for mail merge data source management:
 * - DataSourcePicker: Select and load CSV/JSON data sources
 * - DataPreview: Preview data in a table format
 * - ColumnMapping: Map columns to merge fields
 */

export { DataSourcePicker } from './DataSourcePicker';
export type { DataSourceInfo } from './DataSourcePicker';

export { DataPreview } from './DataPreview';
export type { ColumnDef, ValueDto, RecordDto, DataPreviewDto } from './DataPreview';

export { ColumnMapping } from './ColumnMapping';
export type { FieldMapping } from './ColumnMapping';
