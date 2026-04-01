"use client"

import { motion } from "framer-motion"

interface SplitTextProps {
  text: string
  className?: string
  charClassName?: string
  animation?: {
    hidden: any
    visible: (i: number) => any
  }
}

export function SplitText({ text, className = "", charClassName = "", animation }: SplitTextProps) {
  const letters = Array.from(text)

  const defaultAnimation = {
    hidden: { opacity: 0, y: 50 },
    visible: (i: number) => ({
      opacity: 1,
      y: 0,
      transition: { duration: 0.5, delay: i * 0.05 },
    }),
  }

  const { hidden, visible } = animation || defaultAnimation

  return (
    <span className={className}>
      {letters.map((char, i) => (
        <motion.span
          key={`${char}-${i}`}
          className={charClassName}
          custom={i}
          initial="hidden"
          animate="visible"
          variants={{
            hidden,
            visible,
          }}
        >
          {char === " " ? "\u00A0" : char}
        </motion.span>
      ))}
    </span>
  )
}
