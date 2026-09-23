import { useEffect, useRef, useState } from "react";

type JobState = "queued" | "processing" | "completed" | "failed" | string;

interface PollResult<T> {
  status: JobState;
  data?: T;
  error?: string;
}

interface UseJobPollingOptions<T> {
  enabled?: boolean;
  poll: () => Promise<PollResult<T>>;
  onCompleted?: (result: PollResult<T>) => void;
  onFailed?: (result: PollResult<T>) => void;
}

const delays = [1000, 2000, 4000, 8000, 15000];

export function useJobPolling<T>({
  enabled = true,
  poll,
  onCompleted,
  onFailed,
}: UseJobPollingOptions<T>) {
  const [result, setResult] = useState<PollResult<T> | null>(null);
  const [running, setRunning] = useState(false);
  const cancelled = useRef(false);

  useEffect(() => {
    cancelled.current = false;

    if (!enabled) {
      return;
    }

    let attempt = 0;
    setRunning(true);

    const execute = async () => {
      while (!cancelled.current) {
        const response = await poll();

        if (cancelled.current) {
          return;
        }

        setResult(response);

        if (response.status === "completed") {
          setRunning(false);
          onCompleted?.(response);
          return;
        }

        if (response.status === "failed") {
          setRunning(false);
          onFailed?.(response);
          return;
        }

        const delay = delays[Math.min(attempt, delays.length - 1)];
        attempt += 1;

        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    };

    void execute();

    return () => {
      cancelled.current = true;
    };
  }, [enabled, onCompleted, onFailed, poll]);

  return {
    result,
    running,
  };
}
