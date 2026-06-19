<template>
  <aside class="right-sidebar">
    <div class="panel axis-monitor-panel">
      <div class="panel-header">
        <div class="panel-title">
          <span class="panel-icon">📐</span>
          <span>坐标轴实时监控</span>
        </div>
        <div class="status-pill" :class="{ active: store.isPlaying }">
          {{ store.isPlaying ? '运行中' : '已停止' }}
        </div>
      </div>
      <div class="panel-body">
        <div class="axis-list">
          <div v-for="axis in axisList" :key="axis.key" class="axis-row" :class="axis.key">
            <div class="axis-name">{{ axis.label }}</div>
            <div class="axis-value-wrap">
              <div class="axis-value">{{ formatAxis(store.currentAxis[axis.key]) }}</div>
              <div class="axis-unit">{{ axis.unit }}</div>
            </div>
            <div class="axis-bar-wrap">
              <div class="axis-bar" :style="{
                width: getBarWidth(axis.key) + '%',
                background: axis.color,
              }"></div>
            </div>
            <div class="axis-range">
              <span class="range-min">{{ axis.min }}</span>
              <span class="range-max">{{ axis.max }}</span>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <div class="spindle-feed-section">
          <div class="sf-item feed">
            <div class="sf-icon">🚀</div>
            <div class="sf-info">
              <div class="sf-label">进给速度</div>
              <div class="sf-value">{{ formatNum(store.currentAxis.feedrate) }}<span class="sf-unit">mm/min</span></div>
            </div>
          </div>
          <div class="sf-item spindle">
            <div class="sf-icon">⚙️</div>
            <div class="sf-info">
              <div class="sf-label">主轴转速</div>
              <div class="sf-value">{{ formatNum(store.currentAxis.spindle) }}<span class="sf-unit">RPM</span></div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="panel machine-config-panel">
      <div class="panel-header">
        <div class="panel-title">
          <span class="panel-icon">🏗️</span>
          <span>机床结构配置</span>
        </div>
      </div>
      <div class="panel-body">
        <div class="axis-type-selector">
          <div class="selector-label">机床结构类型</div>
          <div class="type-options">
            <div
              class="type-option"
              :class="{ active: store.machineConfig.axis_type === 1 }"
              @click="setAxisType(1)"
            >
              <div class="type-icon">📐</div>
              <div class="type-name">BC 摆头式</div>
            </div>
            <div
              class="type-option"
              :class="{ active: store.machineConfig.axis_type === 2 }"
              @click="setAxisType(2)"
            >
              <div class="type-icon">🔄</div>
              <div class="type-name">AC 转台式</div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <div class="config-section-title">旋转轴支点位置</div>
        <div class="config-grid-3">
          <div class="config-field small">
            <span class="cf-label">A·X</span>
            <input type="number" class="cf-input" v-model.number="store.machineConfig.rotary_a_pivot_x" />
          </div>
          <div class="config-field small">
            <span class="cf-label">A·Y</span>
            <input type="number" class="cf-input" v-model.number="store.machineConfig.rotary_a_pivot_y" />
          </div>
          <div class="config-field small">
            <span class="cf-label">A·Z</span>
            <input type="number" class="cf-input" v-model.number="store.machineConfig.rotary_a_pivot_z" />
          </div>
          <div class="config-field small">
            <span class="cf-label">B·X</span>
            <input type="number" class="cf-input" v-model.number="store.machineConfig.rotary_b_pivot_x" />
          </div>
          <div class="config-field small">
            <span class="cf-label">B·Y</span>
            <input type="number" class="cf-input" v-model.number="store.machineConfig.rotary_b_pivot_y" />
          </div>
          <div class="config-field small">
            <span class="cf-label">B·Z</span>
            <input type="number" class="cf-input" v-model.number="store.machineConfig.rotary_b_pivot_z" />
          </div>
        </div>

        <div class="divider"></div>

        <div class="config-section-title">线性轴行程范围</div>
        <div class="travel-config">
          <div v-for="axis in ['x', 'y', 'z']" :key="axis" class="travel-row">
            <span class="travel-axis" :class="axis">{{ axis.toUpperCase() }}</span>
            <input type="number" class="travel-input" v-model.number="store.machineConfig[`${axis}_min`]" />
            <div class="travel-arrow">↔</div>
            <input type="number" class="travel-input" v-model.number="store.machineConfig[`${axis}_max`]" />
          </div>
        </div>

        <div class="divider"></div>

        <div class="config-section-title">旋转轴角度范围</div>
        <div class="travel-config">
          <div v-for="axis in ['a', 'b', 'c']" :key="axis" class="travel-row">
            <span class="travel-axis" :class="axis">{{ axis.toUpperCase() }}</span>
            <input type="number" class="travel-input" v-model.number="store.machineConfig[`${axis}_min`]" />
            <div class="travel-arrow">↔</div>
            <input type="number" class="travel-input" v-model.number="store.machineConfig[`${axis}_max`]" />
          </div>
        </div>
      </div>
    </div>

    <div class="panel options-panel">
      <div class="panel-header">
        <div class="panel-title">
          <span class="panel-icon">🎛️</span>
          <span>显示选项</span>
        </div>
      </div>
      <div class="panel-body">
        <div class="option-group">
          <div class="option-label">轨迹颜色模式</div>
          <div class="color-mode-options">
            <div
              v-for="mode in colorModes"
              :key="mode.value"
              class="cm-option"
              :class="{ active: store.viewOptions.toolpathColorMode === mode.value }"
              @click="setColorMode(mode.value)"
            >
              <div class="cm-preview" :style="{ background: mode.gradient }"></div>
              <span class="cm-label">{{ mode.label }}</span>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <div class="option-group">
          <div class="option-label">拖尾长度: {{ store.viewOptions.trailLength }} 点</div>
          <input
            type="range"
            class="trail-slider"
            min="0"
            max="5000"
            step="50"
            v-model.number="store.viewOptions.trailLength"
          />
        </div>

        <div class="divider"></div>

        <div class="option-group checkbox-group">
          <label class="checkbox-item">
            <input type="checkbox" v-model="store.viewOptions.showWorkpiece" />
            <span class="cb-label">显示工件</span>
          </label>
          <label class="checkbox-item">
            <input type="checkbox" v-model="store.viewOptions.showTrail" />
            <span class="cb-label">显示拖尾</span>
          </label>
        </div>
      </div>
    </div>

    <div class="panel collision-alarm-panel" :class="{ 'collision-active': store.collisionStatus.active }">
      <div class="panel-header">
        <div class="panel-title">
          <span class="panel-icon alarm-icon">🚨</span>
          <span>碰撞检测</span>
        </div>
        <div class="alarm-status-pill" :class="{ active: store.collisionStatus.active }">
          {{ store.collisionStatus.active ? '警报' : '安全' }}
        </div>
      </div>
      <div class="panel-body">
        <div class="alarm-lamp-wrap">
          <div class="alarm-lamp" :class="{ flashing: store.collisionStatus.active }">
            <div class="lamp-glow"></div>
            <div class="lamp-bulb"></div>
          </div>
          <div class="alarm-label">
            {{ store.collisionStatus.active ? '干涉警报！' : '系统正常' }}
          </div>
        </div>

        <div class="divider"></div>

        <div class="collision-detail">
          <div class="cd-row">
            <span class="cd-label">刀柄-工件</span>
            <span class="cd-value" :class="{ danger: store.collisionStatus.workpieceCollision }">
              {{ store.collisionStatus.workpieceCollision ? '干涉' : '正常' }}
            </span>
          </div>
          <div class="cd-row">
            <span class="cd-label">刀柄-治具</span>
            <span class="cd-value" :class="{ danger: store.collisionStatus.fixtureCollision }">
              {{ store.collisionStatus.fixtureCollision ? '干涉' : '正常' }}
            </span>
          </div>
          <div class="cd-row">
            <span class="cd-label">累计警报</span>
            <span class="cd-value count">{{ store.collisionStatus.collisionCount }} 次</span>
          </div>
        </div>
      </div>
    </div>

    <div class="panel voxel-panel">
      <div class="panel-header">
        <div class="panel-title">
          <span class="panel-icon">🧊</span>
          <span>体素切削模拟</span>
        </div>
        <label class="voxel-toggle">
          <input type="checkbox" v-model="store.viewOptions.enableVoxel" />
          <span class="toggle-track"></span>
        </label>
      </div>
      <div class="panel-body">
        <div class="option-group" v-if="store.voxelStats">
          <div class="stat-grid">
            <div class="stat-item">
              <div class="stat-label">体素网格</div>
              <div class="stat-value">{{ formatGridSize(store.voxelStats.gridSize) }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">体素尺寸</div>
              <div class="stat-value">{{ store.voxelStats.voxelSize.toFixed(1) }} mm</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">实体体素</div>
              <div class="stat-value">{{ formatNum(store.voxelStats.solidVoxels) }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">去除率</div>
              <div class="stat-value removed">{{ (store.voxelStats.removedRatio * 100).toFixed(1) }}%</div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <div class="option-group">
          <div class="option-label">体素分辨率: {{ store.viewOptions.voxelSize.toFixed(1) }} mm</div>
          <input
            type="range"
            class="voxel-slider"
            min="1"
            max="10"
            step="0.5"
            v-model.number="store.viewOptions.voxelSize"
          />
        </div>

        <div class="divider"></div>

        <div class="option-group checkbox-group">
          <label class="checkbox-item">
            <input type="checkbox" v-model="store.viewOptions.showVoxelWorkpiece" />
            <span class="cb-label">显示体素工件</span>
          </label>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup>
import { computed } from 'vue'
import { useSimulatorStore } from '../stores/simulator'

const store = useSimulatorStore()

const axisList = computed(() => [
  { key: 'x', label: 'X 轴', unit: 'mm', min: store.machineConfig.x_min, max: store.machineConfig.x_max, color: 'linear-gradient(90deg, #ef4444, #f87171)' },
  { key: 'y', label: 'Y 轴', unit: 'mm', min: store.machineConfig.y_min, max: store.machineConfig.y_max, color: 'linear-gradient(90deg, #10b981, #34d399)' },
  { key: 'z', label: 'Z 轴', unit: 'mm', min: store.machineConfig.z_min, max: store.machineConfig.z_max, color: 'linear-gradient(90deg, #3b82f6, #60a5fa)' },
  { key: 'a', label: 'A 轴', unit: '°', min: store.machineConfig.a_min, max: store.machineConfig.a_max, color: 'linear-gradient(90deg, #f59e0b, #fbbf24)' },
  { key: 'b', label: 'B 轴', unit: '°', min: store.machineConfig.b_min, max: store.machineConfig.b_max, color: 'linear-gradient(90deg, #8b5cf6, #a78bfa)' },
  { key: 'c', label: 'C 轴', unit: '°', min: store.machineConfig.c_min, max: store.machineConfig.c_max, color: 'linear-gradient(90deg, #06b6d4, #22d3ee)' },
])

const colorModes = [
  { value: 'feedrate', label: '进给率', gradient: 'linear-gradient(90deg, #00f, #0ff, #0f0, #ff0, #f00)' },
  { value: 'axis', label: '高度值', gradient: 'linear-gradient(180deg, #ef4444, #f59e0b, #10b981, #3b82f6, #8b5cf6)' },
  { value: 'solid', label: '单色', gradient: 'linear-gradient(90deg, #06b6d4, #06b6d4)' },
]

function formatAxis(v) {
  if (!isFinite(v)) return '0.000'
  return v.toFixed(3)
}

function formatNum(v) {
  return Math.floor(v || 0).toLocaleString()
}

function getBarWidth(key) {
  const axis = axisList.value.find(a => a.key === key)
  if (!axis) return 0
  const v = store.currentAxis[key] || 0
  return Math.max(0, Math.min(100, ((v - axis.min) / (axis.max - axis.min)) * 100))
}

function setAxisType(t) {
  store.updateMachineConfig({ axis_type: t })
}

function setColorMode(m) {
  store.updateViewOptions({ toolpathColorMode: m })
}

function formatGridSize(size) {
  if (!size || !Array.isArray(size)) return '-'
  return `${size[0]}×${size[1]}×${size[2]}`
}
</script>

<style scoped>
.right-sidebar {
  width: 340px;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
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

.status-pill {
  padding: 3px 10px;
  background: rgba(156, 163, 175, 0.2);
  color: var(--text-muted);
  border-radius: 10px;
  font-size: 10px;
  font-weight: 500;
}

.status-pill.active {
  background: rgba(16, 185, 129, 0.15);
  color: var(--accent-green);
  animation: blink 1.5s infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.axis-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.axis-row {
  display: grid;
  grid-template-columns: 44px 1fr;
  grid-template-rows: auto auto auto;
  gap: 2px 10px;
  align-items: center;
  padding: 8px 10px;
  background: var(--bg-tertiary);
  border-radius: 6px;
  border-left: 3px solid;
}

.axis-row.x { border-left-color: var(--accent-red); }
.axis-row.y { border-left-color: var(--accent-green); }
.axis-row.z { border-left-color: var(--accent-blue); }
.axis-row.a { border-left-color: var(--accent-yellow); }
.axis-row.b { border-left-color: var(--accent-purple); }
.axis-row.c { border-left-color: var(--accent-cyan); }

.axis-name {
  grid-row: 1 / 4;
  font-size: 14px;
  font-weight: 700;
  text-align: center;
  padding: 4px 6px;
  border-radius: 4px;
}

.axis-row.x .axis-name { background: rgba(239, 68, 68, 0.15); color: var(--accent-red); }
.axis-row.y .axis-name { background: rgba(16, 185, 129, 0.15); color: var(--accent-green); }
.axis-row.z .axis-name { background: rgba(59, 130, 246, 0.15); color: var(--accent-blue); }
.axis-row.a .axis-name { background: rgba(245, 158, 11, 0.15); color: var(--accent-yellow); }
.axis-row.b .axis-name { background: rgba(139, 92, 246, 0.15); color: var(--accent-purple); }
.axis-row.c .axis-name { background: rgba(6, 182, 212, 0.15); color: var(--accent-cyan); }

.axis-value-wrap {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.axis-value {
  font-size: 15px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
}

.axis-unit {
  font-size: 10px;
  color: var(--text-muted);
}

.axis-bar-wrap {
  height: 4px;
  background: var(--bg-primary);
  border-radius: 2px;
  overflow: hidden;
}

.axis-bar {
  height: 100%;
  border-radius: 2px;
  transition: width 0.05s linear;
}

.axis-range {
  display: flex;
  justify-content: space-between;
  font-size: 9px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

.spindle-feed-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.sf-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  background: var(--bg-tertiary);
  border-radius: 6px;
}

.sf-icon {
  font-size: 24px;
  opacity: 0.9;
}

.sf-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sf-label {
  font-size: 10px;
  color: var(--text-muted);
}

.sf-value {
  font-size: 14px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
}

.sf-unit {
  font-size: 9px;
  color: var(--text-muted);
  margin-left: 2px;
  font-weight: 400;
}

.sf-item.feed .sf-value { color: var(--accent-cyan); }
.sf-item.spindle .sf-value { color: var(--accent-orange); }

.axis-type-selector {
  margin-bottom: 4px;
}

.selector-label {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.type-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.type-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 10px;
  background: var(--bg-tertiary);
  border: 2px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.type-option:hover {
  border-color: var(--border-hover);
}

.type-option.active {
  border-color: var(--accent-blue);
  background: rgba(59, 130, 246, 0.1);
}

.type-icon {
  font-size: 22px;
}

.type-name {
  font-size: 11px;
  font-weight: 500;
  color: var(--text-secondary);
}

.type-option.active .type-name {
  color: var(--accent-blue);
}

.config-section-title {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 8px;
  font-weight: 500;
}

.config-grid-3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}

.config-field {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 6px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.cf-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  white-space: nowrap;
}

.cf-input {
  flex: 1;
  width: 100%;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  outline: none;
  min-width: 0;
}

.travel-config {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.travel-row {
  display: grid;
  grid-template-columns: 28px 1fr 14px 1fr;
  gap: 6px;
  align-items: center;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.travel-axis {
  text-align: center;
  font-weight: 700;
  font-size: 12px;
  padding: 2px 0;
  border-radius: 3px;
}

.travel-axis.x { background: rgba(239,68,68,0.2); color: var(--accent-red); }
.travel-axis.y { background: rgba(16,185,129,0.2); color: var(--accent-green); }
.travel-axis.z { background: rgba(59,130,246,0.2); color: var(--accent-blue); }
.travel-axis.a { background: rgba(245,158,11,0.2); color: var(--accent-yellow); }
.travel-axis.b { background: rgba(139,92,246,0.2); color: var(--accent-purple); }
.travel-axis.c { background: rgba(6,182,212,0.2); color: var(--accent-cyan); }

.travel-input {
  width: 100%;
  padding: 4px 6px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  outline: none;
}

.travel-arrow {
  font-size: 10px;
  color: var(--text-muted);
  text-align: center;
}

.option-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.option-label {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 500;
}

.color-mode-options {
  display: flex;
  gap: 6px;
}

.cm-option {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 6px 4px;
  background: var(--bg-tertiary);
  border: 2px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
}

.cm-option:hover {
  border-color: var(--border-hover);
}

.cm-option.active {
  border-color: var(--accent-cyan);
  background: rgba(6, 182, 212, 0.08);
}

.cm-preview {
  width: 100%;
  height: 10px;
  border-radius: 2px;
}

.cm-label {
  font-size: 10px;
  color: var(--text-secondary);
}

.cm-option.active .cm-label {
  color: var(--accent-cyan);
}

.trail-slider {
  width: 100%;
  height: 4px;
}

.trail-slider::-webkit-slider-runnable-track {
  background: var(--bg-tertiary);
  border-radius: 2px;
}

.trail-slider::-webkit-slider-thumb {
  margin-top: -4px;
  width: 12px;
  height: 12px;
  background: var(--accent-purple);
  border-radius: 50%;
  cursor: pointer;
}

.checkbox-group {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.checkbox-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  color: var(--text-secondary);
}

.checkbox-item input {
  accent-color: var(--accent-cyan);
}

.cb-label {
  font-size: 11px;
}

.collision-alarm-panel {
  transition: all 0.3s;
}

.collision-alarm-panel.collision-active {
  border-color: rgba(239, 68, 68, 0.5);
  box-shadow: 0 0 20px rgba(239, 68, 68, 0.2);
}

.alarm-icon {
  display: inline-block;
}

.collision-active .alarm-icon {
  animation: shake 0.5s infinite;
}

@keyframes shake {
  0%, 100% { transform: rotate(0deg); }
  25% { transform: rotate(-10deg); }
  75% { transform: rotate(10deg); }
}

.alarm-status-pill {
  padding: 3px 10px;
  background: rgba(16, 185, 129, 0.15);
  color: var(--accent-green);
  border-radius: 10px;
  font-size: 10px;
  font-weight: 600;
}

.alarm-status-pill.active {
  background: rgba(239, 68, 68, 0.2);
  color: var(--accent-red);
  animation: pulse-alarm 0.8s infinite;
}

@keyframes pulse-alarm {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.alarm-lamp-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
}

.alarm-lamp {
  position: relative;
  width: 60px;
  height: 60px;
}

.lamp-bulb {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: #444;
  box-shadow: inset 0 -4px 8px rgba(0, 0, 0, 0.3);
  transition: all 0.15s;
}

.flashing .lamp-bulb {
  background: radial-gradient(circle at 30% 30%, #ff6b6b, #ef4444, #b91c1c);
  box-shadow: 0 0 30px #ef4444, inset 0 -4px 8px rgba(0, 0, 0, 0.2);
  animation: lamp-flash 0.4s infinite alternate;
}

@keyframes lamp-flash {
  0% {
    background: radial-gradient(circle at 30% 30%, #ff8080, #ef4444, #991b1b);
    box-shadow: 0 0 20px #ef4444, inset 0 -4px 8px rgba(0, 0, 0, 0.2);
  }
  100% {
    background: radial-gradient(circle at 30% 30%, #ffb3b3, #f87171, #dc2626);
    box-shadow: 0 0 40px #ef4444, 0 0 60px rgba(239, 68, 68, 0.5), inset 0 -4px 8px rgba(0, 0, 0, 0.2);
  }
}

.lamp-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 60px;
  height: 60px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(239, 68, 68, 0.3) 0%, transparent 70%);
  opacity: 0;
  transition: opacity 0.15s;
}

.flashing .lamp-glow {
  opacity: 1;
  animation: glow-pulse 0.4s infinite alternate;
}

@keyframes glow-pulse {
  0% { transform: translate(-50%, -50%) scale(1); opacity: 0.6; }
  100% { transform: translate(-50%, -50%) scale(1.3); opacity: 1; }
}

.alarm-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.collision-active .alarm-label {
  color: var(--accent-red);
}

.collision-detail {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cd-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.cd-label {
  font-size: 11px;
  color: var(--text-muted);
}

.cd-value {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent-green);
}

.cd-value.danger {
  color: var(--accent-red);
}

.cd-value.count {
  color: var(--text-primary);
}

.voxel-toggle {
  position: relative;
  width: 36px;
  height: 20px;
  cursor: pointer;
}

.voxel-toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-track {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--bg-primary);
  border-radius: 10px;
  transition: 0.2s;
  border: 1px solid var(--border-color);
}

.toggle-track:before {
  position: absolute;
  content: '';
  height: 14px;
  width: 14px;
  left: 2px;
  bottom: 2px;
  background: var(--text-muted);
  border-radius: 50%;
  transition: 0.2s;
}

.voxel-toggle input:checked + .toggle-track {
  background: rgba(6, 182, 212, 0.2);
  border-color: var(--accent-cyan);
}

.voxel-toggle input:checked + .toggle-track:before {
  transform: translateX(16px);
  background: var(--accent-cyan);
}

.stat-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.stat-label {
  font-size: 10px;
  color: var(--text-muted);
}

.stat-value {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.stat-value.removed {
  color: var(--accent-orange);
}

.voxel-slider {
  width: 100%;
  height: 4px;
}

.voxel-slider::-webkit-slider-runnable-track {
  background: var(--bg-tertiary);
  border-radius: 2px;
}

.voxel-slider::-webkit-slider-thumb {
  margin-top: -4px;
  width: 12px;
  height: 12px;
  background: var(--accent-cyan);
  border-radius: 50%;
  cursor: pointer;
}
</style>
