"use client"

import { useEffect, useRef } from "react"

export function NoiseBackground() {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext("2d")
    if (!ctx) return

    const resize = () => {
      canvas.width = window.innerWidth
      canvas.height = window.innerHeight
    }

    resize()
    window.addEventListener("resize", resize)

    let animationFrameId: number
    let frame = 0

    const noise = (ctx: CanvasRenderingContext2D, frame: number) => {
      const w = ctx.canvas.width
      const h = ctx.canvas.height
      const idata = ctx.createImageData(w, h)
      const buffer32 = new Uint32Array(idata.data.buffer)
      const len = buffer32.length

      for (let i = 0; i < len; i++) {
        if (Math.random() < 0.05) {
          const opacity = Math.random() * 0.05
          buffer32[i] = (opacity * 255) << 24
        }
      }

      ctx.putImageData(idata, 0, 0)
    }

    const loop = () => {
      noise(ctx, frame)
      frame++
      animationFrameId = requestAnimationFrame(loop)
    }

    loop()

    return () => {
      window.removeEventListener("resize", resize)
      cancelAnimationFrame(animationFrameId)
    }
  }, [])

  return (
    <canvas
      ref={canvasRef}
      className="fixed inset-0 z-0 pointer-events-none opacity-20"
      style={{ mixBlendMode: "overlay" }}
    />
  )
}
