<script lang="ts">
  import { Tabs as BitsTabs } from 'bits-ui';
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<BitsTabs.Trigger
  class={cn('inline-flex h-8 items-center justify-center rounded-md px-3 text-sm font-medium transition-all duration-200 hover:-translate-y-0.5 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm', className)}
  {...rest}
>
  {@render children?.()}
</BitsTabs.Trigger>
