import { useEffect, useRef, useState } from "react";

interface SpriteSheetProps {
  src: string;
  frameCount: number;
  columns: number;
  fps: number;
  width: number;
  height: number;
  flipX?: boolean;
}

export default function SpriteSheet({
  src,
  frameCount,
  columns,
  fps,
  width,
  height,
  flipX = false,
}: SpriteSheetProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const imageRef = useRef<HTMLImageElement | null>(null);
  const [loaded, setLoaded] = useState(false);
  const frameRef = useRef(0);
  const lastFrameTimeRef = useRef(0);

  useEffect(() => {
    const img = new Image();
    img.src = src;
    img.onload = () => {
      imageRef.current = img;
      setLoaded(true);
    };
  }, [src]);

  useEffect(() => {
    if (!loaded || !canvasRef.current || !imageRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const img = imageRef.current;
    const rows = Math.ceil(frameCount / columns);
    const srcFrameW = img.naturalWidth / columns;
    const srcFrameH = img.naturalHeight / rows;
    const interval = 1000 / fps;

    let animId: number;

    const render = (timestamp: number) => {
      if (timestamp - lastFrameTimeRef.current >= interval) {
        lastFrameTimeRef.current = timestamp;
        frameRef.current = (frameRef.current + 1) % frameCount;
      }

      const frame = frameRef.current;
      const col = frame % columns;
      const row = Math.floor(frame / columns);

      ctx.clearRect(0, 0, width, height);
      ctx.save();

      if (flipX) {
        ctx.translate(width, 0);
        ctx.scale(-1, 1);
      }

      // Disable smoothing for pixel art
      ctx.imageSmoothingEnabled = false;

      ctx.drawImage(
        img,
        col * srcFrameW,
        row * srcFrameH,
        srcFrameW,
        srcFrameH,
        0,
        0,
        width,
        height
      );

      ctx.restore();
      animId = requestAnimationFrame(render);
    };

    animId = requestAnimationFrame(render);
    return () => cancelAnimationFrame(animId);
  }, [loaded, src, frameCount, columns, fps, width, height, flipX]);

  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      style={{ width, height, imageRendering: "pixelated" }}
    />
  );
}
