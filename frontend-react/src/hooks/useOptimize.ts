import { useState } from "react";
import { optimizeApi } from "../lib/api";

export function useOptimize() {
  const [loading, setLoading] = useState(false);

  async function optimize(input: Parameters<typeof optimizeApi.optimize>[0]) {
    setLoading(true);
    try {
      return await optimizeApi.optimize(input);
    } finally {
      setLoading(false);
    }
  }

  return {
    loading,
    optimize,
  };
}
