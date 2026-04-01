"use client"

import { useEffect } from "react"
import Link from "next/link"
import Image from "next/image"
import { motion, useScroll, useTransform } from "framer-motion"
import { ArrowLeft } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { PageTransition } from "@/components/page-transition"
import { GradientText } from "@/components/gradient-text"
import { NoiseBackground } from "@/components/noise-background"

export default function GMKArchPage() {
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  const { scrollYProgress } = useScroll()
  const opacity = useTransform(scrollYProgress, [0, 0.2], [1, 0])
  const scale = useTransform(scrollYProgress, [0, 0.2], [1, 0.8])
  const y = useTransform(scrollYProgress, [0, 0.2], [0, -50])

  return (
    <PageTransition>
      <div className="bg-darkgray text-white min-h-screen">
        <NoiseBackground />

        <div className="h-[50vh] md:h-[70vh] relative flex items-center justify-center overflow-hidden">
          <motion.div style={{ opacity, scale, y }} className="absolute inset-0 z-0">
            <Image src="/arch-keycaps.png" alt="GMK Arch Keycap Set" fill className="object-cover" priority />
            <div className="absolute inset-0 bg-gradient-to-b from-darkgray/40 via-darkgray/60 to-darkgray" />
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.2 }}
            className="relative z-10 text-center px-4"
          >
            <Badge className="mb-4 bg-teal/20 text-teal border-teal/30">Product Design</Badge>
            <h1 className="text-4xl md:text-6xl font-bold mb-4">
              <GradientText>GMK Arch</GradientText>
            </h1>
            <p className="text-xl text-white/80 max-w-2xl mx-auto">
              A custom mechanical keyboard keycap set inspired by the Arch Linux operating system.
            </p>
          </motion.div>
        </div>

        <div className="container py-12 max-w-5xl">
          <Link
            href="/#projects"
            className="inline-flex items-center text-sm text-white/70 hover:text-teal transition-colors mb-8"
          >
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to Projects
          </Link>

          <div className="space-y-8">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.3 }}
                className="border border-white/10 rounded-lg p-6 bg-white/5"
              >
                <h3 className="font-semibold mb-2 text-teal">Timeline</h3>
                <p className="text-white/70">2020 - 2021</p>
              </motion.div>
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.4 }}
                className="border border-white/10 rounded-lg p-6 bg-white/5"
              >
                <h3 className="font-semibold mb-2 text-teal">Role</h3>
                <p className="text-white/70">Designer & Project Lead</p>
              </motion.div>
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.5 }}
                className="border border-white/10 rounded-lg p-6 bg-white/5"
              >
                <h3 className="font-semibold mb-2 text-teal">Status</h3>
                <p className="text-white/70">Successfully Produced & Sold</p>
              </motion.div>
            </div>

            <div className="space-y-6">
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.6 }}
              >
                <h2 className="text-2xl font-bold text-teal">The Problem</h2>
                <p className="text-white/80 mt-2">
                  Mechanical keyboard enthusiasts lacked a keycap set that embodied software minimalism principles. The
                  Arch Linux community, known for its minimalist and user-centric approach, had no dedicated keycap set
                  that represented their values and aesthetic preferences.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.7 }}
              >
                <h2 className="text-2xl font-bold text-teal">The Process</h2>
                <div className="space-y-4 mt-2">
                  <h3 className="text-xl font-semibold">Research & Concept</h3>
                  <p className="text-white/80">
                    I began by researching the core principles of Arch Linux: simplicity, modernity, pragmatism, user
                    centrality, and versatility. I studied the Arch Linux branding, color schemes, and community
                    preferences to ensure the design would resonate with the target audience.
                  </p>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6 my-8">
                    <div className="aspect-video relative rounded-lg overflow-hidden border border-white/10">
                      <Image src="/blue-gray-keycaps.png" alt="GMK Arch Color Palette" fill className="object-cover" />
                    </div>
                    <div className="aspect-video relative rounded-lg overflow-hidden border border-white/10">
                      <Image
                        src="/keycap-design-sketches.png"
                        alt="GMK Arch Design Concepts"
                        fill
                        className="object-cover"
                      />
                    </div>
                  </div>

                  <h3 className="text-xl font-semibold">Design & Iteration</h3>
                  <p className="text-white/80">
                    I created multiple design iterations, focusing on a color scheme that reflected Arch Linux's blue
                    tones while ensuring readability and aesthetic appeal. The design went through several revisions
                    based on community feedback from forums like GeekHack and Reddit's mechanical keyboard communities.
                  </p>

                  <h3 className="text-xl font-semibold">Production & Launch</h3>
                  <p className="text-white/80">
                    After finalizing the design, I coordinated with GMK, a German manufacturer known for high-quality
                    keycaps, to produce the set. This involved detailed specifications for colors, legends, and
                    packaging. I organized an "Interest Check" phase to gauge community interest, followed by a "Group
                    Buy" where customers could pre-order the set.
                  </p>
                </div>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.8 }}
              >
                <h2 className="text-2xl font-bold text-teal">The Solution</h2>
                <p className="text-white/80 mt-2">
                  The final GMK Arch design featured a clean, minimalist aesthetic with a color palette inspired by the
                  Arch Linux logo and interface. The set included specialized modifier keys with symbols relevant to
                  Linux users and programmers. The packaging was designed to complement the keycaps with matching
                  artwork and information about the inspiration behind the set.
                </p>

                <div className="aspect-video relative rounded-lg overflow-hidden border border-white/10 my-8">
                  <Image src="/blue-gray-keyboard.png" alt="GMK Arch on a keyboard" fill className="object-cover" />
                </div>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.9 }}
              >
                <h2 className="text-2xl font-bold text-teal">Outcome & Impact</h2>
                <p className="text-white/80 mt-2">
                  GMK Arch was successfully funded during the Group Buy phase and went into production. The set was sold
                  through various international vendors and received positive feedback from the community. It became
                  recognized within the mechanical keyboard community as a well-executed theme that successfully
                  captured the essence of Arch Linux.
                </p>

                <div className="border-l-4 border-teal pl-6 py-4 my-8 bg-white/5">
                  <p className="italic text-white/90">
                    "GMK Arch perfectly captures the minimalist ethos of Arch Linux while delivering a beautiful,
                    functional keycap set. The attention to detail in both the colors and legends shows a deep
                    understanding of the source material."
                  </p>
                  <p className="text-sm text-white/60 mt-2">— MechanicalKeyboards.com Review</p>
                </div>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 1.0 }}
              >
                <h2 className="text-2xl font-bold text-teal">Reflections & Learnings</h2>
                <p className="text-white/80 mt-2">
                  Through this project, I learned valuable lessons about product design, community engagement, and
                  manufacturing processes. The importance of iterative design based on user feedback was reinforced, as
                  was the need for clear communication with manufacturers about specifications and quality expectations.
                </p>
                <p className="text-white/80 mt-4">
                  If I were to approach this project again, I would implement a more structured feedback collection
                  system and possibly explore additional kit options to accommodate a wider range of keyboard layouts.
                  The experience of bringing GMK Arch from concept to reality has significantly informed my approach to
                  subsequent design projects.
                </p>
              </motion.div>
            </div>

            <div className="flex justify-between items-center pt-8 border-t border-white/10">
              <Button asChild variant="outline" className="border-white/20 hover:bg-white/10 hover:text-teal">
                <Link href="/#projects">
                  <ArrowLeft className="mr-2 h-4 w-4" />
                  Back to Projects
                </Link>
              </Button>
              <Button asChild className="bg-teal text-darkgray hover:bg-teal/80">
                <Link href="/#contact">Contact Me</Link>
              </Button>
            </div>
          </div>
        </div>
      </div>
    </PageTransition>
  )
}
