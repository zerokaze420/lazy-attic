<script lang="ts">
  import { Tabs as BitsTabs } from 'bits-ui';
  let { value = '', class: className = '', children, ...rest }: { value?: string; class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<BitsTabs.Root {value} class={className} {...rest}>
  {@render children?.()}
</BitsTabs.Root>
