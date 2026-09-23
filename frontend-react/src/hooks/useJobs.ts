import { useCallback, useState } from "react";
import { getJob } from "@/lib/api";

export function useJobs() {
  const [loading, setLoading] = useState(false);

  const fetchJob = useCallback(async (id: string) => {
    setLoading(true);
    try {
      return await getJob(id);
    } finally {
      setLoading(false);
    }
  }, []);

  return { fetchJob, loading };
}
