"use client"

import { useEffect } from "react"
import Link from "next/link"
import Image from "next/image"
import { motion } from "framer-motion"
import { ArrowLeft, ArrowRight } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"

export default function ProjectsPage() {
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  return (
    <div className="bg-darkgray text-white min-h-screen">
      <div className="container py-12">
        <Link
          href="/"
          className="inline-flex items-center text-sm text-white/70 hover:text-teal transition-colors mb-8"
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to Home
        </Link>

        <div className="space-y-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            className="space-y-4"
          >
            <h1 className="text-4xl font-bold">All Projects</h1>
            <p className="text-xl text-white/70">
              A comprehensive collection of my work across software development and design.
            </p>
          </motion.div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {[
              {
                title: "GMK Arch",
                category: "Product Design",
                description:
                  "A custom mechanical keyboard keycap set inspired by Arch Linux, focusing on minimalism and functionality.",
                image: "/blue-gray-keycaps.png",
                link: "/projects/gmk-arch",
              },
              {
                title: "WITF Board",
                category: "Hardware Design",
                description: "Custom mechanical keyboard with seamless aluminum case and custom PCB.",
                image: "/custom-aluminum-keyboard.png",
                link: "/projects/witf-board",
              },
              {
                title: "Phenotype Platform",
                category: "Web Development",
                description: "E-commerce platform for custom mechanical keyboards and accessories.",
                image: "/mechanical-keyboard-storefront.png",
                link: "/projects/phenotype",
              },
              {
                title: "DSS Cipher",
                category: "Product Design",
                description: "Keycap set with a cryptography theme. Designed color scheme, legends, and packaging.",
                image: "/crypto-keys.png",
                link: "/projects/dss-cipher",
              },
              {
                title: "Portfolio Website",
                category: "Web Development",
                description: "Personal portfolio website built with Next.js and deployed on Vercel.",
                image: "/project-thumbnail-2.jpg",
                link: "/projects/portfolio",
              },
              {
                title: "Data Visualization Dashboard",
                category: "Software Development",
                description: "Interactive dashboard for visualizing complex datasets using React and D3.js.",
                image: "/data-dashboard-overview.png",
                link: "/projects/data-dashboard",
              },
              {
                title: "School Event Flyers",
                category: "Graphic Design",
                description: "Collection of flyers and promotional materials designed for school events.",
                image: "/vibrant-celebration.png",
                link: "/projects/flyers",
              },
              {
                title: "Club Logos",
                category: "Graphic Design",
                description: "Logo designs for various school and community clubs and organizations.",
                image: "/modern-logo-collection.png",
                link: "/projects/logos",
              },
            ].map((project, i) => (
              <motion.div
                key={project.title}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.2 + i * 0.1 }}
                whileHover={{ y: -5 }}
                className="group border border-white/10 rounded-lg overflow-hidden bg-white/5 backdrop-blur-sm transition-all hover:border-teal/30"
              >
                <div className="aspect-video relative overflow-hidden">
                  <Image
                    src={project.image || "/placeholder.svg"}
                    alt={project.title}
                    fill
                    className="object-cover transition-transform duration-700 group-hover:scale-110"
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-darkgray via-transparent to-transparent" />
                </div>
                <div className="p-6">
                  <Badge className="mb-2 bg-teal/20 text-teal border-teal/30 hover:bg-teal/30">
                    {project.category}
                  </Badge>
                  <h3 className="text-xl font-semibold mb-2 group-hover:text-teal transition-colors">
                    {project.title}
                  </h3>
                  <p className="text-white/70 mb-4">{project.description}</p>
                  <Button
                    asChild
                    variant="outline"
                    size="sm"
                    className="border-white/20 hover:bg-white/10 hover:border-teal/50 hover:text-teal group/button"
                  >
                    <Link href={project.link}>
                      View Project{" "}
                      <ArrowRight className="ml-2 h-4 w-4 transition-transform duration-300 group-hover/button:translate-x-1" />
                    </Link>
                  </Button>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
