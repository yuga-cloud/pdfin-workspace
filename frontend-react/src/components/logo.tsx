import { Link } from "@tanstack/react-router";
import { cn } from "@/lib/utils";

export function Logo({ className, to = "/" }: { className?: string; to?: string }) {
  return (
    <Link
      to={to}
      className={cn(
        "inline-flex items-center gap-2 font-semibold tracking-tight text-fg no-underline",
        className,
      )}
    >
      <span className="flex size-8 items-center justify-center rounded-md bg-fg" aria-hidden>
        <svg viewBox="0 0 32 32" className="size-5" fill="none">
          <path fill="#FFFFFF" d="M8 6h8v8h8v12H8z" />
          <path fill="#DF5148" d="M16 6l8 8h-8z" />
        </svg>
      </span>
      <span className="text-lg">
        pdf<span className="text-primary">in</span>
      </span>
    </Link>
  );
}
