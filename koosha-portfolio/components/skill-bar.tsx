"use client"

import { motion } from "framer-motion"

interface SkillBarProps {
  name: string
  percentage: number
  delay?: number
}

export function SkillBar({ name, percentage, delay = 0 }: SkillBarProps) {
  return (
    <div className="space-y-2">
      <div className="flex justify-between">
        <span className="text-sm font-medium text-white/90">{name}</span>
        <motion.span
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: delay + 0.6 }}
          className="text-sm font-medium text-teal"
        >
          {percentage}%
        </motion.span>
      </div>
      <div className="h-2 bg-white/10 rounded-full overflow-hidden">
        <motion.div
          className="h-full bg-gradient-to-r from-teal/80 to-teal rounded-full"
          initial={{ width: 0 }}
          whileInView={{ width: `${percentage}%` }}
          viewport={{ once: true }}
          transition={{ duration: 0.8, delay: delay + 0.2, ease: "easeOut" }}
        />
      </div>
    </div>
  )
}
