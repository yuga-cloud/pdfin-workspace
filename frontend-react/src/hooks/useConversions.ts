import { useState } from "react";
import { conversionsApi } from "../lib/api";

export function useConversions() {
  const [loading, setLoading] = useState(false);

  async function createConversion(input: Parameters<typeof conversionsApi.create>[0]) {
    setLoading(true);
    try {
      return await conversionsApi.create(input);
    } finally {
      setLoading(false);
    }
  }

  return {
    loading,
    createConversion,
  };
}
