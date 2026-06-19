const VOXEL_EMPTY = 0
const VOXEL_SOLID = 1

export class Aabb {
  constructor(minX, minY, minZ, maxX, maxY, maxZ) {
    this.minX = minX
    this.minY = minY
    this.minZ = minZ
    this.maxX = maxX
    this.maxY = maxY
    this.maxZ = maxZ
  }

  static fromCenterSize(cx, cy, cz, sx, sy, sz) {
    return new Aabb(
      cx - sx * 0.5, cy - sy * 0.5, cz - sz * 0.5,
      cx + sx * 0.5, cy + sy * 0.5, cz + sz * 0.5
    )
  }

  get sizeX() { return this.maxX - this.minX }
  get sizeY() { return this.maxY - this.minY }
  get sizeZ() { return this.maxZ - this.minZ }

  intersects(other) {
    return this.minX <= other.maxX
      && this.maxX >= other.minX
      && this.minY <= other.maxY
      && this.maxY >= other.minY
      && this.minZ <= other.maxZ
      && this.maxZ >= other.minZ
  }

  containsPoint(x, y, z) {
    return x >= this.minX && x <= this.maxX
      && y >= this.minY && y <= this.maxY
      && z >= this.minZ && z <= this.maxZ
  }

  clone() {
    return new Aabb(this.minX, this.minY, this.minZ, this.maxX, this.maxY, this.maxZ)
  }
}

export class CylinderTool {
  constructor(radius, length, cornerRadius = 0) {
    this.radius = radius
    this.length = length
    this.cornerRadius = Math.min(Math.max(cornerRadius, 0), Math.min(radius, length * 0.5))
  }

  aabbAt(tipX, tipY, tipZ) {
    return new Aabb(
      tipX - this.radius, tipY, tipZ - this.radius,
      tipX + this.radius, tipY + this.length, tipZ + this.radius
    )
  }

  holderAabbAt(tipX, tipY, tipZ, holderLen, holderR) {
    const r = Math.max(holderR, this.radius)
    const bottom = tipY + this.length
    return new Aabb(
      tipX - r, bottom, tipZ - r,
      tipX + r, bottom + holderLen, tipZ + r
    )
  }

  containsPoint(tipX, tipY, tipZ, px, py, pz) {
    const dx = px - tipX
    const dz = pz - tipZ
    const distSq = dx * dx + dz * dz
    const rSq = this.radius * this.radius
    if (distSq > rSq + 1e-6) return false

    const yBottom = tipY
    const yTop = tipY + this.length
    const cr = this.cornerRadius

    if (cr <= 0) {
      return py >= yBottom && py <= yTop
    }

    const cylYBottom = yBottom + cr
    const cylYTop = yTop - cr

    if (py >= cylYBottom && py <= cylYTop) return true
    if (py > yTop || py < yBottom) return false

    let rEff
    if (py < cylYBottom) {
      const dy = cylYBottom - py
      const h = cr - dy
      if (h <= 0) return false
      const rEdge = Math.sqrt(Math.max(0, cr * cr - h * h))
      rEff = this.radius - cr + rEdge
    } else {
      const dy = py - cylYTop
      const h = cr - dy
      if (h <= 0) return false
      const rEdge = Math.sqrt(Math.max(0, cr * cr - h * h))
      rEff = this.radius - cr + rEdge
    }

    return distSq <= rEff * rEff + 1e-6
  }
}

export class VoxelGrid {
  constructor(bounds, voxelSize) {
    this.bounds = bounds
    this.voxelSize = Math.max(voxelSize, 0.01)
    this.sizeX = Math.max(1, Math.ceil(bounds.sizeX / this.voxelSize))
    this.sizeY = Math.max(1, Math.ceil(bounds.sizeY / this.voxelSize))
    this.sizeZ = Math.max(1, Math.ceil(bounds.sizeZ / this.voxelSize))
    this.total = this.sizeX * this.sizeY * this.sizeZ
    this.data = new Uint8Array(this.total)
    this.data.fill(VOXEL_SOLID)
    this._surfaceCache = null
    this._surfaceDirty = true
  }

  idx(ix, iy, iz) {
    return ix + iy * this.sizeX + iz * this.sizeX * this.sizeY
  }

  worldToVoxel(x, y, z) {
    const ix = Math.floor((x - this.bounds.minX) / this.voxelSize)
    const iy = Math.floor((y - this.bounds.minY) / this.voxelSize)
    const iz = Math.floor((z - this.bounds.minZ) / this.voxelSize)
    return [ix, iy, iz]
  }

  voxelToWorldCenter(ix, iy, iz) {
    const cx = this.bounds.minX + (ix + 0.5) * this.voxelSize
    const cy = this.bounds.minY + (iy + 0.5) * this.voxelSize
    const cz = this.bounds.minZ + (iz + 0.5) * this.voxelSize
    return [cx, cy, cz]
  }

  get(ix, iy, iz) {
    if (ix < 0 || iy < 0 || iz < 0) return VOXEL_EMPTY
    if (ix >= this.sizeX || iy >= this.sizeY || iz >= this.sizeZ) return VOXEL_EMPTY
    return this.data[this.idx(ix, iy, iz)]
  }

