<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', variant = 'default', children, ...rest }: { class?: string; variant?: 'default' | 'secondary' | 'outline'; children?: () => unknown; [key: string]: unknown } = $props();
  const variants = {
    default: 'border-transparent bg-primary text-primary-foreground',
    secondary: 'border-transparent bg-secondary text-secondary-foreground',
    outline: 'text-foreground border border-border bg-transparent'
  };
</script>

<div class={cn('inline-flex items-center rounded-full px-3 py-1 text-xs font-semibold transition-all duration-300 hover:-translate-y-0.5 hover:scale-[1.02]', variants[variant], className)} {...rest}>
  {@render children?.()}
</div>
