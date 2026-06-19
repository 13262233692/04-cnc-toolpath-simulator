const { app, BrowserWindow, ipcMain, dialog } = require('electron')
const path = require('path')
const fs = require('fs')

const isDev = process.env.NODE_ENV === 'development'

let mainWindow = null
let cncNative = null

function loadNativeModule() {
  try {
    const possiblePaths = [
      path.join(__dirname, '..', 'rust-core'),
      path.join(__dirname, '..', 'rust-native'),
      path.join(process.resourcesPath || '', 'rust-native'),
      process.cwd(),
    ]

    for (const basePath of possiblePaths) {
      const candidates = [
        path.join(basePath, 'cnc_native.win32-x64-msvc.node'),
        path.join(basePath, 'cnc_native.node'),
      ]
      for (const p of candidates) {
        if (fs.existsSync(p)) {
          cncNative = require(p)
          console.log('[CNC] 原生模块加载成功:', p)
          return true
        }
      }
    }
    console.warn('[CNC] 未找到原生模块，将使用JS fallback模式')
    return false
  } catch (e) {
    console.error('[CNC] 原生模块加载失败:', e.message)
    return false
  }
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1920,
    height: 1080,
    minWidth: 1280,
    minHeight: 720,
    backgroundColor: '#0a0e14',
    title: '五轴数控机床仿真监控系统',
    show: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
      webSecurity: !isDev,
      backgroundThrottling: false,
    },
  })

  mainWindow.once('ready-to-show', () => {
    mainWindow.show()
    if (isDev) {
      mainWindow.webContents.openDevTools({ mode: 'detach' })
    }
  })

  if (isDev) {
    mainWindow.loadURL('http://localhost:5173')
  } else {
    mainWindow.loadFile(path.join(__dirname, '..', 'dist', 'index.html'))
  }

  mainWindow.on('closed', () => {
    mainWindow = null
  })
}

ipcMain.handle('cnc:version', async () => {
  if (cncNative) {
    return cncNative.get_version()
  }
  return '1.0.0-fallback'
})

ipcMain.handle('cnc:parseGCode', async (_, textOrPath, isFile = false) => {
  if (!cncNative) return { ok: false, error: 'Native module not loaded' }
  try {
    const simulator = new cncNative.CncSimulator()
    const result = isFile
      ? simulator.parse_gcode_mmap(textOrPath)
      : simulator.parse_gcode_sync(textOrPath)
    return { ok: true, data: result }
  } catch (e) {
    return { ok: false, error: e.message }
  }
})

ipcMain.handle('cnc:inverseKinematics', async (_, cartesianPoints, machineConfig, toolParams, options) => {
  if (!cncNative) return { ok: false, error: 'Native module not loaded' }
  try {
    const simulator = new cncNative.CncSimulator(machineConfig || undefined)
    if (toolParams) {
      simulator.set_tool_parameters(toolParams)
    }
    const result = simulator.inverse_kinematics_sync(cartesianPoints, options || undefined)
    return { ok: true, data: result }
  } catch (e) {
    return { ok: false, error: e.message }
  }
})

ipcMain.handle('cnc:processPipeline', async (_, payload) => {
  const { gcodeText, filePath, useMmap, machineConfig, toolParams, options } = payload || {}
  if (!cncNative) return { ok: false, error: 'Native module not loaded' }
  try {
    const simulator = new cncNative.CncSimulator(machineConfig || undefined)
    if (toolParams) {
      simulator.set_tool_parameters(toolParams)
    }
    const result = useMmap && filePath
      ? simulator.process_pipeline_shared_mmap(filePath, options || undefined)
      : simulator.process_pipeline_shared(gcodeText || '', options || undefined)
    return { ok: true, data: result }
  } catch (e) {
    return { ok: false, error: e.message }
  }
})

ipcMain.handle('cnc:getFieldOffsets', async () => {
  if (!cncNative) {
    return {
      ok: true,
      data: {
        cartesian: { x_offset: 0, y_offset: 8, z_offset: 16, a_offset: 24, b_offset: 32, c_offset: 40, feedrate_offset: 48, spindle_offset: 56, stride: 64 },
        machine: { x_offset: 0, y_offset: 8, z_offset: 16, a_offset: 24, b_offset: 32, c_offset: 40, feedrate_offset: 48, valid_offset: 56, stride: 64 },
      }
    }
  }
  try {
    return {
      ok: true,
      data: {
        cartesian: cncNative.get_cartesian_field_offsets(),
        machine: cncNative.get_machine_field_offsets(),
      }
    }
  } catch (e) {
    return { ok: false, error: e.message }
  }
})

ipcMain.handle('app:openFileDialog', async (_, options = {}) => {
  try {
    const result = await dialog.showOpenDialog(mainWindow, {
      title: options.title || '选择文件',
      filters: options.filters || [
        { name: 'G-code 文件', extensions: ['nc', 'gcode', 'tap', 'cnc', 'txt'] },
        { name: 'STL 模型', extensions: ['stl'] },
        { name: '所有文件', extensions: ['*'] },
      ],
      properties: options.properties || ['openFile'],
    })
    if (result.canceled || !result.filePaths.length) {
      return { ok: false, canceled: true }
    }
    const filePath = result.filePaths[0]
    return { ok: true, filePath }
  } catch (e) {
    return { ok: false, error: e.message }
  }
})

ipcMain.handle('app:readFile', async (_, filePath, encoding = 'utf-8') => {
  try {
    const content = fs.readFileSync(filePath, encoding)
    const stat = fs.statSync(filePath)
    return { ok: true, content, size: stat.size, filePath }
  } catch (e) {
    return { ok: false, error: e.message }
  }
})

ipcMain.handle('app:readBinaryFile', async (_, filePath) => {
  try {
    const buffer = fs.readFileSync(filePath)
    const stat = fs.statSync(filePath)
    return { ok: true, buffer, size: stat.size, filePath }
  } catch (e) {
    return { ok: false, error: e.message }
  }
})

app.whenReady().then(() => {
  loadNativeModule()
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

process.on('uncaughtException', (e) => {
  console.error('未捕获异常:', e)
})

process.on('unhandledRejection', (r) => {
  console.error('未处理Promise拒绝:', r)
})
