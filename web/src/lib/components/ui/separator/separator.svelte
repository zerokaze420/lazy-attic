<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', orientation = 'horizontal', children, ...rest }: { class?: string; orientation?: 'horizontal' | 'vertical'; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<div
  data-orientation={orientation}
  class={cn(orientation === 'horizontal' ? 'h-px w-full bg-border' : 'h-full w-px bg-border', className)}
  {...rest}
>
  {@render children?.()}
</div>
