import { cva } from "class-variance-authority";

export const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-semibold tracking-[-0.01em] transition-[opacity,transform,background-color,border-color,box-shadow,color] duration-200 ease-out focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:pointer-events-none disabled:opacity-45 [&_svg]:size-4 [&_svg]:shrink-0 active:scale-[0.985]",
  {
    variants: {
      variant: {
        default:
          "bg-primary text-primary-fg shadow-[0_8px_20px_-10px_color-mix(in_oklch,var(--color-primary)_70%,transparent)] hover:bg-primary/92 hover:shadow-[0_12px_26px_-12px_color-mix(in_oklch,var(--color-primary)_76%,transparent)]",
        secondary:
          "bg-surface-2 text-fg shadow-sm ring-1 ring-inset ring-fg/[0.035] hover:bg-border/70 hover:shadow-md",
        outline:
          "border border-border bg-surface text-fg shadow-sm hover:border-fg/15 hover:bg-surface-2 hover:shadow-md",
        ghost:
          "text-fg hover:bg-surface-2 hover:shadow-sm",
        danger:
          "bg-danger text-primary-fg shadow-[0_8px_20px_-10px_color-mix(in_oklch,var(--color-danger)_58%,transparent)] hover:bg-danger/92 hover:shadow-[0_12px_26px_-12px_color-mix(in_oklch,var(--color-danger)_64%,transparent)]",
      },

      size: {
        default: "h-11 min-h-11 px-4",
        sm: "h-9 min-h-9 px-3 text-xs",
        lg: "h-12 min-h-12 px-5",
        icon: "size-11 min-h-11 p-0",
      },
    },

    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);
