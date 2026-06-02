declare namespace chrome {
  namespace runtime {
    const onInstalled: {
      addListener(callback: () => void | Promise<void>): void;
    };
  }

  namespace storage {
    namespace local {
      function get(key: string | string[]): Promise<Record<string, unknown>>;
      function set(items: Record<string, unknown>): Promise<void>;
    }
  }

  namespace declarativeNetRequest {
    interface MatchedRuleInfoDebug {
      request: {
        initiator?: string;
        url: string;
      };
    }

    const onRuleMatchedDebug: {
      addListener(callback: (info: MatchedRuleInfoDebug) => void): void;
    };
  }
}
