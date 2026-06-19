import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'

export class CNCScene {
  constructor(container) {
    this.container = container
    this.scene = null
    this.camera = null
    this.renderer = null
    this.controls = null
    this.clock = new THREE.Clock()
    this.animationId = null
    this.onFrameCallback = null

    this.machineGroup = new THREE.Group()
    this.toolpathGroup = new THREE.Group()
    this.trailGroup = new THREE.Group()
    this.toolGroup = new THREE.Group()
    this.workpieceGroup = new THREE.Group()
    this.helpersGroup = new THREE.Group()

    this.toolpathLine = null
    this.trailLine = null
    this.toolMesh = null
    this.toolHolderMesh = null
    this.currentToolIndex = 0
    this.totalToolPoints = 0

    this.fieldOffsets = null
    this.sharedSAB = null
    this.cartesianPositions = null
    this.feedrateValues = null
    this.maxFeedrate = 5000

    this.isInitialized = false
  }

  init() {
    this._createScene()
    this._createCamera()
    this._createRenderer()
    this._createControls()
    this._createLights()
    this._createHelpers()
    this._createMachineBase()
    this._createTool()
    this._createWorkpiece()

    this.scene.add(this.machineGroup)
    this.scene.add(this.toolpathGroup)
    this.scene.add(this.trailGroup)
    this.scene.add(this.toolGroup)
    this.scene.add(this.workpieceGroup)
    this.scene.add(this.helpersGroup)

    this._animate()
    this._onResize()
    window.addEventListener('resize', this._onResize)
    this.isInitialized = true
  }

  _createScene() {
    this.scene = new THREE.Scene()
    this.scene.background = new THREE.Color(0x0a0e14)
    this.scene.fog = new THREE.Fog(0x0a0e14, 2000, 8000)
  }

  _createCamera() {
    const { clientWidth, clientHeight } = this.container
    this.camera = new THREE.PerspectiveCamera(45, clientWidth / clientHeight, 0.1, 20000)
    this.camera.position.set(800, 800, 800)
    this.camera.lookAt(0, 200, 0)
  }

  _createRenderer() {
    const { clientWidth, clientHeight } = this.container
    this.renderer = new THREE.WebGLRenderer({
      antialias: true,
      alpha: false,
      powerPreference: 'high-performance',
    })
    this.renderer.setSize(clientWidth, clientHeight)
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
    this.renderer.shadowMap.enabled = true
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap
    this.renderer.outputColorSpace = THREE.SRGBColorSpace
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping
    this.renderer.toneMappingExposure = 1.2
    this.container.appendChild(this.renderer.domElement)
  }

  _createControls() {
    this.controls = new OrbitControls(this.camera, this.renderer.domElement)
    this.controls.enableDamping = true
    this.controls.dampingFactor = 0.08
    this.controls.target.set(0, 200, 0)
    this.controls.minDistance = 50
    this.controls.maxDistance = 8000
    this.controls.maxPolarAngle = Math.PI * 0.49
  }

  _createLights() {
    const ambient = new THREE.AmbientLight(0x404858, 0.6)
    this.scene.add(ambient)

    const dirLight = new THREE.DirectionalLight(0xffffff, 1.0)
    dirLight.position.set(800, 1200, 600)
    dirLight.castShadow = true
    dirLight.shadow.mapSize.width = 2048
    dirLight.shadow.mapSize.height = 2048
    dirLight.shadow.camera.near = 1
    dirLight.shadow.camera.far = 4000
    dirLight.shadow.camera.left = -1500
    dirLight.shadow.camera.right = 1500
    dirLight.shadow.camera.top = 1500
    dirLight.shadow.camera.bottom = -1500
    this.scene.add(dirLight)

    const fillLight = new THREE.DirectionalLight(0x88aaff, 0.3)
    fillLight.position.set(-600, 400, -400)
    this.scene.add(fillLight)

    const rimLight = new THREE.DirectionalLight(0xffaa88, 0.2)
    rimLight.position.set(0, 800, -800)
    this.scene.add(rimLight)
  }

  _createHelpers() {
    const gridSize = 2000
    const gridDivisions = 40
    const grid = new THREE.GridHelper(gridSize, gridDivisions, 0x1e2a3d, 0x161d2b)
    grid.position.y = -0.01
    this.helpersGroup.add(grid)

    const axes = new THREE.AxesHelper(300)
    this.helpersGroup.add(axes)

    const planeGeo = new THREE.PlaneGeometry(gridSize, gridSize)
    const planeMat = new THREE.MeshStandardMaterial({
      color: 0x141a24,
      roughness: 0.9,
      metalness: 0.1,
      transparent: true,
      opacity: 0.8,
    })
    const plane = new THREE.Mesh(planeGeo, planeMat)
    plane.rotation.x = -Math.PI / 2
    plane.position.y = -0.02
    plane.receiveShadow = true
    this.helpersGroup.add(plane)
  }

