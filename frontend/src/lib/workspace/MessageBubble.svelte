<script>
  /**
   * Bubble shape follows the canvas template supplied for this project: a
   * rounded card whose free edge folds into a small upward hook. The template
   * was authored for a 21px label (171×93 canvas); its geometry is scaled to
   * our 13.5px UI type. Canvas keeps the hook identical at every size and
   * browser; the text remains normal HTML above it.
   */
  let { outbound = false, children } = $props();

  /** Horizontal room reserved outside the HTML box for the hook. */
  const TAIL = 8;

  // Geometry measured from the template, scaled by 13.5 / 21.
  const R = 15.43;
  const TAIL_TOP = 10.61;
  const TAIL_END = 18.96;

  let host = $state(null);
  let canvas = $state(null);

  /** @param {CanvasRenderingContext2D} ctx @param {number} w @param {number} h */
  function bubblePath(ctx, w, h) {
    ctx.beginPath();
    ctx.moveTo(R, 0);
    ctx.lineTo(w - R, 0);

    // Top edge folds into the hook's upper line.
    ctx.bezierCurveTo(w - 7.07, 0, w - 1.607, 4.82, w, TAIL_TOP);

    // Hook upper edge: almost horizontal with a slight S.
    ctx.bezierCurveTo(w + 1.929, 11.7, w + 4.629, 10.93, w + 6.943, 10.7);

    // Micro-rounded tip (about 1px at template scale).
    ctx.bezierCurveTo(w + 7.426, 10.58, w + 7.747, 10.74, w + 7.714, 11.38);

    // Hook underside sweeps back into the edge.
    ctx.bezierCurveTo(w + 7.232, 13.98, w + 3.857, 17.74, w, TAIL_END);

    ctx.lineTo(w, h - R);
    ctx.bezierCurveTo(w, h - 6.49, w - 6.557, h, w - R, h);
    ctx.lineTo(R, h);
    ctx.bezierCurveTo(2.25, h, 0, h - 6.49, 0, h - R);
    ctx.lineTo(0, 14.79);
    ctx.bezierCurveTo(0, 7.39, 7.07, 0, R, 0);
    ctx.closePath();
  }

  function draw() {
    if (!host || !canvas) return;
    const w = host.clientWidth;
    const h = host.clientHeight;
    if (!w || !h) return;

    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const totalWidth = w + TAIL;
    canvas.width = Math.round(totalWidth * dpr);
    canvas.height = Math.round(h * dpr);
    canvas.style.width = `${totalWidth}px`;
    canvas.style.height = `${h}px`;
    canvas.style.left = outbound ? '0' : `-${TAIL}px`;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, totalWidth, h);

    const fill = getComputedStyle(host)
      .getPropertyValue(outbound ? '--bubble-out' : '--bubble-in')
      .trim();
    ctx.fillStyle = fill || '#f4f4f5';

    ctx.save();
    if (outbound) {
      bubblePath(ctx, w, h);
    } else {
      // Mirror the same path so inbound bubbles hook to the left.
      ctx.translate(totalWidth, 0);
      ctx.scale(-1, 1);
      bubblePath(ctx, w, h);
    }
    ctx.fill();
    ctx.restore();
  }

  $effect(() => {
    outbound;
    if (!host || !canvas) return;
    draw();
    const observer = new ResizeObserver(() => draw());
    observer.observe(host);
    return () => observer.disconnect();
  });
</script>

<div class="bubble" class:bubble--out={outbound} bind:this={host}>
  <canvas class="bubble__shape" bind:this={canvas} aria-hidden="true"></canvas>
  <div class="bubble__content">{@render children?.()}</div>
</div>

<style>
  .bubble {
    position: relative;
    display: inline-flex;
    max-width: min(78%, 560px);
    min-width: 0;
  }

  .bubble__shape {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 0;
    pointer-events: none;
  }

  .bubble__content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 12px 15px;
    min-width: 0;
  }
</style>
