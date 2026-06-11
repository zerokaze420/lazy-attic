<script>
  export let label;
  export let value;
</script>

<div class="block">
  <span>{label}</span>
  <code>{value}</code>
</div>

<style>
  .block {
    display: grid;
    gap: 6px;
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
</style>
