<script lang="ts">
  import { Tabs as BitsTabs } from 'bits-ui';
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<BitsTabs.List class={cn('inline-flex items-center justify-center rounded-md bg-muted p-1 text-muted-foreground', className)} {...rest}>
  {@render children?.()}
</BitsTabs.List>
