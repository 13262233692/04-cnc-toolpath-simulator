const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('cncAPI', {
  getVersion: () => ipcRenderer.invoke('cnc:version'),

  parseGCode: (text) => ipcRenderer.invoke('cnc:parseGCode', text, false),
  parseGCodeFile: (filePath) => ipcRenderer.invoke('cnc:parseGCode', filePath, true),

  inverseKinematics: (points, machineConfig, toolParams, options) =>
    ipcRenderer.invoke('cnc:inverseKinematics', points, machineConfig, toolParams, options),

  processPipeline: (payload) => ipcRenderer.invoke('cnc:processPipeline', payload),

  getFieldOffsets: () => ipcRenderer.invoke('cnc:getFieldOffsets'),
})

contextBridge.exposeInMainWorld('fileAPI', {
  openFileDialog: (options) => ipcRenderer.invoke('app:openFileDialog', options),
  readFile: (filePath, encoding) => ipcRenderer.invoke('app:readFile', filePath, encoding),
  readBinaryFile: (filePath) => ipcRenderer.invoke('app:readBinaryFile', filePath),
})

contextBridge.exposeInMainWorld('bufferUtils', {
  sharedToArrayF64: (sab, byteOffset, length, stride) => {
    const result = new Array(length)
    const dv = new DataView(sab)
    for (let i = 0; i < length; i++) {
      result[i] = dv.getFloat64(byteOffset + i * stride, true)
    }
    return result
  },

  extractFieldColumn: (sab, count, fieldOffset, stride) => {
    const result = new Float64Array(count)
    const dv = new DataView(sab)
    for (let i = 0; i < count; i++) {
      result[i] = dv.getFloat64(fieldOffset + i * stride, true)
    }
    return result
  },

  extractPointXYZ: (sab, count, offsets) => {
    const positions = new Float32Array(count * 3)
    const dv = new DataView(sab)
    const { x_offset, y_offset, z_offset, stride } = offsets
    for (let i = 0; i < count; i++) {
      const base = i * stride
      positions[i * 3] = dv.getFloat64(base + x_offset, true)
      positions[i * 3 + 1] = dv.getFloat64(base + y_offset, true)
      positions[i * 3 + 2] = dv.getFloat64(base + z_offset, true)
    }
    return positions
  },

  extractPointWithFeed: (sab, count, offsets) => {
    const positions = new Float32Array(count * 3)
    const feeds = new Float32Array(count)
    const dv = new DataView(sab)
    const { x_offset, y_offset, z_offset, feedrate_offset, stride } = offsets
    for (let i = 0; i < count; i++) {
      const base = i * stride
      positions[i * 3] = dv.getFloat64(base + x_offset, true)
      positions[i * 3 + 1] = dv.getFloat64(base + y_offset, true)
      positions[i * 3 + 2] = dv.getFloat64(base + z_offset, true)
      feeds[i] = dv.getFloat64(base + feedrate_offset, true)
    }
    return { positions, feeds }
  },

  extractMachineAxis: (sab, count, offsets, axis) => {
    const axisOffsets = {
      x: offsets.x_offset,
      y: offsets.y_offset,
      z: offsets.z_offset,
      a: offsets.a_offset,
      b: offsets.b_offset,
      c: offsets.c_offset,
    }
    const fieldOffset = axisOffsets[axis] || 0
    const result = new Float64Array(count)
    const dv = new DataView(sab)
    for (let i = 0; i < count; i++) {
      result[i] = dv.getFloat64(fieldOffset + i * offsets.stride, true)
    }
    return result
  },
})
