import { ref, reactive, computed, onMounted, onUnmounted, nextTick } from 'vue';

export function useCameraRig() {
  const rigEl = ref<HTMLElement | null>(null);
  const gutterPx = ref(16);
  const isMobile = ref(false);

  const rig = reactive({
    spreadW: 0,
    spreadH: 0,
    panX: 0,
    panY: 0,
    minPanX: 0,
    maxPanX: 0,
    minPanY: 0,
    maxPanY: 0,
    dragging: false,
    pid: null as number | null,
    downX: 0,
    downY: 0,
    startPanX: 0,
    startPanY: 0,
    moved: false,
    lastDragT: 0,
  });

  const rigStyle = computed(() => ({
    '--spreadW': rig.spreadW + 'px',
    '--spreadH': rig.spreadH + 'px',
    '--panX': rig.panX + 'px',
    '--panY': rig.panY + 'px',
    '--gutter': gutterPx.value + 'px'
  }));

  const measureRig = () => {
    // 关键修复：如果没有父元素（还没渲染出来），直接返回，防止报错
    const vp = rigEl.value?.parentElement;
    if (!vp) return;

    const vh = vp.clientHeight;
    const vw = vp.clientWidth;
    const m = window.matchMedia('(max-width: 768px)').matches;
    isMobile.value = (vw < 768) || m;

    const spreadMaxW = vw * 1.0;
    const spreadMaxH = vh * 1.0;

    gutterPx.value = Math.max(12, Math.min(22, Math.round(Math.min(vw, vh) * 0.025)));

    let panelW, panelH;

    if (isMobile.value) {
      const safeW = vw - 8;
      const safeH = vh - 8;
      panelW = safeW;
      panelH = panelW * 1.414;
      if (panelH > safeH) {
        panelH = safeH;
        panelW = panelH / 1.414;
      }
    } else {
      panelW = Math.max(200, (spreadMaxW - gutterPx.value) / 2);
      panelH = panelW * 1.414;
      if (panelH > spreadMaxH) {
        panelH = spreadMaxH;
        panelW = panelH / 1.414;
      }
      panelH = Math.max(320, Math.min(860, panelH));
      panelW = panelH / 1.414;
    }

    rig.spreadW = Math.round(panelW * 2 + gutterPx.value);
    rig.spreadH = Math.round(panelH);

    const maxPan = Math.max(0, rig.spreadW / 2);
    rig.maxPanX = Math.round(maxPan);
    rig.minPanX = -rig.maxPanX;

    const padY = Math.round(panelH * 0.18);
    rig.maxPanY = padY;
    rig.minPanY = -padY;

    // 只有当这是第一次计算（pan为0）或者 resize 时，才重置位置
    // 如果已经在操作中，尽量保持不动，除非越界
    // 但这里为了简单，我们初始化时往往需要重置
    // rig.panX = 0;
    // rig.panY = 0;
  };

  const onRigDown = (e: PointerEvent) => {
    if (isMobile.value) return;
    const target = e.target as HTMLElement;
    if (target.closest('.panel-inner')) return;

    rig.dragging = true;
    rig.pid = e.pointerId;
    rig.downX = e.clientX;
    rig.downY = e.clientY;
    rig.startPanX = rig.panX;
    rig.startPanY = rig.panY;
    rig.moved = false;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  };

  const onRigMove = (e: PointerEvent) => {
    if (!rig.dragging || rig.pid !== e.pointerId) return;
    const dx = e.clientX - rig.downX;
    const dy = e.clientY - rig.downY;
    if (Math.abs(dx) > 3 || Math.abs(dy) > 3) rig.moved = true;
    rig.panX = Math.max(rig.minPanX, Math.min(rig.maxPanX, rig.startPanX + dx));
    rig.panY = Math.max(rig.minPanY, Math.min(rig.maxPanY, rig.startPanY + dy));
  };

  const onRigUp = (e: PointerEvent) => {
    if (rig.pid !== e.pointerId) return;
    rig.dragging = false;
    rig.pid = null;
    if (rig.moved) rig.lastDragT = Date.now();
  };

  onMounted(() => {
    nextTick(measureRig);
    window.addEventListener('resize', measureRig, { passive: true });
  });

  onUnmounted(() => {
    window.removeEventListener('resize', measureRig);
  });

  // ✅ 关键：导出 measureRig
  return { rigEl, rig, rigStyle, isMobile, measureRig, onRigDown, onRigMove, onRigUp };
}