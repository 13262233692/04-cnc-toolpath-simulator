<template>
  <header class="top-toolbar">
    <div class="toolbar-left">
      <div class="logo">
        <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
          <path d="M14 2L4 7v14l10 5 10-5V7L14 2z" stroke="#06b6d4" stroke-width="1.5"/>
          <path d="M14 2v13" stroke="#06b6d4" stroke-width="1.5"/>
          <path d="M4 7l10 5 10-5" stroke="#06b6d4" stroke-width="1.5"/>
          <circle cx="14" cy="18" r="3" stroke="#3b82f6" stroke-width="1.5"/>
        </svg>
        <span class="logo-text">五轴仿真监控系统</span>
      </div>
      <div class="file-info" v-if="store.gcodeFile.name">
        <span class="file-name">{{ store.gcodeFile.name }}</span>
        <span class="file-size">{{ formatSize(store.gcodeFile.size) }}</span>
      </div>
    </div>

    <div class="toolbar-center">
      <button class="btn btn-primary" @click="onLoadGCode">
        <span>📂</span>
        <span>加载G-code</span>
      </button>
      <button class="btn" @click="onLoadSTL">
        <span>🧊</span>
        <span>加载STL模型</span>
      </button>
      <div class="divider-v"></div>
      <div class="camera-presets">
        <button class="btn btn-ghost" @click="$emit('camera', 'iso')" title="等轴测">⬕</button>
        <button class="btn btn-ghost" @click="$emit('camera', 'top')" title="俯视图">⬆</button>
        <button class="btn btn-ghost" @click="$emit('camera', 'front')" title="前视图">⬇</button>
        <button class="btn btn-ghost" @click="$emit('camera', 'side')" title="侧视图">➡</button>
      </div>
      <div class="divider-v"></div>
      <div class="view-toggles">
        <label class="toggle-item" :class="{ active: store.viewOptions.showGrid }" @click="toggle('showGrid')">
          <span class="toggle-icon">⊞</span>
          <span class="toggle-label">网格</span>
        </label>
        <label class="toggle-item" :class="{ active: store.viewOptions.showAxis }" @click="toggle('showAxis')">
          <span class="toggle-icon">⚑</span>
          <span class="toggle-label">坐标轴</span>
        </label>
        <label class="toggle-item" :class="{ active: store.viewOptions.showToolpath }" @click="toggle('showToolpath')">
          <span class="toggle-icon">〰</span>
          <span class="toggle-label">轨迹</span>
        </label>
        <label class="toggle-item" :class="{ active: store.viewOptions.showMachine }" @click="toggle('showMachine')">
          <span class="toggle-icon">🏭</span>
          <span class="toggle-label">机床</span>
        </label>
      </div>
    </div>

    <div class="toolbar-right">
      <div class="status-indicator">
        <div class="status-dot" :class="statusClass"></div>
        <span class="status-text">{{ statusText }}</span>
      </div>
      <div class="perf-info" v-if="parseMeta">
        <span class="perf-item">
          <span class="perf-label">解析</span>
          <span class="perf-value">{{ parseMeta.parse_time_ms?.toFixed(1) }}ms</span>
        </span>
        <span class="perf-item">
          <span class="perf-label">求解</span>
          <span class="perf-value">{{ ikMeta?.solve_time_ms?.toFixed(1) || '-' }}ms</span>
        </span>
      </div>
    </div>
  </header>
</template>

<script setup>
import { computed } from 'vue'
import { useSimulatorStore } from '../stores/simulator'

const emit = defineEmits(['camera'])
const store = useSimulatorStore()

defineExpose({
  onLoadGCode: () => {},
})

const parseMeta = computed(() => store.parseMetadata)
const ikMeta = computed(() => store.ikMetadata)

const statusClass = computed(() => {
  if (store.processingStatus.parsing || store.processingStatus.solving) return 'processing'
  if (store.processingStatus.error) return 'error'
  if (store.pointCount > 0) return 'ready'
  return 'idle'
})

const statusText = computed(() => {
  if (store.processingStatus.parsing) return '解析中...'
  if (store.processingStatus.solving) return '运动学求解中...'
  if (store.processingStatus.error) return '错误'
  if (store.pointCount > 0) return '就绪'
  return '待机'
})

