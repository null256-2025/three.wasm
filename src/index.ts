// three.wasm — the future of 3D on the web

// 4x4 matrices
const matrix = new Float32Array(16);
const projection = new Float32Array(16);
const mvp = new Float32Array(16);

// Cube: 6 faces × 2 triangles × 3 vertices × 3 components = 108 floats
const positions = new Float32Array(108);
const transformed = new Float32Array(108);
const normals = new Float32Array(108);
const transformedNormals = new Float32Array(108);

function initCube(): void {
  const v: f32[] = [
    // Front
    -1, -1,  1,   1, -1,  1,   1,  1,  1,
    -1, -1,  1,   1,  1,  1,  -1,  1,  1,
    // Back
    -1, -1, -1,  -1,  1, -1,   1,  1, -1,
    -1, -1, -1,   1,  1, -1,   1, -1, -1,
    // Top
    -1,  1, -1,  -1,  1,  1,   1,  1,  1,
    -1,  1, -1,   1,  1,  1,   1,  1, -1,
    // Bottom
    -1, -1, -1,   1, -1, -1,   1, -1,  1,
    -1, -1, -1,   1, -1,  1,  -1, -1,  1,
    // Right
     1, -1, -1,   1,  1, -1,   1,  1,  1,
     1, -1, -1,   1,  1,  1,   1, -1,  1,
    // Left
    -1, -1, -1,  -1, -1,  1,  -1,  1,  1,
    -1, -1, -1,  -1,  1,  1,  -1,  1, -1,
  ];

  const n: f32[] = [
     0,  0,  1,   0,  0,  1,   0,  0,  1,
     0,  0,  1,   0,  0,  1,   0,  0,  1,
     0,  0, -1,   0,  0, -1,   0,  0, -1,
     0,  0, -1,   0,  0, -1,   0,  0, -1,
     0,  1,  0,   0,  1,  0,   0,  1,  0,
     0,  1,  0,   0,  1,  0,   0,  1,  0,
     0, -1,  0,   0, -1,  0,   0, -1,  0,
     0, -1,  0,   0, -1,  0,   0, -1,  0,
     1,  0,  0,   1,  0,  0,   1,  0,  0,
     1,  0,  0,   1,  0,  0,   1,  0,  0,
    -1,  0,  0,  -1,  0,  0,  -1,  0,  0,
    -1,  0,  0,  -1,  0,  0,  -1,  0,  0,
  ];

  for (let i = 0; i < 108; i++) {
    positions[i] = v[i];
    normals[i] = n[i];
  }
}

function setIdentity(m: Float32Array): void {
  for (let i = 0; i < 16; i++) m[i] = 0;
  m[0] = 1; m[5] = 1; m[10] = 1; m[15] = 1;
}

function perspectiveMat(fov: f32, aspect: f32, near: f32, far: f32): void {
  const f: f32 = 1.0 / Mathf.tan(fov / 2.0);
  const nf: f32 = 1.0 / (near - far);
  for (let i = 0; i < 16; i++) projection[i] = 0;
  projection[0] = f / aspect;
  projection[5] = f;
  projection[10] = (far + near) * nf;
  projection[11] = -1;
  projection[14] = 2 * far * near * nf;
}

function rotateY(m: Float32Array, angle: f32): void {
  const c = Mathf.cos(angle);
  const s = Mathf.sin(angle);
  const m0 = m[0], m2 = m[2], m4 = m[4], m6 = m[6];
  const m8 = m[8], m10 = m[10], m12 = m[12], m14 = m[14];
  m[0]  = m0 * c - m2 * s;   m[2]  = m0 * s + m2 * c;
  m[4]  = m4 * c - m6 * s;   m[6]  = m4 * s + m6 * c;
  m[8]  = m8 * c - m10 * s;  m[10] = m8 * s + m10 * c;
  m[12] = m12 * c - m14 * s; m[14] = m12 * s + m14 * c;
}

function rotateX(m: Float32Array, angle: f32): void {
  const c = Mathf.cos(angle);
  const s = Mathf.sin(angle);
  const m1 = m[1], m2 = m[2], m5 = m[5], m6 = m[6];
  const m9 = m[9], m10 = m[10], m13 = m[13], m14 = m[14];
  m[1]  = m1 * c + m2 * s;   m[2]  = -m1 * s + m2 * c;
  m[5]  = m5 * c + m6 * s;   m[6]  = -m5 * s + m6 * c;
  m[9]  = m9 * c + m10 * s;  m[10] = -m9 * s + m10 * c;
  m[13] = m13 * c + m14 * s; m[14] = -m13 * s + m14 * c;
}

function multiplyMat(out: Float32Array, a: Float32Array, b: Float32Array): void {
  for (let i = 0; i < 4; i++) {
    for (let j = 0; j < 4; j++) {
      let sum: f32 = 0;
      for (let k = 0; k < 4; k++) {
        sum += a[k * 4 + j] * b[i * 4 + k];
      }
      out[i * 4 + j] = sum;
    }
  }
}

export function init(): void {
  initCube();
  perspectiveMat(0.8, 1.0, 0.1, 100.0);
}

export function update(time: f32, aspect: f32): void {
  perspectiveMat(1.2, aspect, 0.1, 100.0);
  setIdentity(matrix);
  rotateY(matrix, time * 0.7);
  rotateX(matrix, time * 0.5);
  matrix[14] = -6;
  multiplyMat(mvp, projection, matrix);

  for (let i = 0; i < 36; i++) {
    const idx = i * 3;
    const x = positions[idx];
    const y = positions[idx + 1];
    const z = positions[idx + 2];

    const w: f32 = mvp[3] * x + mvp[7] * y + mvp[11] * z + mvp[15];
    transformed[idx]     = (mvp[0] * x + mvp[4] * y + mvp[8]  * z + mvp[12]) / w;
    transformed[idx + 1] = (mvp[1] * x + mvp[5] * y + mvp[9]  * z + mvp[13]) / w;
    transformed[idx + 2] = (mvp[2] * x + mvp[6] * y + mvp[10] * z + mvp[14]) / w;

    const nx = normals[idx];
    const ny = normals[idx + 1];
    const nz = normals[idx + 2];
    transformedNormals[idx]     = matrix[0] * nx + matrix[4] * ny + matrix[8]  * nz;
    transformedNormals[idx + 1] = matrix[1] * nx + matrix[5] * ny + matrix[9]  * nz;
    transformedNormals[idx + 2] = matrix[2] * nx + matrix[6] * ny + matrix[10] * nz;
  }
}

export function getTransformedPtr(): usize {
  return changetype<usize>(transformed.buffer);
}

export function getNormalsPtr(): usize {
  return changetype<usize>(transformedNormals.buffer);
}

export function getVertexCount(): i32 {
  return 36;
}
