/**
 * Image Manager - Handles image loading, storage, and caching for the document editor
 */

// Supported image MIME types
const SUPPORTED_IMAGE_TYPES = [
  'image/png',
  'image/jpeg',
  'image/gif',
  'image/webp',
  'image/svg+xml',
  'image/bmp',
];

// Maximum image size in bytes (10MB)
const MAX_IMAGE_SIZE = 10 * 1024 * 1024;

/**
 * Image data with metadata
 */
export interface ImageInfo {
  resourceId: string;
  dataUrl: string;
  width: number;
  height: number;
  mimeType: string;
  size: number;
  filename?: string;
}

/**
 * Result of loading an image
 */
export type ImageLoadResult =
  | { success: true; image: ImageInfo }
  | { success: false; error: string };

/**
 * Cache for loaded images (data URLs and dimensions)
 */
const imageCache = new Map<string, ImageInfo>();

/**
 * Check if a MIME type is a supported image format
 */
export function isSupportedImageType(mimeType: string): boolean {
  return SUPPORTED_IMAGE_TYPES.includes(mimeType);
}

/**
 * Detect MIME type from file extension
 */
export function getMimeTypeFromExtension(filename: string): string | null {
  const ext = filename.toLowerCase().split('.').pop();
  switch (ext) {
    case 'png':
      return 'image/png';
    case 'jpg':
    case 'jpeg':
      return 'image/jpeg';
    case 'gif':
      return 'image/gif';
    case 'webp':
      return 'image/webp';
    case 'svg':
      return 'image/svg+xml';
    case 'bmp':
      return 'image/bmp';
    default:
      return null;
  }
}

/**
 * Read a file as array buffer
 */
function readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as ArrayBuffer);
    reader.onerror = () => reject(new Error('Failed to read file'));
    reader.readAsArrayBuffer(file);
  });
}

/**
 * Read a file as data URL
 */
function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error('Failed to read file'));
    reader.readAsDataURL(file);
  });
}

/**
 * Get image dimensions from a data URL
 */
function getImageDimensions(dataUrl: string): Promise<{ width: number; height: number }> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      resolve({ width: img.naturalWidth, height: img.naturalHeight });
    };
    img.onerror = () => reject(new Error('Failed to load image'));
    img.src = dataUrl;
  });
}

/**
 * Generate a unique resource ID
 */
function generateResourceId(): string {
  return `img-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
}

/**
 * Load an image from a File object
 */
export async function loadImageFromFile(file: File): Promise<ImageLoadResult> {
  // Check file size
  if (file.size > MAX_IMAGE_SIZE) {
    return {
      success: false,
      error: `Image too large. Maximum size is ${MAX_IMAGE_SIZE / 1024 / 1024}MB.`,
    };
  }

  // Check MIME type
  let mimeType = file.type;
  if (!mimeType || !isSupportedImageType(mimeType)) {
    // Try to detect from extension
    const detectedType = getMimeTypeFromExtension(file.name);
    if (detectedType && isSupportedImageType(detectedType)) {
      mimeType = detectedType;
    } else {
      return {
        success: false,
        error: `Unsupported image format. Supported formats: PNG, JPEG, GIF, WebP, SVG, BMP.`,
      };
    }
  }

  try {
    // Read file as data URL
    const dataUrl = await readFileAsDataUrl(file);

    // Get dimensions
    const { width, height } = await getImageDimensions(dataUrl);

    // Generate resource ID
    const resourceId = generateResourceId();

    const imageInfo: ImageInfo = {
      resourceId,
      dataUrl,
      width,
      height,
      mimeType,
      size: file.size,
      filename: file.name,
    };

    // Cache the image
    imageCache.set(resourceId, imageInfo);

    return { success: true, image: imageInfo };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to load image',
    };
  }
}

/**
 * Load an image from a data URL (e.g., from clipboard)
 */
export async function loadImageFromDataUrl(
  dataUrl: string,
  filename?: string
): Promise<ImageLoadResult> {
  try {
    // Validate data URL format
    const match = dataUrl.match(/^data:([^;,]+)/);
    if (!match) {
      return { success: false, error: 'Invalid data URL format' };
    }

    const mimeType = match[1];
    if (!isSupportedImageType(mimeType)) {
      return { success: false, error: 'Unsupported image format' };
    }

    // Get dimensions
    const { width, height } = await getImageDimensions(dataUrl);

    // Calculate approximate size from base64
    const base64Data = dataUrl.split(',')[1] || '';
    const size = Math.ceil((base64Data.length * 3) / 4);

    if (size > MAX_IMAGE_SIZE) {
      return {
        success: false,
        error: `Image too large. Maximum size is ${MAX_IMAGE_SIZE / 1024 / 1024}MB.`,
      };
    }

    // Generate resource ID
    const resourceId = generateResourceId();

    const imageInfo: ImageInfo = {
      resourceId,
      dataUrl,
      width,
      height,
      mimeType,
      size,
      filename,
    };

    // Cache the image
    imageCache.set(resourceId, imageInfo);

    return { success: true, image: imageInfo };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to load image',
    };
  }
}

/**
 * Load an image from clipboard data
 */
export async function loadImageFromClipboard(
  clipboardData: DataTransfer
): Promise<ImageLoadResult | null> {
  // Check for image files in clipboard
  const files = clipboardData.files;
  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    if (file.type.startsWith('image/')) {
      return loadImageFromFile(file);
    }
  }

  // Check for image data in clipboard items
  const items = clipboardData.items;
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile();
      if (file) {
        return loadImageFromFile(file);
      }
    }
  }

  return null; // No image found in clipboard
}

/**
 * Get cached image by resource ID
 */
export function getCachedImage(resourceId: string): ImageInfo | undefined {
  return imageCache.get(resourceId);
}

/**
 * Store image data URL in cache (for images loaded from backend)
 */
export function cacheImageData(
  resourceId: string,
  dataUrl: string,
  width: number,
  height: number
): void {
  // Extract MIME type from data URL
  const match = dataUrl.match(/^data:([^;,]+)/);
  const mimeType = match ? match[1] : 'application/octet-stream';

  // Calculate size from base64
  const base64Data = dataUrl.split(',')[1] || '';
  const size = Math.ceil((base64Data.length * 3) / 4);

  imageCache.set(resourceId, {
    resourceId,
    dataUrl,
    width,
    height,
    mimeType,
    size,
  });
}

/**
 * Remove image from cache
 */
export function removeCachedImage(resourceId: string): boolean {
  return imageCache.delete(resourceId);
}

/**
 * Clear all cached images
 */
export function clearImageCache(): void {
  imageCache.clear();
}

/**
 * Get cache statistics
 */
export function getImageCacheStats(): { count: number; totalSize: number } {
  let totalSize = 0;
  for (const img of imageCache.values()) {
    totalSize += img.size;
  }
  return { count: imageCache.size, totalSize };
}

/**
 * Create an HTMLImageElement from a cached image (for canvas rendering)
 */
export function createImageElement(resourceId: string): Promise<HTMLImageElement | null> {
  const cached = imageCache.get(resourceId);
  if (!cached) {
    return Promise.resolve(null);
  }

  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => resolve(null);
    img.src = cached.dataUrl;
  });
}

/**
 * Check if an image with the given resource ID is cached
 */
export function isImageCached(resourceId: string): boolean {
  return imageCache.has(resourceId);
}