  set(ix, iy, iz, value) {
    if (ix < 0 || iy < 0 || iz < 0) return
    if (ix >= this.sizeX || iy >= this.sizeY || iz >= this.sizeZ) return
    this.data[this.idx(ix, iy, iz)] = value
    this._surfaceDirty = true
  }

  isSurfaceVoxel(ix, iy, iz) {
    if (this.data[this.idx(ix, iy, iz)] === VOXEL_EMPTY) return false
    if (ix === 0 || iy === 0 || iz === 0) return true
    if (ix + 1 >= this.sizeX || iy + 1 >= this.sizeY || iz + 1 >= this.sizeZ) return true
    return this.get(ix - 1, iy, iz) === VOXEL_EMPTY
      || this.get(ix + 1, iy, iz) === VOXEL_EMPTY
      || this.get(ix, iy - 1, iz) === VOXEL_EMPTY
      || this.get(ix, iy + 1, iz) === VOXEL_EMPTY
      || this.get(ix, iy, iz - 1) === VOXEL_EMPTY
      || this.get(ix, iy, iz + 1) === VOXEL_EMPTY
  }

  extractSurfaceVoxels(maxCount = 50000) {
    if (!this._surfaceDirty && this._surfaceCache && this._surfaceCache.length <= maxCount * 3) {
      return this._surfaceCache
    }
    const result = new Float32Array(Math.min(this.total, maxCount) * 3)
    let count = 0
    for (let iz = 0; iz < this.sizeZ && count < maxCount; iz++) {
      for (let iy = 0; iy < this.sizeY && count < maxCount; iy++) {
        for (let ix = 0; ix < this.sizeX && count < maxCount; ix++) {
          if (this.data[this.idx(ix, iy, iz)] === VOXEL_EMPTY) continue
          if (!this.isSurfaceVoxel(ix, iy, iz)) continue
          const [cx, cy, cz] = this.voxelToWorldCenter(ix, iy, iz)
          result[count * 3] = cx
          result[count * 3 + 1] = cy
          result[count * 3 + 2] = cz
          count++
        }
      }
    }
    const finalResult = result.subarray(0, count * 3)
    this._surfaceCache = finalResult
    this._surfaceDirty = false
    return finalResult
  }

  carveCylinderTool(tool, tipX, tipY, tipZ) {
    const toolAabb = tool.aabbAt(tipX, tipY, tipZ)
    const [minIx, minIy, minIz] = this.worldToVoxel(toolAabb.minX, toolAabb.minY, toolAabb.minZ)
    const [maxIx, maxIy, maxIz] = this.worldToVoxel(toolAabb.maxX, toolAabb.maxY, toolAabb.maxZ)

    const mnIx = Math.max(0, minIx)
    const mxIx = Math.min(this.sizeX - 1, maxIx)
    const mnIy = Math.max(0, minIy)
    const mxIy = Math.min(this.sizeY - 1, maxIy)
    const mnIz = Math.max(0, minIz)
    const mxIz = Math.min(this.sizeZ - 1, maxIz)

    if (mnIx > mxIx || mnIy > mxIy || mnIz > mxIz) return 0

    const halfV = this.voxelSize * 0.5
    let removed = 0

    for (let iz = mnIz; iz <= mxIz; iz++) {
      for (let iy = mnIy; iy <= mxIy; iy++) {
        for (let ix = mnIx; ix <= mxIx; ix++) {
          const idx = this.idx(ix, iy, iz)
          if (this.data[idx] === VOXEL_EMPTY) continue
          const [cx, cy, cz] = this.voxelToWorldCenter(ix, iy, iz)
          if (this._voxelIntersectsTool(tool, tipX, tipY, tipZ, cx, cy, cz, halfV)) {
            this.data[idx] = VOXEL_EMPTY
            removed++
          }
        }
      }
    }
    if (removed > 0) this._surfaceDirty = true
    return removed
  }

  _voxelIntersectsTool(tool, tipX, tipY, tipZ, cx, cy, cz, halfV) {
    if (tool.containsPoint(tipX, tipY, tipZ, cx, cy, cz)) return true

    const corners = [
      [cx - halfV, cy - halfV, cz - halfV],
      [cx + halfV, cy - halfV, cz - halfV],
      [cx - halfV, cy + halfV, cz - halfV],
      [cx + halfV, cy + halfV, cz - halfV],
      [cx - halfV, cy - halfV, cz + halfV],
      [cx + halfV, cy - halfV, cz + halfV],
      [cx - halfV, cy + halfV, cz + halfV],
      [cx + halfV, cy + halfV, cz + halfV],
    ]

    let inside = 0
    for (const [px, py, pz] of corners) {
      if (tool.containsPoint(tipX, tipY, tipZ, px, py, pz)) inside++
    }
    if (inside > 0 && inside < 8) return true

    const dx = cx - tipX
    const dz = cz - tipZ
    const distXZ = Math.sqrt(dx * dx + dz * dz)
    if (distXZ > tool.radius + halfV) return false
    if (cy + halfV < tipY || cy - halfV > tipY + tool.length) return false
    return true
  }

