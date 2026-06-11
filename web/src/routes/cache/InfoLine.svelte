<script>
  import { Clipboard } from '@lucide/svelte';

  export let label;
  export let value;
  export let oncopy;
</script>

<div class="line">
  <span>{label}</span>
  <code>{value}</code>
  <button type="button" title={`复制${label}`} on:click={() => oncopy(value, label)}>
    <Clipboard size={15} />
  </button>
</div>
