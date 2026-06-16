<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', variant = 'default', children, ...rest }: { class?: string; variant?: 'default' | 'secondary' | 'outline'; children?: () => unknown; [key: string]: unknown } = $props();
  const variants = {
    default: 'border-transparent bg-primary text-primary-foreground',
    secondary: 'border-transparent bg-secondary text-secondary-foreground',
    outline: 'text-foreground border border-border bg-transparent'
  };
</script>

<div class={cn('inline-flex items-center rounded-md px-2.5 py-0.5 text-xs font-medium', variants[variant], className)} {...rest}>
  {@render children?.()}
</div>
