import * as THREE from 'three'

export class STLLoader {
  static async loadFromFile(filePath) {
    const result = await window.fileAPI.readBinaryFile(filePath)
    if (!result?.ok) {
      throw new Error(result?.error || '读取STL文件失败')
    }
    return STLLoader.parse(result.buffer)
  }

  static async loadFromArrayBuffer(buffer) {
    return STLLoader.parse(buffer)
  }

  static parse(buffer) {
    if (!buffer) throw new Error('空数据')
    const bytes = new Uint8Array(buffer)

    const isAscii = STLLoader._isASCII(bytes)
    if (isAscii) {
      return STLLoader._parseASCII(bytes)
    } else {
      return STLLoader._parseBinary(bytes, buffer)
    }
  }

  static _isASCII(bytes) {
    if (bytes.length < 80) return true
    const header = new TextDecoder('ascii').decode(bytes.subarray(0, 80))
    return header.trim().toLowerCase().startsWith('solid')
  }

  static _parseASCII(bytes) {
    const text = new TextDecoder('utf-8').decode(bytes)
    const lines = text.split(/\r?\n/)

    const positions = []
    const normals = []
    const indices = []

    let vertexCount = 0
    let currentNormal = [0, 0, 0]

    for (let line of lines) {
      line = line.trim()
      if (!line) continue
      const parts = line.split(/\s+/)
      const first = parts[0].toLowerCase()

      if (first === 'facet' && parts[1] === 'normal') {
        currentNormal = [
          parseFloat(parts[2]),
          parseFloat(parts[3]),
          parseFloat(parts[4]),
        ]
      } else if (first === 'vertex') {
        positions.push(parseFloat(parts[1]), parseFloat(parts[2]), parseFloat(parts[3]))
        normals.push(currentNormal[0], currentNormal[1], currentNormal[2])
        indices.push(vertexCount++)
      }
    }

    return {
      positions: new Float32Array(positions),
      normals: new Float32Array(normals),
      indices: new Uint32Array(indices),
      triangleCount: vertexCount / 3,
    }
  }

  static _parseBinary(bytes, buffer) {
    const dv = new DataView(buffer)
    let offset = 80
    const triangleCount = dv.getUint32(offset, true)
    offset += 4

    const positions = new Float32Array(triangleCount * 9)
    const normals = new Float32Array(triangleCount * 9)
    const indices = new Uint32Array(triangleCount * 3)

    for (let t = 0; t < triangleCount; t++) {
      const nx = dv.getFloat32(offset, true)
      const ny = dv.getFloat32(offset + 4, true)
      const nz = dv.getFloat32(offset + 8, true)
      offset += 12

      for (let v = 0; v < 3; v++) {
        const base = (t * 3 + v) * 3
        positions[base] = dv.getFloat32(offset, true)
        positions[base + 1] = dv.getFloat32(offset + 4, true)
        positions[base + 2] = dv.getFloat32(offset + 8, true)
        normals[base] = nx
        normals[base + 1] = ny
        normals[base + 2] = nz
        indices[t * 3 + v] = t * 3 + v
        offset += 12
      }
      offset += 2
    }

    return {
      positions,
      normals,
      indices,
      triangleCount,
    }
  }

  static toMesh(data, options = {}) {
    const geometry = new THREE.BufferGeometry()
    geometry.setAttribute('position', new THREE.BufferAttribute(data.positions, 3))
    if (data.normals) {
      geometry.setAttribute('normal', new THREE.BufferAttribute(data.normals, 3))
    } else {
      geometry.computeVertexNormals()
    }
    if (data.indices) {
      geometry.setIndex(new THREE.BufferAttribute(data.indices, 1))
    }
    geometry.computeBoundingBox()

    const material = new THREE.MeshStandardMaterial({
      color: options.color || 0x8899aa,
      roughness: options.roughness ?? 0.5,
      metalness: options.metalness ?? 0.5,
      transparent: options.transparent ?? false,
      opacity: options.opacity ?? 1,
      side: options.side ?? THREE.DoubleSide,
    })

    return new THREE.Mesh(geometry, material)
  }
}