function toggle(key) {
  const current = store.viewOptions[key]
  store.updateViewOptions({ [key]: !current })
}

async function onLoadGCode() {
  try {
    const result = await window.fileAPI.openFileDialog({
      title: '选择G-code文件',
      filters: [{ name: 'G-code', extensions: ['nc', 'gcode', 'tap', 'cnc', 'txt'] }],
    })
    if (!result?.ok) return
    store.setProcessing({ parsing: true, error: null })
    store.setGCodeFile({
      path: result.filePath,
      name: result.filePath.split(/[\\/]/).pop(),
      size: 0,
    })

    const fileInfo = await window.fileAPI.readFile(result.filePath)
    if (fileInfo?.ok) {
      store.setGCodeFile({ size: fileInfo.size })
    }

    const pipelineResult = await window.cncAPI.processPipeline({
      filePath: result.filePath,
      useMmap: true,
      machineConfig: store.machineConfig,
      toolParams: store.toolParams,
    })

    if (!pipelineResult?.ok) {
      store.setProcessing({ parsing: false, error: pipelineResult?.error })
      return
    }

    const data = pipelineResult.data || {}
    store.setParseResult(data.parse_metadata || data.parseMetadata, data.point_count || data.pointCount)
    store.setIKResult(data.ik_metadata || data.ikMetadata)
    store.setSharedBuffers({
      cartesianSAB: (data.cartesian_buffer || data.cartesianBuffer)?.buffer,
      machineSAB: (data.machine_buffer || data.machineBuffer)?.buffer,
    })
    store.setProcessing({ parsing: false })
  } catch (e) {
    store.setProcessing({ parsing: false, error: e.message })
    console.error('加载G-code失败:', e)
  }
}

async function onLoadSTL() {
  const result = await window.fileAPI.openFileDialog({
    title: '选择STL模型',
    filters: [{ name: 'STL', extensions: ['stl'] }],
  })
  if (!result?.ok) return
  emit('loadSTL', result.filePath)
}

function formatSize(bytes) {
  if (!bytes) return ''
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1024 / 1024).toFixed(2) + ' MB'
}
</script>

<style scoped>
.top-toolbar {
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: linear-gradient(180deg, #121721 0%, #0f141c 100%);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.toolbar-left,
.toolbar-center,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-right: 12px;
  border-right: 1px solid var(--border-color);
}

.logo-text {
  font-size: 15px;
  font-weight: 700;
  background: linear-gradient(135deg, #06b6d4 0%, #3b82f6 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  letter-spacing: 0.5px;
}

.file-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.file-name {
  font-size: 12px;
  color: var(--text-primary);
  font-weight: 500;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  font-size: 11px;
  color: var(--text-muted);
}

.divider-v {
  width: 1px;
  height: 24px;
  background: var(--border-color);
  margin: 0 4px;
}

.camera-presets,
.view-toggles {
  display: flex;
  gap: 2px;
}

.toggle-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
  color: var(--text-muted);
  font-size: 10px;
  gap: 2px;
}

.toggle-item:hover {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.toggle-item.active {
  background: rgba(6, 182, 212, 0.15);
  color: var(--accent-cyan);
}

.toggle-icon {
  font-size: 14px;
}

.toggle-label {
  font-size: 10px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
}

.status-dot.idle { background: var(--text-muted); }
.status-dot.ready { background: var(--accent-green); box-shadow: 0 0 8px var(--accent-green); }
.status-dot.processing { background: var(--accent-yellow); box-shadow: 0 0 8px var(--accent-yellow); animation: pulse 1s infinite; }
.status-dot.error { background: var(--accent-red); }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.status-text {
  font-size: 11px;
  font-weight: 500;
}

.perf-info {
  display: flex;
  gap: 12px;
  padding: 4px 10px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.perf-item {
  display: flex;
  gap: 6px;
  flex-direction: column;
  align-items: flex-end;
}

.perf-label {
  font-size: 9px;
  color: var(--text-muted);
}

.perf-value {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent-cyan);
  font-variant-numeric: tabular-nums;
}
</style>
