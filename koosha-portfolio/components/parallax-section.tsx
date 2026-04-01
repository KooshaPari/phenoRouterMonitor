"use client"

import { forwardRef, type ReactNode, type CSSProperties } from "react"
import { motion, useScroll, useTransform } from "framer-motion"

interface ParallaxSectionProps {
  children: ReactNode
  className?: string
  style?: CSSProperties
  factor?: number
}

export const ParallaxSection = forwardRef<HTMLDivElement, ParallaxSectionProps>(
  ({ children, className, style, factor = 0.2, ...props }, ref) => {
    const { scrollYProgress } = useScroll({
      target: ref as any,
      offset: ["start end", "end start"],
    })

    // Instead of using multiply, we'll create a new transform with the factor applied
    const y = useTransform(scrollYProgress, [0, 1], [0, -200 * factor])

    return (
      <section ref={ref} className={className} style={style} {...props}>
        <motion.div style={{ y }}>{children}</motion.div>
      </section>
    )
  },
)

ParallaxSection.displayName = "ParallaxSection"
