<template>
  <div class="viewport-3d" ref="containerRef">
    <div v-if="store.processingStatus.parsing || store.processingStatus.solving" class="loading-overlay">
      <div class="loading-spinner"></div>
      <div class="loading-text">
        {{ store.processingStatus.parsing ? 'G-code 解析中...' : '运动学求解中...' }}
      </div>
    </div>
    <div v-if="!store.pointCount && !store.processingStatus.parsing" class="empty-hint">
      <div class="empty-icon">
        <svg width="80" height="80" viewBox="0 0 80 80" fill="none">
          <circle cx="40" cy="40" r="35" stroke="#2a3446" stroke-width="2"/>
          <path d="M20 55L35 35L45 45L60 25" stroke="#3b82f6" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
          <circle cx="20" cy="55" r="3" fill="#3b82f6"/>
          <circle cx="60" cy="25" r="3" fill="#10b981"/>
        </svg>
      </div>
      <div class="empty-title">五轴数控机床仿真系统</div>
      <div class="empty-desc">点击顶部「加载G-code」按钮开始仿真</div>
      <div class="empty-features">
        <span class="feature-tag">✓ Rust极速解析</span>
        <span class="feature-tag">✓ RTCP刀具偏置</span>
        <span class="feature-tag">✓ SharedArrayBuffer零拷贝</span>
        <span class="feature-tag">✓ WebGL高频渲染</span>
      </div>
      <button class="btn btn-primary btn-lg" style="margin-top:20px" @click="$emit('loadGCode')">
        <span>📂</span>
        <span>加载G-code文件</span>
      </button>
    </div>
    <div v-if="store.processingStatus.error" class="error-banner">
      <span class="error-icon">⚠</span>
      <span class="error-text">{{ store.processingStatus.error }}</span>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useSimulatorStore } from '../stores/simulator'
import { CNCScene } from '../core/CNCScene'

const props = defineProps({})
const emit = defineEmits(['frame', 'loadGCode'])
const containerRef = ref(null)
const store = useSimulatorStore()
let scene = null
let lastFrameTime = 0

defineExpose({
  setCameraPreset: (preset) => scene?.setCameraPreset(preset),
  getScene: () => scene,
})

