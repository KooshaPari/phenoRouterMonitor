"use client"

import { motion } from "framer-motion"
import Link from "next/link"
import Image from "next/image"
import { ArrowRight } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"

interface ProjectCardProps {
  title: string
  category: string
  description: string
  image: string
  link: string
  index: number
}

export function ProjectCard({ title, category, description, image, link, index }: ProjectCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-100px" }}
      transition={{ duration: 0.6, delay: 0.4 + index * 0.1 }}
      whileHover={{ y: -5 }}
      className="group border border-white/10 rounded-lg overflow-hidden bg-white/5 backdrop-blur-sm transition-all hover:border-teal/30"
    >
      <div className="aspect-video relative overflow-hidden">
        <Image
          src={image || "/placeholder.svg"}
          alt={title}
          fill
          className="object-cover transition-transform duration-700 group-hover:scale-110"
        />
        <div className="absolute inset-0 bg-gradient-to-t from-darkgray via-transparent to-transparent" />
        <div className="absolute inset-0 bg-teal/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
      </div>
      <div className="p-6">
        <Badge className="mb-2 bg-teal/20 text-teal border-teal/30 hover:bg-teal/30">{category}</Badge>
        <h3 className="text-xl font-semibold mb-2 group-hover:text-teal transition-colors">{title}</h3>
        <p className="text-white/70 mb-4">{description}</p>
        <Button
          asChild
          variant="outline"
          size="sm"
          className="border-white/20 hover:bg-white/10 hover:border-teal/50 hover:text-teal group/button"
        >
          <Link href={link}>
            View Project{" "}
            <ArrowRight className="ml-2 h-4 w-4 transition-transform duration-300 group-hover/button:translate-x-1" />
          </Link>
        </Button>
      </div>
    </motion.div>
  )
}
