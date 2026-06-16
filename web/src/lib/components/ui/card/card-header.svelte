<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<div class={cn('flex items-center justify-between gap-4 border-b border-border px-5 py-3', className)} {...rest}>
  {@render children?.()}
</div>
