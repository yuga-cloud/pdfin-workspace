import { useCallback, useState } from "react";
import { createConversion } from "@/lib/api";
import type { ConvertRequest } from "@/lib/api";

export function useConversions() {
  const [loading, setLoading] = useState(false);

  const create = useCallback(async (input: ConvertRequest) => {
    setLoading(true);
    try {
      return await createConversion(input);
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    loading,
    createConversion: create,
  };
}
