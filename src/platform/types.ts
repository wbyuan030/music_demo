interface Router {
  /**
   * Navigate to a specific path with optional parameters
   */
  navigate: (path: string, params?: Record<string, any>) => Promise<void>;

  /**
   * Navigate back to the previous page
   */
  back: () => void;

  /**
   * Check if navigation back is possible
   */
  canGoBack: () => boolean;

  /**
   * Get the current path
   */
  getPath: () => string;
}

interface Storage<T = any> {
  /**
   * Add an item to a specific sheet
   */
  add: (item: T, sheet: string) => Promise<void>;

  /**
   * Remove an item from a specific sheet
   */
  delete: (id: string, sheet: string) => Promise<void>;

  /**
   * List all items in a specific sheet
   */
  list: (sheet: string) => Promise<T[]>;
}