  carveSegment(tool, x0, y0, z0, x1, y1, z1, maxSteps = 256) {
    const dx = x1 - x0
    const dy = y1 - y0
    const dz = z1 - z0
    const dist = Math.sqrt(dx * dx + dy * dy + dz * dz)
    if (dist < 1e-6) {
      return this.carveCylinderTool(tool, x1, y1, z1)
    }
    const stepSize = this.voxelSize * 0.5
    const steps = Math.max(1, Math.min(maxSteps, Math.ceil(dist / stepSize)))
    let removed = 0
    for (let i = 0; i <= steps; i++) {
      const t = i / steps
      const x = x0 + dx * t
      const y = y0 + dy * t
      const z = z0 + dz * t
      removed += this.carveCylinderTool(tool, x, y, z)
    }
    return removed
  }

  checkCollisionAabb(aabb) {
    const [minIx, minIy, minIz] = this.worldToVoxel(aabb.minX, aabb.minY, aabb.minZ)
    const [maxIx, maxIy, maxIz] = this.worldToVoxel(aabb.maxX, aabb.maxY, aabb.maxZ)

    const mnIx = Math.max(0, minIx)
    const mxIx = Math.min(this.sizeX - 1, maxIx)
    const mnIy = Math.max(0, minIy)
    const mxIy = Math.min(this.sizeY - 1, maxIy)
    const mnIz = Math.max(0, minIz)
    const mxIz = Math.min(this.sizeZ - 1, maxIz)

    if (mnIx > mxIx || mnIy > mxIy || mnIz > mxIz) return false

    for (let iz = mnIz; iz <= mxIz; iz++) {
      for (let iy = mnIy; iy <= mxIy; iy++) {
        for (let ix = mnIx; ix <= mxIx; ix++) {
          if (this.data[this.idx(ix, iy, iz)] !== VOXEL_EMPTY) return true
        }
      }
    }
    return false
  }

  reset() {
    this.data.fill(VOXEL_SOLID)
    this._surfaceDirty = true
    this._surfaceCache = null
  }

  get solidCount() {
    let count = 0
    for (let i = 0; i < this.total; i++) {
      if (this.data[i] !== VOXEL_EMPTY) count++
    }
    return count
  }

  get removedRatio() {
    let empty = 0
    for (let i = 0; i < this.total; i++) {
      if (this.data[i] === VOXEL_EMPTY) empty++
    }
    return empty / this.total
  }
}

export class VoxelEngine {
  constructor(bounds, voxelSize, toolRadius, toolLength, cornerRadius = 0) {
    this.grid = new VoxelGrid(bounds, voxelSize)
    this.tool = new CylinderTool(toolRadius, toolLength, cornerRadius)
    this.holderLength = 80
    this.holderRadius = 35
    this.fixtures = []
    this._lastCarvedIdx = -1
  }

  setTool(radius, length, cornerRadius = 0) {
    this.tool = new CylinderTool(radius, length, cornerRadius)
  }

  setHolder(length, radius) {
    this.holderLength = length
    this.holderRadius = radius
  }

  addFixture(aabb) {
    this.fixtures.push(aabb)
  }

  clearFixtures() {
    this.fixtures = []
  }

  carveAt(tipX, tipY, tipZ) {
    return this.grid.carveCylinderTool(this.tool, tipX, tipY, tipZ)
  }

  carveSegment(x0, y0, z0, x1, y1, z1, maxSteps = 256) {
    return this.grid.carveSegment(this.tool, x0, y0, z0, x1, y1, z1, maxSteps)
  }

  checkToolCollision(tipX, tipY, tipZ) {
    const holderAabb = this.tool.holderAabbAt(tipX, tipY, tipZ, this.holderLength, this.holderRadius)
    return this.grid.checkCollisionAabb(holderAabb)
  }

  checkFixtureCollision(tipX, tipY, tipZ) {
    const holderAabb = this.tool.holderAabbAt(tipX, tipY, tipZ, this.holderLength, this.holderRadius)
    for (const fixture of this.fixtures) {
      if (fixture.intersects(holderAabb)) return true
    }
    return false
  }

  extractSurfacePoints(maxCount = 50000) {
    return this.grid.extractSurfaceVoxels(maxCount)
  }

  resetWorkpiece() {
    this.grid.reset()
    this._lastCarvedIdx = -1
  }

  get totalVoxels() { return this.grid.total }
  get solidVoxels() { return this.grid.solidCount }
  get removedRatio() { return this.grid.removedRatio }
  get voxelSize() { return this.grid.voxelSize }
  get boundsMin() { return [this.grid.bounds.minX, this.grid.bounds.minY, this.grid.bounds.minZ] }
  get boundsMax() { return [this.grid.bounds.maxX, this.grid.bounds.maxY, this.grid.bounds.maxZ] }
  get gridSize() { return [this.grid.sizeX, this.grid.sizeY, this.grid.sizeZ] }
}
