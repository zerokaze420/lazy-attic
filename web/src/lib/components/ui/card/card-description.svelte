<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<p class={cn('mt-0.5 text-xs text-muted-foreground', className)} {...rest}>
  {@render children?.()}
</p>
