import assert from 'node:assert/strict'
import { afterEach, beforeEach, test } from 'node:test'
import { compressToWebP, disposeEncoderWorker } from './imageCompressor.js'

const originalGlobals = {
  createImageBitmap: globalThis.createImageBitmap,
  OffscreenCanvas: globalThis.OffscreenCanvas,
  Worker: globalThis.Worker,
}

class FakeWorker {
  static instances = []
  static nextPostError = null

  constructor() {
    this.listeners = new Map()
    this.terminated = false
    this.postError = FakeWorker.nextPostError
    FakeWorker.nextPostError = null
    FakeWorker.instances.push(this)
  }

  addEventListener(type, listener) {
    this.listeners.set(type, listener)
  }

  postMessage(message) {
    if (this.postError) throw this.postError
    this.message = message
  }

  emitMessage(data) {
    this.listeners.get('message')?.({ data })
  }

  terminate() {
    this.terminated = true
  }
}

class FakeCanvas {
  constructor(width, height) {
    this.width = width
    this.height = height
  }

  getContext() {
    return {
      drawImage() {},
      getImageData: () => ({
        data: new Uint8ClampedArray(this.width * this.height * 4),
        width: this.width,
        height: this.height,
      }),
    }
  }
}

beforeEach(() => {
  FakeWorker.instances = []
  FakeWorker.nextPostError = null
  globalThis.createImageBitmap = async () => ({
    width: 2,
    height: 2,
    close() {},
  })
  globalThis.OffscreenCanvas = FakeCanvas
  globalThis.Worker = FakeWorker
})

afterEach(() => {
  disposeEncoderWorker()
  restoreGlobal('createImageBitmap')
  restoreGlobal('OffscreenCanvas')
  restoreGlobal('Worker')
})

test('Worker 返回编码数据后生成 WebP Blob', async () => {
  const resultPromise = compressToWebP(new Blob(['image']), 0.9, 1920)
  const worker = await waitForWorker()
  const encoded = new Uint8Array([82, 73, 70, 70, 87, 69, 66, 80]).buffer

  worker.emitMessage({ id: worker.message.id, encoded })

  const result = await resultPromise
  assert.equal(result.type, 'image/webp')
  assert.equal(result.size, encoded.byteLength)
})

test('释放 Worker 会拒绝待处理任务，后续编码可重新创建 Worker', async () => {
  const firstResult = compressToWebP(new Blob(['first']))
  const firstWorker = await waitForWorker()

  disposeEncoderWorker()

  await assert.rejects(firstResult, /WebP 编码线程已释放/)
  assert.equal(firstWorker.terminated, true)

  const secondResult = compressToWebP(new Blob(['second']))
  const secondWorker = await waitForWorker(2)
  secondWorker.emitMessage({
    id: secondWorker.message.id,
    encoded: new Uint8Array([1, 2, 3]).buffer,
  })

  await secondResult
  assert.notEqual(secondWorker, firstWorker)
})

test('postMessage 同步失败时拒绝任务且不影响后续释放', async () => {
  const expected = new Error('无法传输像素数据')
  FakeWorker.nextPostError = expected
  const failedPromise = compressToWebP(new Blob(['image']))
  await assert.rejects(failedPromise, /无法传输像素数据/)
  const failedWorker = FakeWorker.instances[0]

  disposeEncoderWorker()
  assert.equal(failedWorker.terminated, true)

  const nextResult = compressToWebP(new Blob(['next']))
  const nextWorker = await waitForWorker(2)
  nextWorker.emitMessage({
    id: nextWorker.message.id,
    encoded: new Uint8Array([1, 2, 3]).buffer,
  })
  await nextResult
})

test('编码超时会拒绝任务并终止失去响应的 Worker', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] })
  const resultPromise = compressToWebP(new Blob(['image']))
  const worker = await waitForWorker()

  t.mock.timers.tick(30_000)

  await assert.rejects(resultPromise, /WebP 编码超时/)
  assert.equal(worker.terminated, true)
})

async function waitForWorker(expectedCount = 1) {
  for (let attempt = 0; attempt < 10; attempt += 1) {
    if (FakeWorker.instances.length >= expectedCount) {
      return FakeWorker.instances[expectedCount - 1]
    }
    await new Promise((resolve) => setImmediate(resolve))
  }
  throw new Error('Worker 未按预期创建')
}

function restoreGlobal(name) {
  const value = originalGlobals[name]
  if (value === undefined) delete globalThis[name]
  else globalThis[name] = value
}
