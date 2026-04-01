"use client"

import { motion } from "framer-motion"
import { Badge } from "@/components/ui/badge"

interface SkillCategoryProps {
  title: string
  skills: string[]
}

export function SkillCategory({ title, skills }: SkillCategoryProps) {
  return (
    <div>
      <h4 className="font-medium mb-2 text-white/90">{title}</h4>
      <div className="flex flex-wrap gap-2">
        {skills.map((skill, i) => (
          <motion.div
            key={skill}
            initial={{ opacity: 0, scale: 0.8 }}
            whileInView={{ opacity: 1, scale: 1 }}
            viewport={{ once: true }}
            transition={{ duration: 0.3, delay: 0.1 + i * 0.05 }}
          >
            <Badge
              variant="secondary"
              className="px-3 py-1 bg-white/10 hover:bg-white/20 text-white/80 hover:text-white border-white/10 transition-colors"
            >
              {skill}
            </Badge>
          </motion.div>
        ))}
      </div>
    </div>
  )
}
