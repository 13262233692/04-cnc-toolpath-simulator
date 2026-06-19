<template>
  <aside class="left-sidebar">
    <div class="panel machine-info-panel">
      <div class="panel-header">
        <div class="panel-title">
          <span class="panel-icon">📊</span>
          <span>加工信息统计</span>
        </div>
      </div>
      <div class="panel-body" v-if="parseMeta">
        <div class="stat-grid">
          <div class="stat-item">
            <div class="stat-label">总行数</div>
            <div class="stat-value">{{ formatNum(parseMeta.total_lines) }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">总块数</div>
            <div class="stat-value">{{ formatNum(parseMeta.total_blocks) }}</div>
          </div>
          <div class="stat-item motion">
            <div class="stat-label">运动块</div>
            <div class="stat-value">{{ formatNum(parseMeta.motion_blocks) }}</div>
          </div>
          <div class="stat-item rapid">
            <div class="stat-label">快速块</div>
            <div class="stat-value">{{ formatNum(parseMeta.rapid_blocks) }}</div>
          </div>
        </div>
        <div class="divider"></div>
        <div class="bounding-info">
          <div class="bounding-title">加工范围</div>
          <div class="bounding-grid">
            <div class="bound-row x">
              <span class="axis-label">X</span>
              <span class="bound-range">{{ fmt(parseMeta.min_x) }} ~ {{ fmt(parseMeta.max_x) }}</span>
              <span class="bound-size">Δ{{ fmt(parseMeta.max_x - parseMeta.min_x) }}<span class="unit">mm</span></span>
            </div>
            <div class="bound-row y">
              <span class="axis-label">Y</span>
              <span class="bound-range">{{ fmt(parseMeta.min_y) }} ~ {{ fmt(parseMeta.max_y) }}</span>
              <span class="bound-size">Δ{{ fmt(parseMeta.max_y - parseMeta.min_y) }}<span class="unit">mm</span></span>
            </div>
            <div class="bound-row z">
              <span class="axis-label">Z</span>
              <span class="bound-range">{{ fmt(parseMeta.min_z) }} ~ {{ fmt(parseMeta.max_z) }}</span>
              <span class="bound-size">Δ{{ fmt(parseMeta.max_z - parseMeta.min_z) }}<span class="unit">mm</span></span>
            </div>
          </div>
        </div>
        <div class="divider"></div>
        <div class="distance-info">
          <div class="distance-item">
            <span class="distance-label">总路径长度</span>
            <span class="distance-value">{{ fmt(parseMeta.total_distance) }}<span class="unit">mm</span></span>
          </div>
          <div class="distance-item">
            <span class="distance-label">预计加工时间</span>
            <span class="distance-value">{{ formatTime(parseMeta.estimated_time) }}</span>
          </div>
        </div>
      </div>
      <div class="panel-body empty-state" v-else>
        <div class="empty-icon">📄</div>
        <div class="empty-text">加载G-code文件后显示加工统计</div>
      </div>
    </div>

    <div class="panel ik-info-panel">
      <div class="panel-header">
        <div class="panel-title">
          <span class="panel-icon">🔬</span>
          <span>运动学求解结果</span>
        </div>
      </div>
      <div class="panel-body" v-if="ikMeta">
        <div class="ik-summary">
          <div class="ik-ring" :class="ikQuality">
            <svg viewBox="0 0 100 100" class="ring-svg">
              <circle cx="50" cy="50" r="42" class="ring-bg" />
              <circle
                cx="50" cy="50" r="42"
                class="ring-fg"
                :style="{ strokeDashoffset: ringOffset }"
              />
            </svg>
            <div class="ring-value">{{ solveRate }}<span class="pct">%</span></div>
          </div>
          <div class="ik-stats">
            <div class="ik-stat valid">
              <span class="ik-stat-label">有效点</span>
              <span class="ik-stat-value">{{ formatNum(ikMeta.valid_points) }}</span>
            </div>
            <div class="ik-stat sing">
              <span class="ik-stat-label">奇异点</span>
              <span class="ik-stat-value">{{ formatNum(ikMeta.singularity_warnings) }}</span>
            </div>
            <div class="ik-stat oor">
              <span class="ik-stat-label">超程点</span>
              <span class="ik-stat-value">{{ formatNum(ikMeta.out_of_range_errors) }}</span>
            </div>
          </div>
        </div>
        <div class="divider"></div>
        <div class="ik-time">
          <span class="ik-time-label">求解耗时</span>
          <span class="ik-time-value">{{ ikMeta.solve_time_ms?.toFixed(2) }}<span class="unit">ms</span></span>
        </div>
      </div>
      <div class="panel-body empty-state" v-else>
        <div class="empty-icon">⚙️</div>
        <div class="empty-text">等待运动学求解...</div>
      </div>
    </div>

    <div class="panel tool-info-panel">
      <div class="panel-header">
        <div class="panel-title">
          <span class="panel-icon">🔧</span>
          <span>刀具参数</span>
        </div>
      </div>
      <div class="panel-body">
        <div class="tool-params-grid">
          <div class="param-item">
            <div class="param-label">刀具长度</div>
            <div class="param-input-wrap">
              <input type="number" class="param-input" v-model.number="store.toolParams.length" step="1" />
              <span class="param-unit">mm</span>
            </div>
          </div>
          <div class="param-item">
            <div class="param-label">刀具直径</div>
            <div class="param-input-wrap">
              <input type="number" class="param-input" v-model.number="store.toolParams.diameter" step="0.1" />
              <span class="param-unit">mm</span>
            </div>
          </div>
          <div class="param-item">
            <div class="param-label">刀尖半径</div>
            <div class="param-input-wrap">
              <input type="number" class="param-input" v-model.number="store.toolParams.radius" step="0.1" />
              <span class="param-unit">mm</span>
            </div>
          </div>
          <div class="param-item">
            <div class="param-label">刀尖角度</div>
            <div class="param-input-wrap">
              <input type="number" class="param-input" v-model.number="store.toolParams.tip_angle" step="1" />
              <span class="param-unit">°</span>
            </div>
          </div>
        </div>
        <button class="btn btn-primary w-full" style="margin-top:10px" @click="applyToolParams">
          应用参数
        </button>
      </div>
    </div>
  </aside>
</template>

<script setup>
import { computed } from 'vue'
import { useSimulatorStore } from '../stores/simulator'

const store = useSimulatorStore()

const parseMeta = computed(() => store.parseMetadata)
const ikMeta = computed(() => store.ikMetadata)

const solveRate = computed(() => {
  if (!ikMeta.value || !ikMeta.value.total_points) return 100
  return Math.round((ikMeta.value.valid_points / ikMeta.value.total_points) * 100)
})

const ringOffset = computed(() => {
  return 263.89 * (1 - solveRate.value / 100)
})

const ikQuality = computed(() => {
  if (solveRate.value >= 99) return 'excellent'
  if (solveRate.value >= 95) return 'good'
  if (solveRate.value >= 80) return 'warn'
  return 'bad'
})

function applyToolParams() {
  store.updateToolParams({ ...store.toolParams })
}

function formatNum(n) {
  return Math.floor(n || 0).toLocaleString()
}

function fmt(v) {
  if (!isFinite(v)) return '-'
  return v.toFixed(2)
}

function formatTime(s) {
  if (!s || !isFinite(s)) return '-'
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = Math.floor(s % 60)
  if (h > 0) return `${h}h ${m}m ${sec}s`
  if (m > 0) return `${m}m ${sec}s`
  return `${sec}s`
}
</script>

<style scoped>
.left-sidebar {
  width: 320px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
  overflow-y: auto;
  flex-shrink: 0;
}

.panel {
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 600;
  font-size: 13px;
  background: linear-gradient(180deg, rgba(255,255,255,0.02) 0%, transparent 100%);
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary);
}

