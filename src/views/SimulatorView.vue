<template>
  <div class="simulator-view">
    <TopToolbar ref="toolbarRef" @camera="onCameraPreset" />
    <div class="main-layout">
      <LeftSidebar />
      <div class="viewport-container">
        <Viewport3D ref="viewportRef" @loadGCode="onLoadGCodeFromViewport" />
        <PlaybackControls />
      </div>
      <RightSidebar />
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import TopToolbar from '../components/TopToolbar.vue'
import LeftSidebar from '../components/LeftSidebar.vue'
import RightSidebar from '../components/RightSidebar.vue'
import Viewport3D from '../components/Viewport3D.vue'
import PlaybackControls from '../components/PlaybackControls.vue'
import { useSimulatorStore } from '../stores/simulator'

const store = useSimulatorStore()
const viewportRef = ref(null)
const toolbarRef = ref(null)

function onCameraPreset(preset) {
  viewportRef.value?.setCameraPreset(preset)
}

function onLoadGCodeFromViewport() {
  toolbarRef.value?.onLoadGCode?.()
}

onMounted(async () => {
  if (window.cncAPI) {
    const offsets = await window.cncAPI.getFieldOffsets()
    if (offsets?.ok) {
      store.setSharedBuffers({ fieldOffsets: offsets.data })
    }
  }
})

onUnmounted(() => {
  store.hardCleanup()
})
</script>

<style scoped>
.simulator-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.main-layout {
  flex: 1;
  display: flex;
  min-height: 0;
}

.viewport-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  background: var(--bg-secondary);
}
</style>
