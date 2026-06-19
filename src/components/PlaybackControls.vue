<template>
  <div class="playback-controls">
    <div class="progress-section">
      <div class="progress-info">
        <div class="progress-labels">
          <span class="label-item">
            <span class="label-text">进度</span>
            <span class="label-value">{{ formatIndex(store.currentIndex) }}</span>
          </span>
          <span class="label-sep">/</span>
          <span class="label-item">
            <span class="label-value total">{{ formatIndex(store.pointCount) }}</span>
            <span class="label-text">点</span>
          </span>
        </div>
        <div class="progress-percent">{{ store.currentProgress.toFixed(2) }}%</div>
      </div>
      <div class="progress-track" ref="trackRef" @mousedown="onSeekStart" @click="onSeekClick">
        <div class="progress-fill" :style="{ width: store.currentProgress + '%' }"></div>
        <div class="progress-thumb" :style="{ left: store.currentProgress + '%' }"></div>
      </div>
      <div class="progress-minimap" v-if="store.pointCount > 0">
        <canvas ref="minimapCanvas"></canvas>
      </div>
    </div>

    <div class="controls-section">
      <div class="transport-controls">
        <button class="btn btn-ghost" @click="jumpTo(0)" :disabled="store.pointCount === 0" title="开始">⏮</button>
        <button class="btn btn-ghost" @click="stepBack" :disabled="store.pointCount === 0" title="后退一步">⏪</button>
        <button
          class="btn play-btn"
          :class="store.isPlaying ? 'btn-warn' : 'btn-success'"
          @click="togglePlay"
          :disabled="store.pointCount === 0"
        >
          {{ store.isPlaying ? '⏸ 暂停' : '▶ 播放' }}
        </button>
        <button class="btn btn-ghost" @click="stepForward" :disabled="store.pointCount === 0" title="前进一步">⏩</button>
        <button class="btn btn-ghost" @click="jumpTo(store.pointCount - 1)" :disabled="store.pointCount === 0" title="结束">⏭</button>
      </div>

      <div class="divider-v"></div>

      <div class="speed-control">
        <span class="speed-label">速度</span>
        <button class="btn speed-btn" @click="setSpeed(store.playSpeed * 0.5)" :disabled="store.pointCount === 0">½</button>
        <div class="speed-value">{{ store.playSpeed.toFixed(1) }}x</div>
        <button class="btn speed-btn" @click="setSpeed(store.playSpeed * 2)" :disabled="store.pointCount === 0">2x</button>
        <input
          type="range"
          class="speed-slider"
          min="0.1"
          max="100"
          step="0.1"
          :value="store.playSpeed"
          @input="onSpeedSlider"
          :disabled="store.pointCount === 0"
        />
      </div>

      <div class="divider-v"></div>

      <div class="mode-control">
        <div class="mode-btn" :class="{ active: store.playbackMode === 'realtime' }" @click="setMode('realtime')">
          实时
        </div>
        <div class="mode-btn" :class="{ active: store.playbackMode === 'fast' }" @click="setMode('fast')">
          快速
        </div>
      </div>

      <div class="divider-v"></div>

      <div class="time-info">
        <div class="time-item">
          <span class="time-label">已加工</span>
          <span class="time-value">{{ formatTime(elapsedTime) }}</span>
        </div>
        <div class="time-item">
          <span class="time-label">总预计</span>
          <span class="time-value">{{ formatTime(estimatedTotal) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useSimulatorStore } from '../stores/simulator'

const store = useSimulatorStore()
const trackRef = ref(null)
const minimapCanvas = ref(null)

const elapsedTime = ref(0)
let rafId = null
let playStartTimestamp = 0

const estimatedTotal = computed(() => {
  return store.parseMetadata?.estimated_time || 0
})

watch(() => store.isPlaying, (playing) => {
  if (playing) {
    playStartTimestamp = performance.now()
    startTimer()
  } else {
    stopTimer()
  }
})

watch(() => store.pointCount, () => {
  nextTick(drawMinimap)
})

function startTimer() {
  const tick = () => {
    if (!store.isPlaying) return
    elapsedTime.value = (performance.now() - playStartTimestamp) / 1000
    rafId = requestAnimationFrame(tick)
  }
  tick()
}

function stopTimer() {
  if (rafId) cancelAnimationFrame(rafId)
}

function togglePlay() {
  store.setPlaying(!store.isPlaying)
}

function stepBack() {
  store.setCurrentIndex(Math.max(0, store.currentIndex - 1))
}

function stepForward() {
  store.setCurrentIndex(Math.min(store.pointCount - 1, store.currentIndex + 1))
}

function jumpTo(idx) {
  store.setCurrentIndex(idx)
}

function setSpeed(s) {
  store.setPlaySpeed(Math.max(0.1, Math.min(100, s)))
}

function onSpeedSlider(e) {
  store.setPlaySpeed(parseFloat(e.target.value))
}

function setMode(mode) {
  store.playbackMode = mode
}

let isDragging = false

function onSeekStart(e) {
  isDragging = true
  onSeek(e)
  window.addEventListener('mousemove', onSeek)
  window.addEventListener('mouseup', onSeekEnd)
}

function onSeekClick(e) {
  if (!isDragging) onSeek(e)
}

function onSeek(e) {
  if (!trackRef.value || store.pointCount === 0) return
  const rect = trackRef.value.getBoundingClientRect()
  const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
  store.setCurrentIndex(x * store.pointCount)
}

function onSeekEnd() {
  setTimeout(() => { isDragging = false }, 10)
  window.removeEventListener('mousemove', onSeek)
  window.removeEventListener('mouseup', onSeekEnd)
}

onMounted(() => {
  drawMinimap()
})

onUnmounted(() => {
  stopTimer()
  window.removeEventListener('mousemove', onSeek)
  window.removeEventListener('mouseup', onSeekEnd)
})

function drawMinimap() {
  if (!minimapCanvas.value || !store.sharedBuffers.cartesianSAB || !store.sharedBuffers.fieldOffsets) {
    return
  }
  const canvas = minimapCanvas.value
  const ctx = canvas.getContext('2d')
  const sab = store.sharedBuffers.cartesianSAB
  const off = store.sharedBuffers.fieldOffsets.cartesian
  const count = store.pointCount
  if (!off || count < 2) return

  canvas.width = canvas.offsetWidth * 2
  canvas.height = canvas.offsetHeight * 2
  ctx.scale(2, 2)
  const w = canvas.offsetWidth
  const h = canvas.offsetHeight
  ctx.clearRect(0, 0, w, h)

  const dv = new DataView(sab)
  let minZ = Infinity, maxZ = -Infinity
  let minX = Infinity, maxX = -Infinity
  const stepDraw = Math.max(1, Math.floor(count / 5000))

  for (let i = 0; i < count; i += stepDraw) {
    const x = dv.getFloat64(off.x_offset + i * off.stride, true)
    const z = dv.getFloat64(off.z_offset + i * off.stride, true)
    if (x < minX) minX = x
    if (x > maxX) maxX = x
    if (z < minZ) minZ = z
    if (z > maxZ) maxZ = z
  }

  const rangeX = maxX - minX || 1
  const rangeZ = maxZ - minZ || 1
  const scale = Math.min((w - 8) / rangeX, (h - 8) / rangeZ)
  const ox = (w - rangeX * scale) / 2
  const oy = (h - rangeZ * scale) / 2

  ctx.beginPath()
  for (let i = 0; i < count; i += stepDraw) {
    const x = dv.getFloat64(off.x_offset + i * off.stride, true)
    const z = dv.getFloat64(off.z_offset + i * off.stride, true)
    const px = ox + (x - minX) * scale
    const py = oy + (z - minZ) * scale
    if (i === 0) ctx.moveTo(px, py)
    else ctx.lineTo(px, py)
  }
  ctx.strokeStyle = 'rgba(6, 182, 212, 0.6)'
  ctx.lineWidth = 0.5
  ctx.stroke()

  const curIdx = Math.floor(store.currentIndex)
  const curX = dv.getFloat64(off.x_offset + curIdx * off.stride, true)
  const curZ = dv.getFloat64(off.z_offset + curIdx * off.stride, true)
  const cpx = ox + (curX - minX) * scale
  const cpy = oy + (curZ - minZ) * scale
  ctx.beginPath()
  ctx.arc(cpx, cpy, 3, 0, Math.PI * 2)
  ctx.fillStyle = '#f59e0b'
  ctx.fill()
}

watch([() => store.currentIndex, () => store.sharedBuffers.cartesianSAB], drawMinimap)

function formatIndex(n) {
  return Math.floor(n).toLocaleString()
}

function formatTime(s) {
  if (!s || !isFinite(s)) return '0:00'
  s = Math.max(0, s)
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = Math.floor(s % 60)
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${sec.toString().padStart(2, '0')}`
  return `${m}:${sec.toString().padStart(2, '0')}`
}
</script>

<style scoped>
.playback-controls {
  height: 140px;
  background: linear-gradient(180deg, #0f141c 0%, #121721 100%);
  border-top: 1px solid var(--border-color);
  padding: 10px 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-labels {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.label-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.label-text {
  font-size: 11px;
  color: var(--text-muted);
}

.label-value {
  font-size: 13px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--accent-cyan);
}

.label-value.total {
  color: var(--text-secondary);
}

.label-sep {
  color: var(--text-muted);
}

.progress-percent {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
  padding: 2px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.progress-track {
  position: relative;
  height: 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  cursor: pointer;
  overflow: visible;
}

.progress-fill {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  background: linear-gradient(90deg, var(--accent-cyan), var(--accent-blue));
  border-radius: 4px;
  transition: width 0.05s linear;
}

.progress-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 14px;
  height: 14px;
  background: var(--accent-yellow);
  border: 2px solid #fff;
  border-radius: 50%;
  box-shadow: 0 2px 8px rgba(0,0,0,0.4);
  transition: left 0.05s linear;
  z-index: 2;
}

.progress-minimap {
  height: 36px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  margin-top: 2px;
}

.progress-minimap canvas {
  width: 100%;
  height: 100%;
  display: block;
}

.controls-section {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
}

.transport-controls {
  display: flex;
  gap: 4px;
}

.play-btn {
  min-width: 110px;
  font-weight: 600;
}

.divider-v {
  width: 1px;
  height: 28px;
  background: var(--border-color);
}

.speed-control {
  display: flex;
  align-items: center;
  gap: 6px;
}

.speed-label {
  font-size: 11px;
  color: var(--text-muted);
}

.speed-btn {
  padding: 4px 10px;
  font-weight: 600;
}

.speed-value {
  min-width: 48px;
  text-align: center;
  font-weight: 600;
  color: var(--accent-cyan);
  font-variant-numeric: tabular-nums;
}

.speed-slider {
  width: 120px;
  height: 4px;
}

.speed-slider::-webkit-slider-runnable-track {
  background: var(--bg-tertiary);
  border-radius: 2px;
}

.speed-slider::-webkit-slider-thumb {
  margin-top: -4px;
  width: 12px;
  height: 12px;
  background: var(--accent-cyan);
  border-radius: 50%;
  cursor: pointer;
}

.mode-control {
  display: flex;
  background: var(--bg-tertiary);
  border-radius: 4px;
  padding: 2px;
}

.mode-btn {
  padding: 4px 12px;
  font-size: 11px;
  border-radius: 3px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.15s;
}

.mode-btn.active {
  background: var(--accent-blue);
  color: #fff;
}

.time-info {
  display: flex;
  gap: 16px;
  margin-left: auto;
}

.time-item {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.time-label {
  font-size: 10px;
  color: var(--text-muted);
}

.time-value {
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
}
</style>
