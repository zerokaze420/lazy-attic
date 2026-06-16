<script>
  import { Clipboard } from '@lucide/svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';
  let { label = '', value = '', oncopy } = $props();
</script>

<div class="grid grid-cols-[100px_1fr_36px] items-center gap-2 border-b border-border py-2 last:border-b-0">
  <span class="text-xs font-medium text-muted-foreground">{label}</span>
  <code class="block rounded bg-muted px-2 py-1 text-xs break-all" style="font-family:var(--font-mono)">{value}</code>
  <Button variant="ghost" size="icon" class="size-8" title={`${t('copy')} ${label}`} onclick={() => oncopy?.(value, label)}>
    <Clipboard size={13} />
  </Button>
</div>