import { useEffect, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ArrowDown, ArrowUp, Trash2 } from "lucide-react";
import { PdfPageThumbnail } from "@/features/workspace/pdf-page-thumbnail";
import type { PdfThumbnailCache } from "@/lib/pdf/thumbnail-cache";

function columnsForWidth(width: number): number {
  if (width >= 820) return 4;
  if (width >= 560) return 3;
  return 2;
}

export function VirtualizedPdfPageGrid({
  order,
  cache,
  busy,
  onMove,
  onRemove,
}: {
  order: number[];
  cache: PdfThumbnailCache | null;
  busy: boolean;
  onMove: (index: number, direction: -1 | 1) => void;
  onRemove: (index: number) => void;
}) {
  const viewportRef = useRef<HTMLDivElement>(null);
  const [columns, setColumns] = useState(4);

  useEffect(() => {
    const element = viewportRef.current;
    if (!element || typeof ResizeObserver === "undefined") return;

    const update = () => setColumns(columnsForWidth(element.clientWidth));
    update();

    const observer = new ResizeObserver(update);
    observer.observe(element);

    return () => observer.disconnect();
  }, []);

  const rowCount = Math.ceil(order.length / columns);
  const rowVirtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => viewportRef.current,
    estimateSize: () => (columns === 4 ? 245 : columns === 3 ? 265 : 285),
    overscan: 3,
    getItemKey: (index) => index,
  });

  return (
    <div
      ref={viewportRef}
      className="workspace-page-viewport mt-4 max-h-[min(70vh,760px)] overflow-auto overscroll-contain rounded-xl pr-1"
    >
      <div
        className="relative w-full"
        style={{ height: rowVirtualizer.getTotalSize() }}
      >
        {rowVirtualizer.getVirtualItems().map((virtualRow) => {
          const startIndex = virtualRow.index * columns;

          return (
            <div
              key={virtualRow.key}
              ref={rowVirtualizer.measureElement}
              data-index={virtualRow.index}
              className="absolute inset-x-0 grid gap-3 pb-3"
              style={{
                transform: `translateY(${virtualRow.start}px)`,
                gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
              }}
            >
              {Array.from({ length: columns }, (_, columnIndex) => {
                const index = startIndex + columnIndex;
                const pageIndex = order[index];

                if (pageIndex === undefined) return null;

                return (
                  <div
                    key={`${pageIndex}-${index}`}
                    className="workspace-page-card rounded-lg bg-bg p-2"
                  >
                    <PdfPageThumbnail
                      cache={cache}
                      pageNumber={pageIndex + 1}
                      alt={`Halaman ${pageIndex + 1}`}
                    />
                    <div className="mt-2 flex items-center justify-between gap-1">
                      <span className="text-xs tabular-nums text-muted">
                        Halaman {pageIndex + 1}
                      </span>
                      <div className="flex">
                        <button
                          type="button"
                          className="grid size-8 place-items-center rounded-md hover:bg-surface-2 disabled:opacity-40"
                          aria-label="Naikkan halaman"
                          onClick={() => onMove(index, -1)}
                          disabled={index === 0 || busy}
                        >
                          <ArrowUp className="size-3.5" />
                        </button>
                        <button
                          type="button"
                          className="grid size-8 place-items-center rounded-md hover:bg-surface-2 disabled:opacity-40"
                          aria-label="Turunkan halaman"
                          onClick={() => onMove(index, 1)}
                          disabled={index === order.length - 1 || busy}
                        >
                          <ArrowDown className="size-3.5" />
                        </button>
                        <button
                          type="button"
                          className="grid size-8 place-items-center rounded-md text-danger hover:bg-surface-2 disabled:opacity-40"
                          aria-label="Hapus halaman"
                          onClick={() => onRemove(index)}
                          disabled={order.length <= 1 || busy}
                        >
                          <Trash2 className="size-3.5" />
                        </button>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          );
        })}
      </div>
    </div>
  );
}
