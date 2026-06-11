<script>
  import { Clipboard } from '@lucide/svelte';
  let { label = '', value = '', oncopy } = $props();
</script>

<div class="info-row">
  <span>{label}</span>
  <code>{value}</code>
  <button class="btn btn-ghost btn-sm btn-icon" title={`复制${label}`} onclick={() => oncopy?.(value, label)}>
    <Clipboard size={13} />
  </button>
</div>
