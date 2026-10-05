/**
 * QR acquisition for the import flow: image files and screen capture.
 *
 * This is deliberately face-side input plumbing, not shared logic — the
 * *meaning* of a decoded payload (otpauth, migration batch, export JSON)
 * is parsed by `castellan-otp` behind `preview_otp_import`, and this
 * module never interprets a byte of it. The decoder is jsqr rather than
 * the Rust barcode crate the task plan sketched: the airlocked toolchain
 * vendors no QR/image crates, npm is the one reachable registry, and
 * pixel-decoding a frame the webview already holds is presentation-layer
 * work by the same rule that keeps `getDisplayMedia` here.
 */

import jsQR from "jsqr";

/** Decode a QR from one bitmap frame, or `null` when none is found. */
function decodeFrame(bitmap: ImageBitmap): string | null {
  const canvas = document.createElement("canvas");
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  const context = canvas.getContext("2d");
  if (!context) return null;
  context.drawImage(bitmap, 0, 0);
  const image = context.getImageData(0, 0, canvas.width, canvas.height);
  const result = jsQR(image.data, image.width, image.height);
  return result?.data ?? null;
}

/** Decode a QR from an image file (a saved screenshot, a photo). */
export async function decodeImage(file: File): Promise<string | null> {
  try {
    const bitmap = await createImageBitmap(file);
    const decoded = decodeFrame(bitmap);
    bitmap.close();
    return decoded;
  } catch {
    return null;
  }
}

/**
 * Capture the screen and scan it for a QR: the "import from another
 * app's window" path. The user picks what to share through the
 * platform's own picker; frames are sampled for a few seconds so there
 * is time to bring the QR on screen, and the track always stops —
 * capture must never outlive the scan.
 */
export async function captureScreen(): Promise<string | null> {
  const stream = await navigator.mediaDevices.getDisplayMedia({ video: true });
  const video = document.createElement("video");
  video.srcObject = stream;
  video.muted = true;
  try {
    await video.play();
    for (let attempt = 0; attempt < 15; attempt += 1) {
      if (video.videoWidth > 0) {
        const bitmap = await createImageBitmap(video);
        const decoded = decodeFrame(bitmap);
        bitmap.close();
        if (decoded !== null) return decoded;
      }
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
    return null;
  } finally {
    video.pause();
    video.srcObject = null;
    for (const track of stream.getTracks()) track.stop();
  }
}
