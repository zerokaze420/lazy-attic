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
    'inline-flex min-w-max shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-2xl text-[0.95rem] font-medium outline-none transition-all duration-300 ease-out hover:-translate-y-px active:translate-y-0 disabled:pointer-events-none disabled:opacity-50 focus-visible:ring-2 focus-visible:ring-zinc-400/40 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0';
  const variants: Record<Variant, string> = {
    default: 'border border-white/10 bg-white text-zinc-950 shadow-sm shadow-black/20 hover:bg-white/95 hover:shadow-md',
    secondary: 'border border-white/10 bg-white/5 text-foreground backdrop-blur-md hover:border-white/15 hover:bg-white/8',
    destructive: 'border border-rose-400/20 bg-rose-500 text-white shadow-sm shadow-rose-950/20 hover:bg-rose-400',
    ghost: 'text-muted-foreground hover:bg-white/8 hover:text-foreground',
    outline: 'border border-white/10 bg-transparent text-foreground hover:border-white/15 hover:bg-white/6'
  };
  const sizes: Record<Size, string> = {
    default: 'h-11 px-5 py-2.5',
    sm: 'h-10 px-4 text-[0.9rem]',
    icon: 'h-11 w-11 px-0'
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