.panel-icon {
  font-size: 14px;
}

.panel-body {
  padding: 12px 14px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px 14px;
  gap: 8px;
}

.empty-icon {
  font-size: 32px;
  opacity: 0.4;
}

.empty-text {
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
}

.stat-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.stat-item {
  padding: 8px 10px;
  background: var(--bg-tertiary);
  border-radius: 6px;
  border-left: 3px solid var(--border-color);
}

.stat-item.motion {
  border-left-color: var(--accent-blue);
}

.stat-item.rapid {
  border-left-color: var(--accent-yellow);
}

.stat-label {
  font-size: 10px;
  color: var(--text-muted);
  margin-bottom: 2px;
}

.stat-value {
  font-size: 16px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
}

.bounding-info {
  margin-top: 4px;
}

.bounding-title {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 8px;
  font-weight: 500;
}

.bounding-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.bound-row {
  display: grid;
  grid-template-columns: 20px 1fr auto;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  font-size: 11px;
}

.axis-label {
  font-weight: 700;
  font-size: 12px;
  text-align: center;
  border-radius: 3px;
  padding: 2px 0;
}

.bound-row.x .axis-label { background: rgba(239, 68, 68, 0.2); color: var(--accent-red); }
.bound-row.y .axis-label { background: rgba(16, 185, 129, 0.2); color: var(--accent-green); }
.bound-row.z .axis-label { background: rgba(59, 130, 246, 0.2); color: var(--accent-blue); }

