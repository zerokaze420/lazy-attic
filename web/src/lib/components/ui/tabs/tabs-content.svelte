<script lang="ts">
  import { Tabs as BitsTabs } from 'bits-ui';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

<BitsTabs.Content class={className} {...rest}>
  {@render children?.()}
</BitsTabs.Content>
