import { cn } from "@/lib/utils";

export function Progress({
  value,
  className,
}: {
  value: number;
  className?: string;
}) {
  const pct = Math.max(0, Math.min(100, value));
  return (
    <div
      className={cn(
        "pdfin-progress h-2 w-full overflow-hidden rounded-full border border-fg/[0.05] bg-surface-2 shadow-[inset_0_1px_2px_rgba(32,33,36,0.06)]",
        className,
      )}
      role="progressbar"
      aria-valuenow={Math.round(pct)}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <div
        className="pdfin-progress-bar h-full min-w-0 rounded-full bg-primary shadow-[0_0_14px_rgba(223,81,72,0.22)] transition-[width] duration-200 ease-out"
        style={{ width: `${pct}%` }}
      />
    </div>
  );
}
