<script>
  let { series = [], height = 240 } = $props();

  let hover = $state(null);

  const MARGIN = { top: 20, right: 16, bottom: 32, left: 52 };
  const CHART_W = 800;
  let CHART_H = $derived(height);
  let INNER_W = $derived(CHART_W - MARGIN.left - MARGIN.right);
  let INNER_H = $derived(CHART_H - MARGIN.top - MARGIN.bottom);

  let allY = $derived(series.flatMap(s => s.points.map(p => p.y)));
  let yMax = $derived(Math.max(...allY, 1));

  let gridCount = $derived(4);
  let yStep = $derived(yMax / gridCount);
  let gridLines = $derived(
    Array.from({ length: gridCount + 1 }, (_, i) => i * yStep)
  );

  function fmtY(v) {
    if (v >= 1_000_000_000) return (v / 1_000_000_000).toFixed(1) + 'G';
    if (v >= 1_000_000) return (v / 1_000_000).toFixed(1) + 'M';
    if (v >= 1_000) return (v / 1_000).toFixed(1) + 'k';
    return v.toFixed(0);
  }

  function px(i, total) {
    return MARGIN.left + (i / Math.max(total - 1, 1)) * INNER_W;
  }

  function py(v) {
    return MARGIN.top + INNER_H - (v / yMax) * INNER_H;
  }

  function buildPath(s) {
    if (s.points.length === 0) return '';
    return s.points
      .map((p, i) => {
        const x = px(i, s.points.length);
        const y = py(p.y);
        return i === 0 ? `M${x},${y}` : `L${x},${y}`;
      })
      .join(' ');
  }

  function handleMouseMove(e) {
    const svg = e.currentTarget;
    const rect = svg.getBoundingClientRect();
    const scaleX = CHART_W / rect.width;
    const mouseX = (e.clientX - rect.left) * scaleX;

    if (mouseX < MARGIN.left || mouseX > MARGIN.left + INNER_W) {
      hover = null;
      return;
    }

    // Find closest point across all series
    let bestSeriesIdx = 0;
    let bestPointIdx = 0;
    let bestDist = Infinity;

    for (let si = 0; si < series.length; si++) {
      const pts = series[si].points;
      for (let pi = 0; pi < pts.length; pi++) {
        const dist = Math.abs(px(pi, pts.length) - mouseX);
        if (dist < bestDist) {
          bestDist = dist;
          bestSeriesIdx = si;
          bestPointIdx = pi;
        }
      }
    }

    const pts = series[bestSeriesIdx].points;
    hover = {
      seriesIdx: bestSeriesIdx,
      pointIdx: bestPointIdx,
      x: px(bestPointIdx, pts.length),
      y: py(pts[bestPointIdx].y)
    };
  }

  function handleMouseLeave() {
    hover = null;
  }
</script>

<div class="line-chart-wrapper">
  <svg
    viewBox="0 0 {CHART_W} {CHART_H}"
    preserveAspectRatio="xMidYMid meet"
    role="img"
    aria-label="Line chart"
    onmousemove={handleMouseMove}
    onmouseleave={handleMouseLeave}
  >
    <!-- Grid lines -->
    {#each gridLines as val}
      <line
        x1={MARGIN.left}
        y1={py(val)}
        x2={MARGIN.left + INNER_W}
        y2={py(val)}
        stroke="hsl(var(--border))"
        stroke-width="1"
      />
      <text
        x={MARGIN.left - 8}
        y={py(val) + 4}
        text-anchor="end"
        fill="hsl(var(--muted-foreground))"
        font-size="11"
      >{fmtY(val)}</text>
    {/each}

    <!-- X axis labels (show every ~7th) -->
    {#if series.length > 0}
      {@const pts = series[0].points}
      {@const step = Math.max(1, Math.floor(pts.length / 7))}
      {#each pts as p, i}
        {#if i % step === 0 || i === pts.length - 1}
          <text
            x={px(i, pts.length)}
            y={CHART_H - 6}
            text-anchor="middle"
            fill="hsl(var(--muted-foreground))"
            font-size="10"
          >{p.x.slice(5)}</text>
        {/if}
      {/each}
    {/if}

    <!-- Lines -->
    {#each series as s, si}
      <path
        d={buildPath(s)}
        fill="none"
        stroke={s.color}
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    {/each}

    <!-- Hover indicator -->
    {#if hover}
      <line
        x1={hover.x}
        y1={MARGIN.top}
        x2={hover.x}
        y2={MARGIN.top + INNER_H}
        stroke="hsl(var(--muted-foreground))"
        stroke-width="1"
        stroke-dasharray="4,4"
      />
      {#each series as s, si}
        {@const pts = s.points}
        {#if hover.pointIdx < pts.length}
          <circle
            cx={px(hover.pointIdx, pts.length)}
            cy={py(pts[hover.pointIdx].y)}
            r="4"
            fill={s.color}
            stroke="hsl(var(--background))"
            stroke-width="2"
          />
        {/if}
      {/each}
    {/if}
  </svg>

  <!-- Tooltip -->
  {#if hover}
    {@const s = series[hover.seriesIdx]}
    {@const p = s.points[hover.pointIdx]}
    <div class="chart-tooltip" style="left:{(hover.x / CHART_W) * 100}%">
      <div class="chart-tooltip-date">{p.x}</div>
      {#each series as line}
        {@const pt = line.points[hover.pointIdx]}
        {#if pt}
          <div class="chart-tooltip-series">
            <span class="chart-tooltip-dot" style="background:{line.color}"></span>
            <span class="chart-tooltip-label">{line.label}</span>
            <span class="chart-tooltip-value">{fmtY(pt.y)}</span>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .line-chart-wrapper {
    position: relative;
    width: 100%;
  }
  .line-chart-wrapper svg {
    width: 100%;
    height: auto;
    display: block;
  }
  .chart-tooltip {
    position: absolute;
    top: 0;
    transform: translateX(-50%);
    pointer-events: none;
    background: hsl(var(--popover));
    border: 1px solid hsl(var(--border));
    border-radius: var(--radius);
    padding: 8px 12px;
    font-size: 0.75rem;
    box-shadow: 0 4px 12px rgba(0,0,0,0.08);
    z-index: 10;
    min-width: 120px;
  }
  .chart-tooltip-date {
    font-weight: 600;
    color: hsl(var(--foreground));
    margin-bottom: 4px;
    font-size: 0.78rem;
  }
  .chart-tooltip-series {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
  }
  .chart-tooltip-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .chart-tooltip-label {
    color: hsl(var(--muted-foreground));
    flex: 1;
  }
  .chart-tooltip-value {
    font-weight: 600;
    color: hsl(var(--foreground));
    font-variant-numeric: tabular-nums;
  }
</style>
