<script lang="ts">
  import { cn } from '$lib/utils';

  type Variant = 'default' | 'secondary' | 'destructive' | 'ghost' | 'outline';
  type Size = 'default' | 'sm' | 'icon';

  let {
    variant = 'default',
    size = 'default',
    class: className = '',
    href,
    type = 'button',
    disabled = false,
    children,
    ...rest
  }: {
    variant?: Variant;
    size?: Size;
    class?: string;
    href?: string;
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    children?: () => unknown;
    [key: string]: unknown;
  } = $props();

  const base =
    'inline-flex max-w-full shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium outline-none transition-[background,border-color,color,opacity,transform] duration-200 ease-out hover:-translate-y-px active:translate-y-0 disabled:pointer-events-none disabled:opacity-50 focus-visible:ring-2 focus-visible:ring-zinc-400/40 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 overflow-hidden text-ellipsis';
  const variants: Record<Variant, string> = {
    default: 'bg-primary text-primary-foreground shadow-sm hover:bg-primary/90',
    secondary: 'border border-border bg-secondary text-secondary-foreground hover:bg-secondary/80',
    destructive: 'bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90',
    ghost: 'text-muted-foreground hover:bg-secondary hover:text-foreground',
    outline: 'border border-border bg-transparent text-foreground hover:bg-secondary'
  };
  const sizes: Record<Size, string> = {
    default: 'h-9 px-4 py-2',
    sm: 'h-8 px-3 text-xs',
    icon: 'h-9 w-9 px-0'
  };
</script>

{#if href}
  <a class={cn(base, variants[variant], sizes[size], className)} href={href} aria-disabled={disabled} {...rest}>
    {@render children?.()}
  </a>
{:else}
  <button class={cn(base, variants[variant], sizes[size], className)} {type} {disabled} {...rest}>
    {@render children?.()}
  </button>
{/if}