onMounted(() => {
 if (containerRef.value) {
 scene = new CNCScene(containerRef.value);
 scene.init();
 scene.setOnFrameCallback(onFrameTick);
 }
});
onUnmounted(() => {
 scene?.dispose();
 scene = null;
});
watch(() => [
 store.sharedBuffers.cartesianSAB,
 store.sharedBuffers.fieldOffsets,
 store.pointCount,
], ([sab, offsets, count]) => {
 if (sab && offsets?.cartesian && count > 0) {
 scene?.loadToolpathFromShared(sab, count, offsets.cartesian, store.viewOptions.toolpathColorMode);
 scene?.setToolIndex(0, store.viewOptions.trailLength);
 updateAxisFromShared(0);
 }
}, { immediate: true });
watch(() => store.viewOptions.showGrid, (v) => {
 if (scene?.helpersGroup) {
 scene.helpersGroup.visible = v;
 }
});
watch(() => store.viewOptions.showMachine, (v) => {
 if (scene?.machineGroup) {
 scene.machineGroup.visible = v;
 }
});
watch(() => store.viewOptions.showToolpath, (v) => {
 if (scene?.toolpathGroup) {
 scene.toolpathGroup.visible = v;
 }
});
watch(() => store.currentIndex, (idx) => {
 if (!scene)
 return;
 scene.setToolIndex(idx, store.viewOptions.trailLength);
 updateAxisFromShared(idx);
}, { immediate: true });
watch(() => store.toolParams.length, (len) => {
 scene?.setToolLength(len);
});
watch(() => props.cameraPreset, (preset) => {
 scene?.setCameraPreset(preset);
});
function onFrameTick(dt, elapsed) {
 if (!store.isPlaying || !store.pointCount === 0)
 return;
 const now = performance.now();
 const step = (now - lastFrameTime) / 1000;
 if (step < 0.016)
 return;
 lastFrameTime = now;
 const baseSpeed = store.playbackMode === 'realtime'
 ? calculateRealtimeStep(step)
 : step * store.playSpeed * 500;
 const newIndex = Math.min(store.pointCount - 1, store.currentIndex + baseSpeed);
 store.setCurrentIndex(newIndex);
 emit('frame', { dt, elapsed, index: newIndex });
}
function calculateRealtimeStep(step) {
 if (!store.sharedBuffers.cartesianSAB || !store.sharedBuffers.fieldOffsets) {
 return step * 100;
 }
 const off = store.sharedBuffers.fieldOffsets.cartesian;
 if (!off)
 return step * 100;
 const dv = new DataView(store.sharedBuffers.cartesianSAB);
 const idx = Math.floor(store.currentIndex);
 const nextIdx = Math.min(store.pointCount - 1, idx + 1);
 const feed = dv.getFloat64(off.feedrate_offset + idx * off.stride, true) || 100;
 if (feed === 0)
 return step * 100;
 const x1 = dv.getFloat64(off.x_offset + idx * off.stride, true);
 const y1 = dv.getFloat64(off.y_offset + idx * off.stride, true);
 const z1 = dv.getFloat64(off.z_offset + idx * off.stride, true);
 const x2 = dv.getFloat64(off.x_offset + nextIdx * off.stride, true);
 const y2 = dv.getFloat64(off.y_offset + nextIdx * off.stride, true);
 const z2 = dv.getFloat64(off.z_offset + nextIdx * off.stride, true);
 const dist = Math.hypot(x2 - x1, y2 - y1, z2 - z1);
 if (dist < 1e-6)
 return 1;
 const feedMmPerSec = feed / 60 * store.playSpeed;
 const segTime = dist / Math.max(1, feedMmPerSec);
 return Math.min(10, (step / Math.max(0.001, segTime)));
}
function updateAxisFromShared(idx) {
 if (!store.sharedBuffers.machineSAB || !store.sharedBuffers.fieldOffsets)
 return;
 const mOff = store.sharedBuffers.fieldOffsets.machine;
 const cOff = store.sharedBuffers.fieldOffsets.cartesian;
 if (!mOff) {
 return;
 }
 const dv = new DataView(store.sharedBuffers.machineSAB);
 const i = Math.floor(idx);
 store.setCurrentAxis({
 x: dv.getFloat64(mOff.x_offset + i * mOff.stride, true),
 y: dv.getFloat64(mOff.y_offset + i * mOff.stride, true),
 z: dv.getFloat64(mOff.z_offset + i * mOff.stride, true),
 a: dv.getFloat64(mOff.a_offset + i * mOff.stride, true),
 b: dv.getFloat64(mOff.b_offset + i * mOff.stride, true),
 c: dv.getFloat64(mOff.c_offset + i * mOff.stride, true),
 feedrate: dv.getFloat64(mOff.feedrate_offset + i * mOff.stride, true),
 spindle: cOff ? dv.getFloat64(cOff.spindle_offset + i * cOff.stride, true) : 0,
 });
}
</script>

<style scoped>
.viewport-3d {
  position: relative;
  flex: 1;
  min-width: 0;
  min-height: 0;
  background: #080b10;
  overflow: hidden;
}

:deep(canvas) {
  display: block;
  width: 100%;
  height: 100%;
}

.loading-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(10, 14, 20, 0.85);
  backdrop-filter: blur(4px);
  z-index: 10;
  gap: 16px;
}

.loading-spinner {
  width: 48px;
  height: 48px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-cyan);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-text {
  font-size: 14px;
  color: var(--text-secondary);
  letter-spacing: 1px;
}

.empty-hint {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 5;
  pointer-events: none;
}

.empty-icon {
  opacity: 0.8;
  margin-bottom: 24px;
}

.empty-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 8px;
  letter-spacing: 2px;
}

.empty-desc {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 20px;
}

.empty-features {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: center;
  max-width: 600px;
}

.feature-tag {
  padding: 4px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 11px;
  color: var(--accent-cyan);
}

.error-banner {
  position: absolute;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  color: var(--accent-red);
  font-size: 13px;
  z-index: 20;
  max-width: 80%;
}

.error-icon {
  font-size: 16px;
}
</style>
