import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useSimulatorStore = defineStore('simulator', () => {
  const gcodeFile = ref({
    name: '',
    path: '',
    size: 0,
  })

  const parseMetadata = ref(null)
  const ikMetadata = ref(null)

  const pointCount = ref(0)
  const currentIndex = ref(0)
  const currentProgress = computed(() => pointCount.value > 0 ? (currentIndex.value / pointCount.value) * 100 : 0)

  const isPlaying = ref(false)
  const playSpeed = ref(1)
  const playbackMode = ref('realtime')

  const machineConfig = ref({
    axis_type: 1,
    rotary_a_pivot_x: 0,
    rotary_a_pivot_y: 0,
    rotary_a_pivot_z: 0,
    rotary_b_pivot_x: 0,
    rotary_b_pivot_y: 0,
    rotary_b_pivot_z: 300,
    x_min: -500,
    x_max: 500,
    y_min: -400,
    y_max: 400,
    z_min: 0,
    z_max: 500,
    a_min: -120,
    a_max: 120,
    b_min: -360,
    b_max: 360,
    c_min: -360,
    c_max: 360,
    a_is_table: true,
    b_is_table: false,
    c_is_table: false,
  })

  const toolParams = ref({
    length: 100,
    diameter: 10,
    radius: 5,
    tip_angle: 0,
    tool_number: 1,
  })

  const currentAxis = ref({
    x: 0,
    y: 0,
    z: 0,
    a: 0,
    b: 0,
    c: 0,
    feedrate: 0,
    spindle: 0,
  })

  const viewOptions = ref({
    showToolpath: true,
    showMachine: true,
    showWorkpiece: true,
    showAxis: true,
    showGrid: true,
    showTrail: true,
    toolpathColorMode: 'feedrate',
    trailLength: 500,
    cameraPerspective: 'iso',
  })

  const sharedBuffers = ref({
    cartesianSAB: null,
    machineSAB: null,
    fieldOffsets: null,
  })

  const processingStatus = ref({
    parsing: false,
    solving: false,
    error: null,
  })

  function setGCodeFile(info) {
    gcodeFile.value = { ...gcodeFile.value, ...info }
  }

  function setParseResult(metadata, count) {
    parseMetadata.value = metadata
    pointCount.value = count || metadata?.total_blocks || 0
    currentIndex.value = 0
  }

  function setIKResult(metadata) {
    ikMetadata.value = metadata
  }

  function setSharedBuffers(buffers) {
    sharedBuffers.value = { ...sharedBuffers.value, ...buffers }
  }

  function setCurrentIndex(idx) {
    currentIndex.value = Math.max(0, Math.min(pointCount.value - 1, Math.floor(idx)))
  }

  function setCurrentAxis(axis) {
    currentAxis.value = { ...currentAxis.value, ...axis }
  }

  function setPlaying(playing) {
    isPlaying.value = playing
  }

  function setPlaySpeed(speed) {
    playSpeed.value = Math.max(0.1, Math.min(100, speed))
  }

  function updateViewOptions(options) {
    viewOptions.value = { ...viewOptions.value, ...options }
  }

  function updateMachineConfig(config) {
    machineConfig.value = { ...machineConfig.value, ...config }
  }

  function updateToolParams(params) {
    toolParams.value = { ...toolParams.value, ...params }
  }

  function setProcessing(status) {
    processingStatus.value = { ...processingStatus.value, ...status }
  }

  function reset() {
    pointCount.value = 0
    currentIndex.value = 0
    isPlaying.value = false
    parseMetadata.value = null
    ikMetadata.value = null
    sharedBuffers.value = {
      cartesianSAB: null,
      machineSAB: null,
      fieldOffsets: null,
    }
    currentAxis.value = { x: 0, y: 0, z: 0, a: 0, b: 0, c: 0, feedrate: 0, spindle: 0 }
    processingStatus.value = { parsing: false, solving: false, error: null }
  }

  return {
    gcodeFile,
    parseMetadata,
    ikMetadata,
    pointCount,
    currentIndex,
    currentProgress,
    isPlaying,
    playSpeed,
    playbackMode,
    machineConfig,
    toolParams,
    currentAxis,
    viewOptions,
    sharedBuffers,
    processingStatus,
    setGCodeFile,
    setParseResult,
    setIKResult,
    setSharedBuffers,
    setCurrentIndex,
    setCurrentAxis,
    setPlaying,
    setPlaySpeed,
    updateViewOptions,
    updateMachineConfig,
    updateToolParams,
    setProcessing,
    reset,
  }
})
