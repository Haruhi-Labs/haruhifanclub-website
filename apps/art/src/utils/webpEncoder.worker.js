import encodeWebP from '@jsquash/webp/encode.js'

self.addEventListener('message', async ({ data }) => {
  const { id, pixels, width, height, quality } = data

  try {
    const imageData = new ImageData(new Uint8ClampedArray(pixels), width, height)
    const encoded = await encodeWebP(imageData, { quality })
    self.postMessage({ id, encoded }, [encoded])
  } catch (error) {
    self.postMessage({
      id,
      error: error instanceof Error ? error.message : String(error),
    })
  }
})
