<script>
  import { Clipboard } from '@lucide/svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';
  let { label = '', value = '', oncopy } = $props();
</script>

<div class="info-row">
  <span>{label}</span>
  <code>{value}</code>
  <Button variant="ghost" size="icon" class="size-8" title={`${t('copy')} ${label}`} onclick={() => oncopy?.(value, label)}>
    <Clipboard size={13} />
  </Button>
</div>
