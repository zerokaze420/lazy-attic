<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<h2 class={cn('text-sm font-semibold tracking-normal text-foreground', className)} {...rest}>
  {@render children?.()}
</h2>
