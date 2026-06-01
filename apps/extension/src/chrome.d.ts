declare namespace chrome {
  namespace runtime {
    const onInstalled: {
      addListener(callback: () => void | Promise<void>): void;
    };
  }

  namespace storage {
    namespace local {
      function get(key: string): Promise<Record<string, unknown>>;
      function set(items: Record<string, unknown>): Promise<void>;
    }
  }
}