  _createMachineBase() {
    const baseMaterial = new THREE.MeshStandardMaterial({
      color: 0x3a4556,
      roughness: 0.6,
      metalness: 0.5,
    })

    const baseGeo = new THREE.BoxGeometry(1400, 120, 1200)
    const base = new THREE.Mesh(baseGeo, baseMaterial)
    base.position.y = 60
    base.castShadow = true
    base.receiveShadow = true
    this.machineGroup.add(base)

    const columnMat = new THREE.MeshStandardMaterial({
      color: 0x4a5566,
      roughness: 0.5,
      metalness: 0.6,
    })

    const columnGeo = new THREE.BoxGeometry(200, 700, 250)
    const column = new THREE.Mesh(columnGeo, columnMat)
    column.position.set(450, 470, 0)
    column.castShadow = true
    this.machineGroup.add(column)

    const spindleMat = new THREE.MeshStandardMaterial({
      color: 0x2a3440,
      roughness: 0.4,
      metalness: 0.7,
    })

    const spindleGeo = new THREE.BoxGeometry(180, 200, 180)
    this.spindleHead = new THREE.Mesh(spindleGeo, spindleMat)
    this.spindleHead.position.set(250, 700, 0)
    this.spindleHead.castShadow = true
    this.machineGroup.add(this.spindleHead)

    const rotaryMat = new THREE.MeshStandardMaterial({
      color: 0x556677,
      roughness: 0.5,
      metalness: 0.4,
    })

    const rotaryGeo = new THREE.CylinderGeometry(350, 380, 100, 64)
    this.rotaryTable = new THREE.Mesh(rotaryGeo, rotaryMat)
    this.rotaryTable.position.set(-100, 170, 0)
    this.rotaryTable.castShadow = true
    this.rotaryTable.receiveShadow = true
    this.machineGroup.add(this.rotaryTable)

    const tableMat = new THREE.MeshStandardMaterial({
      color: 0x667788,
      roughness: 0.3,
      metalness: 0.5,
    })
    const tableGeo = new THREE.BoxGeometry(600, 30, 600)
    this.rotaryTableTop = new THREE.Mesh(tableGeo, tableMat)
    this.rotaryTableTop.position.y = 65
    this.rotaryTableTop.castShadow = true
    this.rotaryTableTop.receiveShadow = true
    this.rotaryTable.add(this.rotaryTableTop)
  }

  _createTool() {
    const holderMat = new THREE.MeshStandardMaterial({
      color: 0x8899aa,
      roughness: 0.3,
      metalness: 0.8,
    })

    const holderGeo = new THREE.CylinderGeometry(25, 35, 80, 32)
    this.toolHolderMesh = new THREE.Mesh(holderGeo, holderMat)
    this.toolHolderMesh.position.y = -40
    this.toolHolderMesh.castShadow = true
    this.toolGroup.add(this.toolHolderMesh)

    const toolMat = new THREE.MeshStandardMaterial({
      color: 0xeeeeff,
      roughness: 0.2,
      metalness: 0.9,
      emissive: 0x222233,
    })

    const toolLen = 100
    const toolR = 5
    const toolGeo = new THREE.CylinderGeometry(toolR, toolR * 0.3, toolLen, 24)
    this.toolMesh = new THREE.Mesh(toolGeo, toolMat)
    this.toolMesh.position.y = -toolLen / 2 - 80
    this.toolMesh.castShadow = true
    this.toolGroup.add(this.toolMesh)

    this.toolGroup.position.set(0, 700, 0)
  }

  _createWorkpiece() {
    const wpMat = new THREE.MeshStandardMaterial({
      color: 0x6688aa,
      roughness: 0.7,
      metalness: 0.2,
      transparent: true,
      opacity: 0.6,
    })

    const wpGeo = new THREE.BoxGeometry(300, 150, 300)
    const wp = new THREE.Mesh(wpGeo, wpMat)
    wp.position.y = 245
    wp.castShadow = true
    wp.receiveShadow = true
    this.workpieceGroup.add(wp)
  }

