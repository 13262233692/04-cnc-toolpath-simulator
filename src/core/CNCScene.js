import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'
import { VoxelEngine, Aabb } from './VoxelEngine.js'

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
    this.toolpathGeometry = null
    this.toolpathPositionsAttr = null
    this.toolpathColorsAttr = null
    this.toolpathMaterial = null

    this.trailLine = null
    this.trailGeometry = null
    this.trailPositionsAttr = null
    this.trailColorsAttr = null
    this.trailMaterial = null
    this.trailMaxPoints = 2000
    this.trailCurrentCount = 0

    this.toolMesh = null
    this.toolHolderMesh = null
    this.currentToolIndex = 0
    this.totalToolPoints = 0

    this.fieldOffsets = null
    this.sharedSAB = null
    this.cartesianPositions = null
    this.feedrateValues = null
    this.maxFeedrate = 5000

    this._disposed = false
    this.isInitialized = false
    this._renderInfo = { geometries: 0, materials: 0, textures: 0 }

    this.voxelEngine = null
    this.voxelMesh = null
    this.voxelGeometry = null
    this.voxelMaterial = null
    this.voxelInstanceCount = 0
    this.voxelMaxInstances = 50000
    this.voxelEnabled = true
    this.voxelSize = 2.5

    this.collisionAlarm = {
      active: false,
      fixtureCollision: false,
      workpieceCollision: false,
      flashPhase: 0,
      flashSpeed: 8,
    }
    this.onCollisionCallback = null

    this.lastCarvedIndex = -1
    this.cutStep = 1
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
    this.workpieceBaseMesh = wp
  }

  _createVoxelEngine(bounds) {
    if (!this.voxelEnabled) return

    const toolR = 5
    const toolLen = 100

    this.voxelEngine = new VoxelEngine(
      bounds,
      this.voxelSize,
      toolR,
      toolLen,
      0.5
    )
    this.voxelEngine.setHolder(80, 35)

    const tableTopY = 170 + 65
    const fixture1 = new Aabb(
      bounds.minX - 10, tableTopY, bounds.minZ - 10,
      bounds.minX + 20, tableTopY + 40, bounds.maxZ + 10
    )
    const fixture2 = new Aabb(
      bounds.maxX - 20, tableTopY, bounds.minZ - 10,
      bounds.maxX + 10, tableTopY + 40, bounds.maxZ + 10
    )
    this.voxelEngine.addFixture(fixture1)
    this.voxelEngine.addFixture(fixture2)

    this._createVoxelMesh()
  }

  _createVoxelMesh() {
    if (!this.voxelEngine) return
    this._clearVoxelMesh()

    const vs = this.voxelSize
    const geo = new THREE.BoxGeometry(vs * 0.95, vs * 0.95, vs * 0.95)

    const mat = new THREE.MeshStandardMaterial({
      color: 0x88aabb,
      roughness: 0.6,
      metalness: 0.25,
      flatShading: true,
    })

    const mesh = new THREE.InstancedMesh(geo, mat, this.voxelMaxInstances)
    mesh.castShadow = true
    mesh.receiveShadow = true
    mesh.count = 0

    const dummy = new THREE.Object3D()
    for (let i = 0; i < this.voxelMaxInstances; i++) {
      mesh.setMatrixAt(i, dummy.matrix)
    }
    mesh.instanceMatrix.needsUpdate = false

    this.voxelMesh = mesh
    this.voxelGeometry = geo
    this.voxelMaterial = mat
    this.workpieceGroup.add(mesh)

    this._updateVoxelMesh()
  }

  _updateVoxelMesh() {
    if (!this.voxelEngine || !this.voxelMesh) return

    const points = this.voxelEngine.extractSurfacePoints(this.voxelMaxInstances)
    const count = Math.floor(points.length / 3)
    const dummy = new THREE.Object3D()

    for (let i = 0; i < count; i++) {
      const x = points[i * 3]
      const y = points[i * 3 + 1]
      const z = points[i * 3 + 2]
      dummy.position.set(x, y, z)
      dummy.updateMatrix()
      this.voxelMesh.setMatrixAt(i, dummy.matrix)
    }

    this.voxelMesh.count = count
    this.voxelMesh.instanceMatrix.needsUpdate = true
    this.voxelInstanceCount = count
  }

  _clearVoxelMesh() {
    if (this.voxelMesh) {
      this.workpieceGroup.remove(this.voxelMesh)
      this._disposeGeometry(this.voxelMesh.geometry)
      this._disposeMaterial(this.voxelMesh.material)
      this.voxelMesh = null
    }
    if (this.voxelGeometry) {
      this._disposeGeometry(this.voxelGeometry)
      this.voxelGeometry = null
    }
    if (this.voxelMaterial) {
      this._disposeMaterial(this.voxelMaterial)
      this.voxelMaterial = null
    }
    this.voxelInstanceCount = 0
  }

  resetVoxelWorkpiece() {
    if (this.voxelEngine) {
      this.voxelEngine.resetWorkpiece()
      this._updateVoxelMesh()
    }
    this.lastCarvedIndex = -1
    this.collisionAlarm.active = false
    this.collisionAlarm.fixtureCollision = false
    this.collisionAlarm.workpieceCollision = false
  }

  setVoxelEnabled(enabled) {
    this.voxelEnabled = enabled
    if (!enabled) {
      this._clearVoxelMesh()
      this.voxelEngine = null
    } else if (this.cartesianPositions && this.totalToolPoints > 0) {
      const bounds = this._computeToolpathBounds()
      this._createVoxelEngine(bounds)
    }
  }

  setVoxelSize(size) {
    this.voxelSize = Math.max(0.5, size)
    if (this.voxelEngine && this.cartesianPositions && this.totalToolPoints > 0) {
      const bounds = this._computeToolpathBounds()
      this._createVoxelEngine(bounds)
    }
  }

  _computeToolpathBounds() {
    if (!this.cartesianPositions || this.totalToolPoints === 0) {
      return new Aabb(-150, 170, -150, 150, 320, 150)
    }
    let minX = Infinity, minY = Infinity, minZ = Infinity
    let maxX = -Infinity, maxY = -Infinity, maxZ = -Infinity
    const step = Math.max(1, Math.floor(this.totalToolPoints / 2000))
    for (let i = 0; i < this.totalToolPoints; i += step) {
      const x = this.cartesianPositions[i * 3]
      const y = this.cartesianPositions[i * 3 + 1]
      const z = this.cartesianPositions[i * 3 + 2]
      if (x < minX) minX = x
      if (x > maxX) maxX = x
      if (y < minY) minY = y
      if (y > maxY) maxY = y
      if (z < minZ) minZ = z
      if (z > maxZ) maxZ = z
    }
    const toolR = 10
    const pad = toolR * 2
    minX -= pad
    minY -= pad
    minZ -= pad
    maxX += pad
    maxY += pad
    maxZ += pad
    const tableTopY = 170 + 65
    minY = Math.min(minY, tableTopY + 5)
    maxY = Math.max(maxY, tableTopY + 30)
    return new Aabb(minX, minY, minZ, maxX, maxY, maxZ)
  }

  loadToolpathFromShared(sab, count, offsets, colorMode = 'feedrate') {
    this._clearToolpath()
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

    this.toolpathGeometry = new THREE.BufferGeometry()
    this.toolpathPositionsAttr = new THREE.BufferAttribute(positions, 3)
    this.toolpathColorsAttr = new THREE.BufferAttribute(colors, 3)
    this.toolpathPositionsAttr.setUsage(THREE.StaticDrawUsage)
    this.toolpathColorsAttr.setUsage(THREE.StaticDrawUsage)
    this.toolpathGeometry.setAttribute('position', this.toolpathPositionsAttr)
    this.toolpathGeometry.setAttribute('color', this.toolpathColorsAttr)
    this.toolpathGeometry.setDrawRange(0, count)

    this.toolpathMaterial = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.85,
      linewidth: 1,
    })

    this.toolpathLine = new THREE.Line(this.toolpathGeometry, this.toolpathMaterial)
    this.toolpathGroup.add(this.toolpathLine)

    if (this.voxelEnabled) {
      const bounds = this._computeToolpathBounds()
      this._createVoxelEngine(bounds)
    }

    if (this.workpieceBaseMesh) {
      this.workpieceBaseMesh.visible = !this.voxelEnabled
    }

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
    this._updateTrailFast(idx, trailLength)
    this._updateMachinePose(x, y, z, idx)
    this._advanceVoxelCut(idx)
    this._checkCollisions(x, y, z)
  }

  _advanceVoxelCut(toIdx) {
    if (!this.voxelEngine) return
    if (toIdx <= this.lastCarvedIndex) return
    if (this.lastCarvedIndex < 0) {
      this.lastCarvedIndex = toIdx
      return
    }

    const startIdx = Math.max(0, this.lastCarvedIndex)
    const endIdx = toIdx

    let needsUpdate = false

    for (let i = startIdx; i < endIdx; i += this.cutStep) {
      const nextIdx = Math.min(i + this.cutStep, endIdx)
      const x0 = this.cartesianPositions[i * 3]
      const y0 = this.cartesianPositions[i * 3 + 1]
      const z0 = this.cartesianPositions[i * 3 + 2]
      const x1 = this.cartesianPositions[nextIdx * 3]
      const y1 = this.cartesianPositions[nextIdx * 3 + 1]
      const z1 = this.cartesianPositions[nextIdx * 3 + 2]

      const feed = this.feedrateValues?.[i] ?? 0
      if (feed < 1) continue

      const removed = this.voxelEngine.carveSegment(x0, y0, z0, x1, y1, z1, 128)
      if (removed > 0) {
        needsUpdate = true
      }
    }

    if (needsUpdate) {
      this._updateVoxelMesh()
    }
    this.lastCarvedIndex = endIdx
  }

  _checkCollisions(tipX, tipY, tipZ) {
    if (!this.voxelEngine) return

    const fixtureHit = this.voxelEngine.checkFixtureCollision(tipX, tipY, tipZ)
    const workpieceHit = this.voxelEngine.checkToolCollision(tipX, tipY, tipZ)

    const wasActive = this.collisionAlarm.active
    const wasFixture = this.collisionAlarm.fixtureCollision
    const wasWorkpiece = this.collisionAlarm.workpieceCollision
    this.collisionAlarm.fixtureCollision = fixtureHit
    this.collisionAlarm.workpieceCollision = workpieceHit
    this.collisionAlarm.active = fixtureHit || workpieceHit

    const statusChanged = wasActive !== this.collisionAlarm.active
      || wasFixture !== fixtureHit
      || wasWorkpiece !== workpieceHit

    if (statusChanged && this.onCollisionCallback) {
      this.onCollisionCallback({
        active: this.collisionAlarm.active,
        fixtureCollision: fixtureHit,
        workpieceCollision: workpieceHit,
        lastCollisionPos: this.collisionAlarm.active ? [tipX, tipY, tipZ] : null,
      })
    }
  }

  setOnCollisionCallback(cb) {
    this.onCollisionCallback = cb
  }

  _updateCollisionFlash(dt) {
    if (!this.collisionAlarm.active) {
      if (this.toolHolderMesh?.material?.emissive) {
        this.toolHolderMesh.material.emissive.setHex(0x000000)
      }
      return
    }
    this.collisionAlarm.flashPhase += dt * this.collisionAlarm.flashSpeed
    const intensity = (Math.sin(this.collisionAlarm.flashPhase) + 1) * 0.5
    if (this.toolHolderMesh?.material?.emissive) {
      this.toolHolderMesh.material.emissive.setRGB(intensity * 0.8, 0, 0)
    }
    if (this.spindleHead?.material?.emissive) {
      this.spindleHead.material.emissive.setRGB(intensity * 0.3, 0, 0)
    }
  }

  getVoxelStats() {
    if (!this.voxelEngine) {
      return null
    }
    return {
      totalVoxels: this.voxelEngine.totalVoxels,
      solidVoxels: this.voxelEngine.solidVoxels,
      removedRatio: this.voxelEngine.removedRatio,
      voxelSize: this.voxelEngine.voxelSize,
      gridSize: this.voxelEngine.gridSize,
      instanceCount: this.voxelInstanceCount,
      boundsMin: this.voxelEngine.boundsMin,
      boundsMax: this.voxelEngine.boundsMax,
    }
  }

  getCollisionStatus() {
    return {
      active: this.collisionAlarm.active,
      fixtureCollision: this.collisionAlarm.fixtureCollision,
      workpieceCollision: this.collisionAlarm.workpieceCollision,
    }
  }

  _ensureTrailGeometry(maxPoints) {
    if (this.trailLine && this.trailPositionsAttr && this.trailPositionsAttr.count >= maxPoints) {
      return
    }
    this._clearTrail()

    const trailPositions = new Float32Array(maxPoints * 3)
    const trailColors = new Float32Array(maxPoints * 3)

    this.trailGeometry = new THREE.BufferGeometry()
    this.trailPositionsAttr = new THREE.BufferAttribute(trailPositions, 3)
    this.trailColorsAttr = new THREE.BufferAttribute(trailColors, 3)
    this.trailPositionsAttr.setUsage(THREE.DynamicDrawUsage)
    this.trailColorsAttr.setUsage(THREE.DynamicDrawUsage)
    this.trailGeometry.setAttribute('position', this.trailPositionsAttr)
    this.trailGeometry.setAttribute('color', this.trailColorsAttr)
    this.trailGeometry.setDrawRange(0, 0)

    this.trailMaterial = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.95,
    })

    this.trailLine = new THREE.Line(this.trailGeometry, this.trailMaterial)
    this.trailGroup.add(this.trailLine)
    this.trailMaxPoints = maxPoints
    this.trailCurrentCount = 0
  }

  _updateTrailFast(endIdx, trailLength) {
    if (!this.cartesianPositions || this.totalToolPoints === 0) return
    if (trailLength <= 0) {
      this.trailGeometry?.setDrawRange(0, 0)
      return
    }

    const safeLen = Math.min(trailLength, this.trailMaxPoints)
    const startIdx = Math.max(0, endIdx - safeLen + 1)
    const count = endIdx - startIdx + 1
    if (count < 2) {
      this.trailGeometry?.setDrawRange(0, 0)
      this.trailCurrentCount = 0
      return
    }

    this._ensureTrailGeometry(safeLen)

    const posArr = this.trailPositionsAttr.array
    const colArr = this.trailColorsAttr.array

    for (let i = 0; i < count; i++) {
      const srcIdx = (startIdx + i) * 3
      const dstIdx = i * 3
      posArr[dstIdx] = this.cartesianPositions[srcIdx]
      posArr[dstIdx + 1] = this.cartesianPositions[srcIdx + 1]
      posArr[dstIdx + 2] = this.cartesianPositions[srcIdx + 2]

      const t = i / (count - 1)
      const color = this._heatmapColor(t)
      colArr[dstIdx] = color.r
      colArr[dstIdx + 1] = color.g
      colArr[dstIdx + 2] = color.b
    }

    this.trailPositionsAttr.needsUpdate = true
    this.trailColorsAttr.needsUpdate = true
    this.trailGeometry.setDrawRange(0, count)
    this.trailCurrentCount = count
  }

  _updateTrail(endIdx, trailLength) {
    this._updateTrailFast(endIdx, trailLength)
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

  _disposeAttribute(attr) {
    if (attr) {
      try { attr.dispose?.() } catch (e) { /* noop */ }
    }
  }

  _disposeGeometry(geo) {
    if (!geo) return
    if (geo.index) {
      this._disposeAttribute(geo.index)
    }
    geo.deleteAttribute?.('position')
    geo.deleteAttribute?.('color')
    geo.deleteAttribute?.('normal')
    geo.deleteAttribute?.('uv')
    try { geo.dispose?.() } catch (e) { /* noop */ }
  }

  _disposeMaterial(mat) {
    if (mat) {
      try { mat.dispose?.() } catch (e) { /* noop */ }
    }
  }

  _clearToolpath() {
    if (this.toolpathLine) {
      this.toolpathGroup.remove(this.toolpathLine)
      this._disposeGeometry(this.toolpathLine.geometry)
      this._disposeMaterial(this.toolpathLine.material)
      this.toolpathLine = null
    }
    this._disposeAttribute(this.toolpathPositionsAttr)
    this._disposeAttribute(this.toolpathColorsAttr)
    this._disposeGeometry(this.toolpathGeometry)
    this._disposeMaterial(this.toolpathMaterial)
    this.toolpathPositionsAttr = null
    this.toolpathColorsAttr = null
    this.toolpathGeometry = null
    this.toolpathMaterial = null
  }

  _clearTrail() {
    if (this.trailLine) {
      this.trailGroup.remove(this.trailLine)
      this._disposeGeometry(this.trailLine.geometry)
      this._disposeMaterial(this.trailLine.material)
      this.trailLine = null
    }
    this._disposeAttribute(this.trailPositionsAttr)
    this._disposeAttribute(this.trailColorsAttr)
    this._disposeGeometry(this.trailGeometry)
    this._disposeMaterial(this.trailMaterial)
    this.trailPositionsAttr = null
    this.trailColorsAttr = null
    this.trailGeometry = null
    this.trailMaterial = null
    this.trailCurrentCount = 0
  }

  setToolLength(length) {
    if (!this.toolMesh) return
    const toolR = 5
    this._disposeGeometry(this.toolMesh.geometry)
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
    this._updateCollisionFlash(dt)
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

  forceGPUResourceCleanup() {
    if (!this.renderer) return
    const gl = this.renderer.getContext()
    if (gl) {
      const ext = gl.getExtension('WEBGL_lose_context')
      if (ext) {
        try { ext.loseContext() } catch (e) { /* noop */ }
      }
    }
  }

  dispose() {
    if (this._disposed) return
    this._disposed = true

    if (this.animationId) {
      cancelAnimationFrame(this.animationId)
      this.animationId = null
    }
    window.removeEventListener('resize', this._onResize)

    this._clearToolpath()
    this._clearTrail()
    this._clearVoxelMesh()

    this.voxelEngine = null
    this.onCollisionCallback = null

    if (this.controls?.dispose) {
      try { this.controls.dispose() } catch (e) { /* noop */ }
    }
    this.controls = null

    this.scene?.traverse((obj) => {
      if (obj.isMesh || obj.isLine || obj.isPoints) {
        this._disposeGeometry(obj.geometry)
      }
      if (obj.material) {
        if (Array.isArray(obj.material)) {
          obj.material.forEach((m) => this._disposeMaterial(m))
        } else {
          this._disposeMaterial(obj.material)
          if (obj.material.map) {
            try { obj.material.map.dispose?.() } catch (e) { /* noop */ }
          }
        }
      }
    })

    this.scene?.clear?.()

    this.forceGPUResourceCleanup()

    if (this.renderer) {
      try {
        this.renderer.dispose?.()
        this.renderer.forceContextLoss?.()
      } catch (e) { /* noop */ }
      if (this.renderer.domElement?.parentNode) {
        this.renderer.domElement.parentNode.removeChild(this.renderer.domElement)
      }
      this.renderer = null
    }

    this.cartesianPositions = null
    this.feedrateValues = null
    this.sharedSAB = null
    this.fieldOffsets = null

    this.scene = null
    this.camera = null
    this.controls = null
  }
}
