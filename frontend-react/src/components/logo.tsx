import { Link } from "@tanstack/react-router";
import { cn } from "@/lib/utils";

export function Logo({ className, to = "/" }: { className?: string; to?: string }) {
  return (
    <Link
      to={to}
      className={cn(
        "group inline-flex items-center gap-2.5 font-semibold tracking-tight text-fg no-underline",
        className,
      )}
      aria-label="pdfin beranda"
    >
      <span
        className="brand-mark grid size-9 place-items-center rounded-xl bg-fg shadow-sm"
        aria-hidden
      >
        <svg viewBox="0 0 32 32" className="brand-mark-icon size-5" fill="none">
          <path fill="#FFFFFF" d="M8 6h8v8h8v12H8z" />
          <path fill="#DF5148" d="M16 6l8 8h-8z" />
        </svg>
      </span>
      <span className="text-[1.15rem] font-bold tracking-[-0.045em]">
        pdf<span className="text-primary">in</span>
      </span>
    </Link>
  );
}
