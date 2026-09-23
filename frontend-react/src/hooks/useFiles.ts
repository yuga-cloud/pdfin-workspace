import { useCallback, useState } from "react";
import { uploadFile } from "@/lib/api";

export function useFiles() {
  const [loading, setLoading] = useState(false);

  const upload = useCallback(async (file: File) => {
    setLoading(true);
    try {
      return await uploadFile(file);
    } finally {
      setLoading(false);
    }
  }, []);

  return { upload, loading };
}
