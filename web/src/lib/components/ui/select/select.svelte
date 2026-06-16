<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<select class={cn('flex h-9 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground outline-none transition-[background,border-color,color,opacity] duration-200 focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20', className)} {...rest}>
  {@render children?.()}
</select>
