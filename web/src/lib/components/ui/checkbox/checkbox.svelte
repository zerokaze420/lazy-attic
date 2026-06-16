<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', checked = false, ...rest }: { class?: string; checked?: boolean; [key: string]: unknown } = $props();
</script>

<input
  type="checkbox"
  checked={checked}
  class={cn('size-4 rounded border border-input bg-background text-primary outline-none transition-all duration-200 focus-visible:ring-2 focus-visible:ring-ring/40', className)}
  {...rest}
/>
