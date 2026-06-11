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

<style>
  .line {
    display: grid;
    grid-template-columns: 120px minmax(0, 1fr) 32px;
    gap: 8px;
    align-items: center;
  }

  span {
    color: #667487;
    font-size: 0.84rem;
  }

  code {
    min-width: 0;
    border-radius: 6px;
    padding: 9px 10px;
    background: #f3f6fa;
    color: #263548;
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
    font-size: 0.79rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }

  button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    min-height: 32px;
    border: 1px solid #175cd3;
    border-radius: 6px;
    padding: 0;
    background: #175cd3;
    color: #fff;
    cursor: pointer;
  }

  @media (max-width: 680px) {
    .line {
      grid-template-columns: 1fr 32px;
    }

    span {
      grid-column: 1 / -1;
    }
  }
</style>
