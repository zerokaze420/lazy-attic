<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', ...rest }: { class?: string; [key: string]: unknown } = $props();
</script>

<input
   class={cn('flex h-9 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground outline-none transition-[background,border-color,color,opacity] duration-200 placeholder:text-muted-foreground focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20 disabled:cursor-not-allowed disabled:opacity-50', className)}
  {...rest}
/>
