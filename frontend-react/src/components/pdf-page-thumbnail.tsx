import { useEffect, useRef, useState } from "react";
import type { PdfThumbnail, PdfThumbnailCache } from "@/lib/pdf/thumbnail-cache";
import { cn } from "@/lib/utils";

const ROOT_MARGIN = "320px 0px";

export function PdfPageThumbnail({
  cache,
  pageNumber,
  alt,
  className,
}: {
  cache: PdfThumbnailCache | null;
  pageNumber: number;
  alt: string;
  className?: string;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const [thumbnail, setThumbnail] = useState<PdfThumbnail | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    const host = hostRef.current;
    if (!host || !cache) return;

    let cancelled = false;
    let controller: AbortController | null = null;

    const load = async () => {
      controller = new AbortController();
      try {
        const next = await cache.render(pageNumber, controller.signal);
        if (!cancelled) setThumbnail(next);
      } catch (error) {
        if (!cancelled && !(error instanceof Error && error.name === "AbortError")) {
          setFailed(true);
        }
      }
    };

    if (typeof IntersectionObserver === "undefined") {
      void load();
      return () => {
        cancelled = true;
        controller?.abort();
      };
    }

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          observer.disconnect();
          void load();
        }
      },
      { rootMargin: ROOT_MARGIN },
    );

    observer.observe(host);

    return () => {
      cancelled = true;
      observer.disconnect();
      controller?.abort();
    };
  }, [cache, pageNumber]);

  return (
    <div ref={hostRef} className={cn("relative aspect-3/4 overflow-hidden rounded-md bg-surface-2", className)}>
      {thumbnail ? (
        <img
          src={thumbnail.url}
          alt={alt}
          width={thumbnail.width}
          height={thumbnail.height}
          loading="lazy"
          decoding="async"
          className="h-full w-full object-contain"
        />
      ) : failed ? (
        <div className="grid h-full place-items-center px-3 text-center text-xs text-muted">
          Pratinjau tidak tersedia
        </div>
      ) : (
        <div className="grid h-full place-items-center text-xs text-muted">
          Memuat…
        </div>
      )}
    </div>
  );
}