  loadToolpathFromShared(sab, count, offsets, colorMode = 'feedrate') {
    this._clearToolpath()
    this._clearTrail()
    this.sharedSAB = sab
    this.fieldOffsets = offsets
    this.totalToolPoints = count

    const result = window.bufferUtils.extractPointWithFeed(sab, count, offsets)
    this.cartesianPositions = result.positions
    this.feedrateValues = result.feeds

    const positions = new Float32Array(count * 3)
    const colors = new Float32Array(count * 3)

    let maxFeed = 0
    for (let i = 0; i < count; i++) {
      positions[i * 3] = this.cartesianPositions[i * 3]
      positions[i * 3 + 1] = this.cartesianPositions[i * 3 + 1]
      positions[i * 3 + 2] = this.cartesianPositions[i * 3 + 2]
      if (this.feedrateValues[i] > maxFeed) maxFeed = this.feedrateValues[i]
    }

    this.maxFeedrate = maxFeed || 5000

    for (let i = 0; i < count; i++) {
      let color
      if (colorMode === 'feedrate') {
        const t = this.feedrateValues[i] / this.maxFeedrate
        color = this._heatmapColor(t)
      } else if (colorMode === 'axis') {
        const normY = (positions[i * 3 + 1] + 500) / 1000
        color = new THREE.Color().setHSL(0.66 * (1 - Math.max(0, Math.min(1, normY))), 0.8, 0.5)
      } else {
        color = new THREE.Color(0x06b6d4)
      }
      colors[i * 3] = color.r
      colors[i * 3 + 1] = color.g
      colors[i * 3 + 2] = color.b
    }

    const geometry = new THREE.BufferGeometry()
    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3))
    geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3))

    const material = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.85,
      linewidth: 1,
    })

    this.toolpathLine = new THREE.Line(geometry, material)
    this.toolpathGroup.add(this.toolpathLine)
    this._fitCameraToToolpath(positions, count)
  }

  _heatmapColor(t) {
    t = Math.max(0, Math.min(1, t))
    const r = Math.min(1, t * 2)
    const g = Math.min(1, 2 - t * 2)
    const b = t < 0.5 ? 1 - t * 2 : 0
    return new THREE.Color(r, g, b)
  }

  _fitCameraToToolpath(positions, count) {
    let minX = Infinity, maxX = -Infinity
    let minY = Infinity, maxY = -Infinity
    let minZ = Infinity, maxZ = -Infinity
    const step = Math.max(1, Math.floor(count / 10000))
    for (let i = 0; i < count; i += step) {
      const x = positions[i * 3], y = positions[i * 3 + 1], z = positions[i * 3 + 2]
      if (x < minX) minX = x
      if (x > maxX) maxX = x
      if (y < minY) minY = y
      if (y > maxY) maxY = y
      if (z < minZ) minZ = z
      if (z > maxZ) maxZ = z
    }
    if (!isFinite(minX)) return
    const centerX = (minX + maxX) / 2
    const centerY = (minY + maxY) / 2
    const centerZ = (minZ + maxZ) / 2
    const size = Math.max(maxX - minX, maxY - minY, maxZ - minZ)
    const dist = size * 1.8

    const target = new THREE.Vector3(centerX, centerY + size * 0.2, centerZ)
    this.controls.target.copy(target)
    this.camera.position.set(
      target.x + dist * 0.7,
      target.y + dist * 0.7,
      target.z + dist * 0.7
    )
    this.camera.near = size / 1000
    this.camera.far = size * 100
    this.camera.updateProjectionMatrix()
  }

  setToolIndex(index, trailLength = 500) {
    if (!this.cartesianPositions || this.totalToolPoints === 0) return
    const idx = Math.max(0, Math.min(this.totalToolPoints - 1, Math.floor(index)))
    this.currentToolIndex = idx

    const base = idx * 3
    const x = this.cartesianPositions[base]
    const y = this.cartesianPositions[base + 1]
    const z = this.cartesianPositions[base + 2]

    this.toolGroup.position.set(x, y + 700, z)
    this._updateTrail(idx, trailLength)
    this._updateMachinePose(x, y, z, idx)
  }

  _updateTrail(endIdx, trailLength) {
    this._clearTrail()
    if (trailLength <= 0) return

    const startIdx = Math.max(0, endIdx - trailLength)
    const count = endIdx - startIdx + 1
    if (count < 2) return

    const positions = new Float32Array(count * 3)
    const colors = new Float32Array(count * 3)

    for (let i = 0; i < count; i++) {
      const srcIdx = (startIdx + i) * 3
      positions[i * 3] = this.cartesianPositions[srcIdx]
      positions[i * 3 + 1] = this.cartesianPositions[srcIdx + 1]
      positions[i * 3 + 2] = this.cartesianPositions[srcIdx + 2]

      const t = i / (count - 1)
      const color = this._heatmapColor(t)
      colors[i * 3] = color.r
      colors[i * 3 + 1] = color.g
      colors[i * 3 + 2] = color.b
    }

    const geo = new THREE.BufferGeometry()
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3))
    geo.setAttribute('color', new THREE.BufferAttribute(colors, 3))

    const mat = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.95,
    })

    this.trailLine = new THREE.Line(geo, mat)
    this.trailGroup.add(this.trailLine)
  }

  _updateMachinePose(x, y, z, idx) {
    if (this.sharedSAB && this.fieldOffsets) {
      const off = this.fieldOffsets.machine
      if (off) {
        const dv = new DataView(this.sharedSAB)
        const stride = off.stride
        const a = dv.getFloat64(off.a_offset + idx * stride, true)
        const b = dv.getFloat64(off.b_offset + idx * stride, true)
        if (this.rotaryTable) {
          this.rotaryTable.rotation.x = a * Math.PI / 180
          this.rotaryTable.rotation.y = b * Math.PI / 180
        }
      }
    }
  }

  _clearToolpath() {
    if (this.toolpathLine) {
      this.toolpathLine.geometry.dispose()
      this.toolpathLine.material.dispose()
      this.toolpathGroup.remove(this.toolpathLine)
      this.toolpathLine = null
    }
  }

  _clearTrail() {
    if (this.trailLine) {
      this.trailLine.geometry.dispose()
      this.trailLine.material.dispose()
      this.trailGroup.remove(this.trailLine)
      this.trailLine = null
    }
  }

  setToolLength(length) {
    if (!this.toolMesh) return
    const toolR = 5
    this.toolMesh.geometry.dispose()
    this.toolMesh.geometry = new THREE.CylinderGeometry(toolR, toolR * 0.3, length, 24)
    this.toolMesh.position.y = -length / 2 - 80
  }

  setCameraPreset(preset) {
    const target = this.controls.target.clone()
    let dist = this.camera.position.distanceTo(target)
    if (dist < 100) dist = 1000

    switch (preset) {
      case 'top':
        this.camera.position.set(target.x, target.y + dist, target.z)
        break
      case 'front':
        this.camera.position.set(target.x, target.y, target.z + dist)
        break
      case 'side':
        this.camera.position.set(target.x + dist, target.y, target.z)
        break
      case 'iso':
      default:
        this.camera.position.set(
          target.x + dist * 0.7,
          target.y + dist * 0.7,
          target.z + dist * 0.7
        )
    }
    this.camera.lookAt(target)
  }

  getRendererInfo() {
    if (!this.renderer) return null
    return {
      triangles: this.renderer.info.render.triangles,
      drawCalls: this.renderer.info.render.calls,
      memory: this.renderer.info.memory,
    }
  }

  setOnFrameCallback(cb) {
    this.onFrameCallback = cb
  }

  _animate = () => {
    this.animationId = requestAnimationFrame(this._animate)
    const dt = this.clock.getDelta()
    this.controls.update()
    if (this.onFrameCallback) {
      this.onFrameCallback(dt, this.clock.elapsedTime)
    }
    this.renderer.render(this.scene, this.camera)
  }

  _onResize = () => {
    if (!this.container || !this.camera || !this.renderer) return
    const { clientWidth, clientHeight } = this.container
    this.camera.aspect = clientWidth / clientHeight
    this.camera.updateProjectionMatrix()
    this.renderer.setSize(clientWidth, clientHeight)
  }

  getCurrentToolPosition() {
    if (!this.cartesianPositions || this.totalToolPoints === 0) {
      return { x: 0, y: 0, z: 0, feedrate: 0 }
    }
    const base = this.currentToolIndex * 3
    return {
      x: this.cartesianPositions[base],
      y: this.cartesianPositions[base + 1],
      z: this.cartesianPositions[base + 2],
      feedrate: this.feedrateValues?.[this.currentToolIndex] || 0,
      index: this.currentToolIndex,
    }
  }

  dispose() {
    if (this.animationId) {
      cancelAnimationFrame(this.animationId)
    }
    window.removeEventListener('resize', this._onResize)
    this._clearToolpath()
    this._clearTrail()
    if (this.renderer) {
      this.renderer.dispose()
      if (this.renderer.domElement?.parentNode) {
        this.renderer.domElement.parentNode.removeChild(this.renderer.domElement)
      }
    }
    this.scene?.traverse((obj) => {
      if (obj.geometry) obj.geometry.dispose?.()
      if (obj.material) {
        if (Array.isArray(obj.material)) {
          obj.material.forEach((m) => m.dispose?.())
        } else {
          obj.material.dispose?.()
        }
      }
    })
  }
}