.bound-range {
  font-variant-numeric: tabular-nums;
  color: var(--text-secondary);
}

.bound-size {
  font-weight: 600;
  color: var(--accent-cyan);
  font-variant-numeric: tabular-nums;
}

.unit {
  font-size: 9px;
  color: var(--text-muted);
  margin-left: 2px;
  font-weight: 400;
}

.distance-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.distance-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.distance-label {
  font-size: 11px;
  color: var(--text-muted);
}

.distance-value {
  font-size: 13px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--accent-cyan);
}

.ik-summary {
  display: flex;
  align-items: center;
  gap: 16px;
}

.ik-ring {
  position: relative;
  width: 88px;
  height: 88px;
  flex-shrink: 0;
}

.ring-svg {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.ring-bg {
  fill: none;
  stroke: var(--bg-tertiary);
  stroke-width: 8;
}

.ring-fg {
  fill: none;
  stroke-width: 8;
  stroke-linecap: round;
  stroke-dasharray: 263.89;
  transition: all 0.3s;
}

.ik-ring.excellent .ring-fg { stroke: var(--accent-green); }
.ik-ring.good .ring-fg { stroke: var(--accent-cyan); }
.ik-ring.warn .ring-fg { stroke: var(--accent-yellow); }
.ik-ring.bad .ring-fg { stroke: var(--accent-red); }

.ring-value {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
}

.pct {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 400;
  margin-left: 2px;
}

.ik-stats {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.ik-stat {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  font-size: 11px;
}

.ik-stat-label {
  color: var(--text-muted);
}

.ik-stat-value {
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.ik-stat.valid .ik-stat-value { color: var(--accent-green); }
.ik-stat.sing .ik-stat-value { color: var(--accent-yellow); }
.ik-stat.oor .ik-stat-value { color: var(--accent-red); }

.ik-time {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  background: rgba(6, 182, 212, 0.08);
  border: 1px solid rgba(6, 182, 212, 0.2);
  border-radius: 4px;
}

.ik-time-label {
  font-size: 11px;
  color: var(--text-secondary);
}

.ik-time-value {
  font-size: 14px;
  font-weight: 700;
  color: var(--accent-cyan);
  font-variant-numeric: tabular-nums;
}

.tool-params-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.param-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.param-label {
  font-size: 11px;
  color: var(--text-muted);
}

.param-input-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
}

.param-input {
  flex: 1;
  padding: 6px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  outline: none;
  transition: all 0.15s;
}

.param-input:focus {
  border-color: var(--accent-cyan);
  box-shadow: 0 0 0 2px rgba(6, 182, 212, 0.15);
}

.param-unit {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 24px;
}

.w-full {
  width: 100%;
}
</style>
