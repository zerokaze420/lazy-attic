<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<label class={cn('text-sm font-medium leading-none text-foreground', className)} {...rest}>
  {@render children?.()}
</label>
