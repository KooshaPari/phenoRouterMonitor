"use client"

import { useEffect, useRef, useState } from "react"
import Link from "next/link"
import { motion, AnimatePresence, useScroll, useTransform } from "framer-motion"
import { ArrowRight, Github, Linkedin, Mail, Menu, X } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"
import { useMobile } from "@/hooks/use-mobile"
import { Cursor } from "@/components/cursor"
import { SplitText } from "@/components/split-text"
import { ProjectCard } from "@/components/project-card"
import { SkillBar } from "@/components/skill-bar"
import { KeyboardModel } from "@/components/keyboard-model"
import { PageTransition } from "@/components/page-transition"
import { ParallaxSection } from "@/components/parallax-section"
import { NoiseBackground } from "@/components/noise-background"
import { GradientText } from "@/components/gradient-text"
import { FloatingParticles } from "@/components/floating-particles"

export default function Home() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)
  const isMobile = useMobile()
  const { scrollY } = useScroll()
  const headerOpacity = useTransform(scrollY, [0, 100], [1, 0.8])
  const headerBlur = useTransform(scrollY, [0, 100], [0, 8])
  const heroRef = useRef<HTMLDivElement>(null)
  const aboutRef = useRef<HTMLDivElement>(null)
  const projectsRef = useRef<HTMLDivElement>(null)
  const skillsRef = useRef<HTMLDivElement>(null)
  const contactRef = useRef<HTMLDivElement>(null)
  const [activeSection, setActiveSection] = useState("hero")

  useEffect(() => {
    // Disable body scroll when mobile menu is open
    if (mobileMenuOpen) {
      document.body.style.overflow = "hidden"
    } else {
      document.body.style.overflow = ""
    }
    return () => {
      document.body.style.overflow = ""
    }
  }, [mobileMenuOpen])

  useEffect(() => {
    const sections = [
      { id: "hero", ref: heroRef },
      { id: "about", ref: aboutRef },
      { id: "projects", ref: projectsRef },
      { id: "skills", ref: skillsRef },
      { id: "contact", ref: contactRef },
    ]

    const handleScroll = () => {
      const scrollPosition = window.scrollY + window.innerHeight / 3

      for (const section of sections) {
        const element = section.ref.current
        if (!element) continue

        const { offsetTop, offsetHeight } = element
        if (scrollPosition >= offsetTop && scrollPosition < offsetTop + offsetHeight) {
          setActiveSection(section.id)
          break
        }
      }
    }

    window.addEventListener("scroll", handleScroll)
    return () => window.removeEventListener("scroll", handleScroll)
  }, [])

  const navItems = [
    { name: "About", id: "about", ref: aboutRef },
    { name: "Projects", id: "projects", ref: projectsRef },
    { name: "Skills", id: "skills", ref: skillsRef },
    { name: "Blog", id: "blog", href: "/blog" as const },
    { name: "Contact", id: "contact", ref: contactRef },
  ]

  return (
    <PageTransition>
      <div className="flex flex-col min-h-screen bg-darkgray text-white">
        {!isMobile && <Cursor />}
        <NoiseBackground />

        {/* Navigation */}
        <motion.header
          style={{ opacity: headerOpacity, backdropFilter: `blur(${headerBlur}px)` }}
          className="sticky top-0 z-50 w-full border-b border-white/10 bg-darkgray/60"
        >
          <div className="container flex h-16 items-center justify-between">
            <Link href="/" className="flex items-center gap-2 font-bold text-xl">
              <motion.span
                initial={{ opacity: 0, y: -20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5 }}
                className="text-teal"
              >
                KP
              </motion.span>
              <motion.span
                initial={{ opacity: 0, y: -20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.1 }}
                className="hidden sm:inline-block"
              >
                Koosha Paridehpour
              </motion.span>
            </Link>
            <nav className="hidden md:flex gap-6">
              {navItems.map((item, i) => (
                <motion.div
                  key={item.name}
                  initial={{ opacity: 0, y: -20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.5, delay: 0.1 + i * 0.1 }}
                >
                  {"href" in item ? (
                    <Link
                      href={item.href as string}
                      className="text-sm font-medium text-white/70 hover:text-teal transition-colors relative group"
                    >
                      {item.name}
                      <span className="absolute -bottom-1 left-0 h-[2px] bg-teal w-0 group-hover:w-full transition-all duration-300"></span>
                    </Link>
                  ) : (
                    <button
                      type="button"
                      onClick={() => {
                        item.ref.current?.scrollIntoView({ behavior: "smooth" })
                      }}
                      className={cn(
                        "text-sm font-medium transition-colors relative group",
                        activeSection === item.id ? "text-teal" : "text-white/70 hover:text-white",
                      )}
                    >
                      {item.name}
                      <span
                        className={cn(
                          "absolute -bottom-1 left-0 h-[2px] bg-teal transition-all duration-300",
                          activeSection === item.id ? "w-full" : "w-0 group-hover:w-full",
                        )}
                      ></span>
                    </button>
                  )}
                </motion.div>
              ))}
            </nav>
            <div className="flex items-center gap-4">
              <motion.div
                initial={{ opacity: 0, y: -20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.5 }}
              >
                <Link
                  href="/resume.pdf"
                  className="text-sm font-medium text-white/70 hover:text-white transition-colors hidden sm:block"
                  target="_blank"
                >
                  Resume
                </Link>
              </motion.div>
              <motion.div
                initial={{ opacity: 0, y: -20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.6 }}
              >
                <Button
                  onClick={() => contactRef.current?.scrollIntoView({ behavior: "smooth" })}
                  className="bg-teal text-darkgray hover:bg-teal/80"
                >
                  Get in Touch
                </Button>
              </motion.div>
              <motion.button
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.5, delay: 0.7 }}
                className="md:hidden text-white hover:text-teal transition-colors"
                onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              >
                {mobileMenuOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
              </motion.button>
            </div>
          </div>
        </motion.header>

        {/* Mobile Menu */}
        <AnimatePresence>
          {mobileMenuOpen && (
            <motion.div
              initial={{ opacity: 0, y: -20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.3 }}
              className="fixed inset-0 z-40 bg-darkgray pt-16 px-4"
            >
              <nav className="flex flex-col gap-6 py-8">
                {navItems.map((item, i) => (
                  <motion.div
                    key={item.name}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: i * 0.1 }}
                  >
                    {"href" in item ? (
                      <Link
                        href={item.href as string}
                        className="text-2xl font-medium hover:text-teal transition-colors"
                        onClick={() => setMobileMenuOpen(false)}
                      >
                        {item.name}
                      </Link>
                    ) : (
                      <button
                        type="button"
                        onClick={() => {
                          item.ref.current?.scrollIntoView({ behavior: "smooth" })
                          setMobileMenuOpen(false)
                        }}
                        className="text-2xl font-medium hover:text-teal transition-colors"
                      >
                        {item.name}
                      </button>
                    )}
                  </motion.div>
                ))}
                <motion.div
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.3, delay: 0.4 }}
                >
                  <Link
                    href="/resume.pdf"
                    className="text-2xl font-medium hover:text-teal transition-colors"
                    target="_blank"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    Resume
                  </Link>
                </motion.div>
              </nav>
              <div className="absolute bottom-8 left-4 right-4">
                <div className="flex justify-between">
                  <Link
                    href="https://github.com/kooshapari"
                    target="_blank"
                    className="text-white hover:text-teal transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <Github className="h-6 w-6" />
                  </Link>
                  <Link
                    href="https://linkedin.com/in/kooshapari"
                    target="_blank"
                    className="text-white hover:text-teal transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <Linkedin className="h-6 w-6" />
                  </Link>
                  <Link
                    href="mailto:contact@kooshapari.com"
                    className="text-white hover:text-teal transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <Mail className="h-6 w-6" />
                  </Link>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        <main className="flex-1">
          {/* Hero Section */}
          <section ref={heroRef} className="relative min-h-[100vh] flex items-center overflow-hidden">
            <FloatingParticles />

            <div className="container relative z-10 py-24 md:py-32 space-y-8">
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 1 }}
                className="absolute top-0 left-0 w-full h-full bg-gradient-to-r from-darkgray via-darkgray/90 to-transparent pointer-events-none"
              />

              <div className="flex flex-col md:flex-row gap-8 md:gap-16 items-center">
                <div className="space-y-6 md:w-2/3">
                  <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.6, delay: 0.2 }}
                  >
                    <Badge className="px-3 py-1 text-sm rounded-md bg-teal/20 text-teal border-teal/30">
                      Available for opportunities
                    </Badge>
                  </motion.div>

                  <motion.div
                    initial={{ opacity: 0, y: 30 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.6, delay: 0.3 }}
                  >
                    <h1 className="text-4xl md:text-6xl font-bold tracking-tight">
                      <GradientText>
                        <SplitText
                          text="Software Engineer"
                          className="inline-block"
                          charClassName="inline-block"
                          animation={{
                            hidden: { opacity: 0, y: 20 },
                            visible: (i: number) => ({
                              opacity: 1,
                              y: 0,
                              transition: { duration: 0.2, delay: 0.4 + i * 0.05 },
                            }),
                          }}
                        />
                      </GradientText>
                      <span className="text-teal"> & </span>
                      <GradientText>
                        <SplitText
                          text="Designer"
                          className="inline-block"
                          charClassName="inline-block"
                          animation={{
                            hidden: { opacity: 0, y: 20 },
                            visible: (i: number) => ({
                              opacity: 1,
                              y: 0,
                              transition: { duration: 0.2, delay: 0.8 + i * 0.05 },
                            }),
                          }}
                        />
                      </GradientText>
                    </h1>
                  </motion.div>

                  <motion.p
                    initial={{ opacity: 0, y: 30 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.6, delay: 0.5 }}
                    className="text-xl text-white/80"
                  >
                    Building scalable software with creative design insight. Where coding meets design – crafting
                    products from concept to launch.
                  </motion.p>

                  <motion.div
                    initial={{ opacity: 0, y: 30 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.6, delay: 0.6 }}
                    className="flex flex-col sm:flex-row gap-3"
                  >
                    <Button
                      onClick={() => projectsRef.current?.scrollIntoView({ behavior: "smooth" })}
                      size="lg"
                      className="bg-teal text-darkgray hover:bg-teal/80 group"
                    >
                      View My Work{" "}
                      <ArrowRight className="ml-2 h-4 w-4 transition-transform duration-300 group-hover:translate-x-1" />
                    </Button>
                    <Button
                      variant="outline"
                      size="lg"
                      asChild
                      className="border-white/20 hover:bg-white/10 hover:border-white/30"
                    >
                      <Link href="/resume.pdf" target="_blank">
                        Download Resume
                      </Link>
                    </Button>
                  </motion.div>
                </div>

                <motion.div
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ duration: 0.8, delay: 0.7 }}
                  className="md:w-1/3 relative"
                >
                  <div className="w-[280px] h-[280px] md:w-[320px] md:h-[320px] relative">
                    <KeyboardModel />
                  </div>
                </motion.div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-6 pt-8">
                {[
                  {
                    title: "CS Graduate",
                    description:
                      "Computer Science student at Arizona State University, Barrett Honors College, class of 2025.",
                    delay: 0.7,
                  },
                  {
                    title: "Product Manager",
                    description:
                      "Leading design and development at Phenotype, and incoming Development Engineer intern at CVS Health.",
                    delay: 0.8,
                  },
                  {
                    title: "Designer",
                    description:
                      "Creator of multiple successful mechanical keyboard projects including GMK Arch and DSS Cipher.",
                    delay: 0.9,
                  },
                ].map((item, i) => (
                  <motion.div
                    key={i}
                    initial={{ opacity: 0, y: 30 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.6, delay: item.delay }}
                    className="p-6 border border-white/10 rounded-lg bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-colors"
                  >
                    <h3 className="font-semibold text-lg mb-2 text-teal">{item.title}</h3>
                    <p className="text-white/70">{item.description}</p>
                  </motion.div>
                ))}
              </div>
            </div>

            <div className="absolute bottom-8 left-1/2 -translate-x-1/2 animate-bounce">
              <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.6, delay: 1.2 }}>
                <ArrowRight className="h-6 w-6 text-teal rotate-90" />
              </motion.div>
            </div>
          </section>

          {/* About Section */}
          <ParallaxSection
            ref={aboutRef}
            className="py-16 md:py-24 relative overflow-hidden"
            style={{ background: "linear-gradient(to bottom, #1f2022, #1a1b1d)" }}
          >
            <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-teal/50 to-transparent" />
            <div className="absolute -top-24 -right-24 w-64 h-64 bg-teal/10 rounded-full blur-3xl" />
            <div className="absolute -bottom-32 -left-32 w-96 h-96 bg-teal/5 rounded-full blur-3xl" />

            <div className="container relative">
              <div className="flex flex-col items-center mb-12">
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6 }}
                >
                  <Badge className="px-3 py-1 text-sm rounded-md bg-teal/20 text-teal border-teal/30 mb-4">About</Badge>
                </motion.div>
                <motion.h2
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.1 }}
                  className="text-3xl font-bold mb-2 text-center"
                >
                  <GradientText>About Me</GradientText>
                </motion.h2>
                <motion.div
                  initial={{ scaleX: 0 }}
                  whileInView={{ scaleX: 1 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.2 }}
                  className="w-24 h-1 bg-teal"
                />
              </div>

              <div className="grid md:grid-cols-2 gap-12 items-center">
                <div className="space-y-6">
                  {[
                    "Hi, I'm Koosha. I'm a Computer Science student at Arizona State University's Barrett Honors College, class of 2025. I've always been fascinated by the intersection of technology and design.",
                    "My journey began with graphic design projects in high school, which evolved into a passion for both software development and product design. This unique combination has allowed me to create solutions that are not only functional but also intuitive and visually engaging.",
                    "Currently, I serve as a Product Manager at Phenotype, where I lead the development of custom mechanical keyboard products. I'm also excited to be joining CVS Health as a Development Engineer intern, where I'll be applying my technical skills in a corporate environment.",
                    "When I'm not coding or designing, you can find me tinkering with mechanical keyboards or exploring new design trends. This blend of hobbies keeps my problem-solving skills sharp and my creativity fresh.",
                  ].map((paragraph, i) => (
                    <motion.p
                      key={i}
                      initial={{ opacity: 0, y: 20 }}
                      whileInView={{ opacity: 1, y: 0 }}
                      viewport={{ once: true, margin: "-100px" }}
                      transition={{ duration: 0.6, delay: 0.3 + i * 0.1 }}
                      className={cn("text-white/80", i === 0 && "text-lg")}
                    >
                      {paragraph}
                    </motion.p>
                  ))}
                </div>

                <div className="space-y-8">
                  <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "-100px" }}
                    transition={{ duration: 0.6, delay: 0.3 }}
                    className="border border-white/10 rounded-lg p-6 bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-colors"
                  >
                    <h3 className="font-semibold text-xl mb-4 text-teal">Education</h3>
                    <div className="space-y-3">
                      <div>
                        <h4 className="font-medium">Arizona State University</h4>
                        <p className="text-white/70">B.S. Computer Science, Barrett Honors College</p>
                        <p className="text-sm text-white/50">2021 - 2025 (Expected)</p>
                      </div>
                    </div>
                  </motion.div>

                  <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "-100px" }}
                    transition={{ duration: 0.6, delay: 0.4 }}
                    className="border border-white/10 rounded-lg p-6 bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-colors"
                  >
                    <h3 className="font-semibold text-xl mb-4 text-teal">Experience</h3>
                    <div className="space-y-4">
                      <div>
                        <h4 className="font-medium">CVS Health</h4>
                        <p className="text-white/70">Development Engineer Intern</p>
                        <p className="text-sm text-white/50">Summer 2025</p>
                      </div>
                      <div>
                        <h4 className="font-medium">Phenotype</h4>
                        <p className="text-white/70">Product Manager</p>
                        <p className="text-sm text-white/50">2023 - Present</p>
                      </div>
                    </div>
                  </motion.div>
                </div>
              </div>
            </div>
          </ParallaxSection>

          {/* Projects Section */}
          <ParallaxSection
            ref={projectsRef}
            className="py-16 md:py-24 relative overflow-hidden"
            style={{ background: "linear-gradient(to bottom, #1a1b1d, #1f2022)" }}
          >
            <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-teal/50 to-transparent" />
            <div className="absolute -top-32 -left-32 w-96 h-96 bg-teal/5 rounded-full blur-3xl" />
            <div className="absolute -bottom-24 -right-24 w-64 h-64 bg-teal/10 rounded-full blur-3xl" />

            <div className="container relative">
              <div className="flex flex-col items-center mb-12">
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6 }}
                >
                  <Badge className="px-3 py-1 text-sm rounded-md bg-teal/20 text-teal border-teal/30 mb-4">
                    Portfolio
                  </Badge>
                </motion.div>
                <motion.h2
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.1 }}
                  className="text-3xl font-bold mb-2 text-center"
                >
                  <GradientText>Featured Projects</GradientText>
                </motion.h2>
                <motion.div
                  initial={{ scaleX: 0 }}
                  whileInView={{ scaleX: 1 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.2 }}
                  className="w-24 h-1 bg-teal"
                />
                <motion.p
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.3 }}
                  className="text-center text-white/70 mt-4 max-w-2xl"
                >
                  A selection of my work across software development and design. Each project represents a unique
                  challenge and solution.
                </motion.p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <ProjectCard
                  title="GMK Arch"
                  category="Product Design"
                  description="A custom mechanical keyboard keycap set inspired by Arch Linux, focusing on minimalism and functionality. Successfully funded and produced through group buy."
                  image="/blue-gray-keycaps.png"
                  link="/projects/gmk-arch"
                  index={0}
                />
                <ProjectCard
                  title="WITF Board"
                  category="Hardware Design"
                  description="Custom mechanical keyboard with seamless aluminum case and custom PCB. Managed the entire process from concept to group buy."
                  image="/custom-aluminum-keyboard.png"
                  link="/projects/witf-board"
                  index={1}
                />
                <ProjectCard
                  title="Phenotype Platform"
                  category="Web Development"
                  description="E-commerce platform for custom mechanical keyboards and accessories. Built with React, Next.js, and integrated payment processing."
                  image="/mechanical-keyboard-storefront.png"
                  link="/projects/phenotype"
                  index={2}
                />
                <ProjectCard
                  title="DSS Cipher"
                  category="Product Design"
                  description="Keycap set with a cryptography theme. Designed color scheme, legends, and packaging. Coordinated with manufacturers for production."
                  image="/crypto-keys.png"
                  link="/projects/dss-cipher"
                  index={3}
                />
              </div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: "-100px" }}
                transition={{ duration: 0.6, delay: 0.6 }}
                className="mt-12 text-center"
              >
                <Button
                  asChild
                  variant="outline"
                  size="lg"
                  className="border-white/20 hover:bg-white/10 hover:border-white/30 group"
                >
                  <Link href="/projects">
                    View All Projects{" "}
                    <ArrowRight className="ml-2 h-4 w-4 transition-transform duration-300 group-hover:translate-x-1" />
                  </Link>
                </Button>
              </motion.div>
            </div>
          </ParallaxSection>

          {/* Skills Section */}
          <ParallaxSection
            ref={skillsRef}
            className="py-16 md:py-24 relative overflow-hidden"
            style={{ background: "linear-gradient(to bottom, #1f2022, #1a1b1d)" }}
          >
            <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-teal/50 to-transparent" />
            <div className="absolute -top-24 -right-24 w-64 h-64 bg-teal/10 rounded-full blur-3xl" />
            <div className="absolute -bottom-32 -left-32 w-96 h-96 bg-teal/5 rounded-full blur-3xl" />

            <div className="container relative">
              <div className="flex flex-col items-center mb-12">
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6 }}
                >
                  <Badge className="px-3 py-1 text-sm rounded-md bg-teal/20 text-teal border-teal/30 mb-4">
                    Expertise
                  </Badge>
                </motion.div>
                <motion.h2
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.1 }}
                  className="text-3xl font-bold mb-2 text-center"
                >
                  <GradientText>Skills & Tools</GradientText>
                </motion.h2>
                <motion.div
                  initial={{ scaleX: 0 }}
                  whileInView={{ scaleX: 1 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.2 }}
                  className="w-24 h-1 bg-teal"
                />
                <motion.p
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.3 }}
                  className="text-center text-white/70 mt-4 max-w-2xl"
                >
                  My technical toolkit spans both software development and design, allowing me to create end-to-end
                  solutions.
                </motion.p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.4 }}
                  className="border border-white/10 rounded-lg p-6 bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-colors"
                >
                  <h3 className="font-semibold text-xl mb-6 text-teal">Software Development</h3>
                  <div className="space-y-6">
                    <SkillBar name="JavaScript/TypeScript" percentage={90} delay={0.1} />
                    <SkillBar name="React & Next.js" percentage={85} delay={0.2} />
                    <SkillBar name="Python" percentage={80} delay={0.3} />
                    <SkillBar name="Node.js" percentage={75} delay={0.4} />
                    <SkillBar name="Java" percentage={70} delay={0.5} />
                  </div>
                </motion.div>

                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.5 }}
                  className="border border-white/10 rounded-lg p-6 bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-colors"
                >
                  <h3 className="font-semibold text-xl mb-6 text-teal">Design</h3>
                  <div className="space-y-6">
                    <SkillBar name="UI/UX Design" percentage={88} delay={0.1} />
                    <SkillBar name="Figma" percentage={92} delay={0.2} />
                    <SkillBar name="Adobe Photoshop" percentage={85} delay={0.3} />
                    <SkillBar name="Adobe Illustrator" percentage={78} delay={0.4} />
                    <SkillBar name="Product Design" percentage={82} delay={0.5} />
                  </div>
                </motion.div>
              </div>
            </div>
          </ParallaxSection>

          {/* Accomplishments Section */}
          <ParallaxSection
            className="py-16 md:py-24 relative overflow-hidden"
            style={{ background: "linear-gradient(to bottom, #1a1b1d, #1f2022)" }}
          >
            <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-teal/50 to-transparent" />
            <div className="absolute -top-32 -left-32 w-96 h-96 bg-teal/5 rounded-full blur-3xl" />
            <div className="absolute -bottom-24 -right-24 w-64 h-64 bg-teal/10 rounded-full blur-3xl" />

            <div className="container relative">
              <div className="flex flex-col items-center mb-12">
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6 }}
                >
                  <Badge className="px-3 py-1 text-sm rounded-md bg-teal/20 text-teal border-teal/30 mb-4">
                    Achievements
                  </Badge>
                </motion.div>
                <motion.h2
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.1 }}
                  className="text-3xl font-bold mb-2 text-center"
                >
                  <GradientText>Accomplishments</GradientText>
                </motion.h2>
                <motion.div
                  initial={{ scaleX: 0 }}
                  whileInView={{ scaleX: 1 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.2 }}
                  className="w-24 h-1 bg-teal"
                />
                <motion.p
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.3 }}
                  className="text-center text-white/70 mt-4 max-w-2xl"
                >
                  Key milestones and achievements throughout my academic and professional journey.
                </motion.p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                {[
                  {
                    title: "GMK Arch",
                    description:
                      "Successfully designed, crowdfunded, and produced a custom keycap set sold through international vendors.",
                    delay: 0.4,
                  },
                  {
                    title: "Phenotype Leadership",
                    description:
                      "Led a team of 5 in developing the Phenotype e-commerce platform for custom mechanical keyboards.",
                    delay: 0.5,
                  },
                  {
                    title: "CVS Health Internship",
                    description: "Selected for competitive Development Engineer internship at Fortune 500 company.",
                    delay: 0.6,
                  },
                ].map((item, i) => (
                  <motion.div
                    key={i}
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "-100px" }}
                    transition={{ duration: 0.6, delay: item.delay }}
                    className="border border-white/10 rounded-lg p-6 bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-colors"
                  >
                    <h3 className="font-semibold text-lg mb-3 text-teal">{item.title}</h3>
                    <p className="text-white/70">{item.description}</p>
                  </motion.div>
                ))}
              </div>
            </div>
          </ParallaxSection>

          {/* Contact Section */}
          <ParallaxSection
            ref={contactRef}
            className="py-16 md:py-24 relative overflow-hidden"
            style={{ background: "linear-gradient(to bottom, #1f2022, #1a1b1d)" }}
          >
            <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-teal/50 to-transparent" />
            <div className="absolute -top-24 -right-24 w-64 h-64 bg-teal/10 rounded-full blur-3xl" />
            <div className="absolute -bottom-32 -left-32 w-96 h-96 bg-teal/5 rounded-full blur-3xl" />

            <div className="container relative">
              <div className="flex flex-col items-center mb-12">
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6 }}
                >
                  <Badge className="px-3 py-1 text-sm rounded-md bg-teal/20 text-teal border-teal/30 mb-4">
                    Contact
                  </Badge>
                </motion.div>
                <motion.h2
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.1 }}
                  className="text-3xl font-bold mb-2 text-center"
                >
                  <GradientText>Get in Touch</GradientText>
                </motion.h2>
                <motion.div
                  initial={{ scaleX: 0 }}
                  whileInView={{ scaleX: 1 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.2 }}
                  className="w-24 h-1 bg-teal"
                />
                <motion.p
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.3 }}
                  className="text-center text-white/70 mt-4 max-w-2xl"
                >
                  I'd love to hear from you! Feel free to reach out with opportunities, ideas, or just to connect.
                </motion.p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-12">
                <motion.div
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.4 }}
                  className="space-y-6"
                >
                  <h3 className="text-xl font-semibold text-teal">Contact Information</h3>
                  <div className="space-y-4">
                    <div className="flex items-center gap-3 group">
                      <div className="w-10 h-10 rounded-full bg-teal/20 flex items-center justify-center group-hover:bg-teal/30 transition-colors">
                        <Mail className="h-5 w-5 text-teal" />
                      </div>
                      <a
                        href="mailto:contact@kooshapari.com"
                        className="hover:text-teal transition-colors group-hover:translate-x-1 transition-transform duration-300"
                      >
                        contact@kooshapari.com
                      </a>
                    </div>
                    <div className="flex items-center gap-3 group">
                      <div className="w-10 h-10 rounded-full bg-teal/20 flex items-center justify-center group-hover:bg-teal/30 transition-colors">
                        <Linkedin className="h-5 w-5 text-teal" />
                      </div>
                      <a
                        href="https://linkedin.com/in/kooshapari"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="hover:text-teal transition-colors group-hover:translate-x-1 transition-transform duration-300"
                      >
                        linkedin.com/in/kooshapari
                      </a>
                    </div>
                    <div className="flex items-center gap-3 group">
                      <div className="w-10 h-10 rounded-full bg-teal/20 flex items-center justify-center group-hover:bg-teal/30 transition-colors">
                        <Github className="h-5 w-5 text-teal" />
                      </div>
                      <a
                        href="https://github.com/kooshapari"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="hover:text-teal transition-colors group-hover:translate-x-1 transition-transform duration-300"
                      >
                        github.com/kooshapari
                      </a>
                    </div>
                  </div>
                  <div className="pt-4">
                    <p className="text-white/70">Based in Tempe, AZ</p>
                    <p className="text-white/70 mt-2">
                      Currently open to full-time software engineering roles for Fall 2025
                    </p>
                  </div>
                </motion.div>

                <motion.div
                  initial={{ opacity: 0, x: 20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true, margin: "-100px" }}
                  transition={{ duration: 0.6, delay: 0.5 }}
                  className="border border-white/10 rounded-lg p-6 bg-white/5 backdrop-blur-sm"
                >
                  <form className="space-y-4">
                    <div className="grid grid-cols-1 gap-4">
                      <div className="space-y-2">
                        <label htmlFor="name" className="text-sm font-medium text-white/80">
                          Name
                        </label>
                        <input
                          id="name"
                          className="flex h-10 w-full rounded-md border border-white/10 bg-white/5 px-3 py-2 text-sm text-white ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-white/40 focus:border-teal focus:outline-none focus:ring-1 focus:ring-teal/50 disabled:cursor-not-allowed disabled:opacity-50"
                          placeholder="Your name"
                        />
                      </div>
                      <div className="space-y-2">
                        <label htmlFor="email" className="text-sm font-medium text-white/80">
                          Email
                        </label>
                        <input
                          id="email"
                          type="email"
                          className="flex h-10 w-full rounded-md border border-white/10 bg-white/5 px-3 py-2 text-sm text-white ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-white/40 focus:border-teal focus:outline-none focus:ring-1 focus:ring-teal/50 disabled:cursor-not-allowed disabled:opacity-50"
                          placeholder="Your email"
                        />
                      </div>
                      <div className="space-y-2">
                        <label htmlFor="message" className="text-sm font-medium text-white/80">
                          Message
                        </label>
                        <textarea
                          id="message"
                          className="flex min-h-[120px] w-full rounded-md border border-white/10 bg-white/5 px-3 py-2 text-sm text-white ring-offset-background placeholder:text-white/40 focus:border-teal focus:outline-none focus:ring-1 focus:ring-teal/50 disabled:cursor-not-allowed disabled:opacity-50"
                          placeholder="Your message"
                        />
                      </div>
                    </div>
                    <Button className="w-full bg-teal text-darkgray hover:bg-teal/80">Send Message</Button>
                  </form>
                </motion.div>
              </div>
            </div>
          </ParallaxSection>
        </main>

        {/* Footer */}
        <footer className="border-t border-white/10 py-6 md:py-8 bg-darkgray">
          <div className="container flex flex-col md:flex-row justify-between items-center gap-4">
            <div className="flex items-center gap-2">
              <span className="text-sm text-white/50">© 2025 Koosha Paridehpour. All rights reserved.</span>
            </div>
            <div className="flex items-center gap-4">
              <a
                href="https://linkedin.com/in/kooshapari"
                target="_blank"
                rel="noopener noreferrer"
                className="text-white/50 hover:text-teal transition-colors"
              >
                <Linkedin className="h-5 w-5" />
                <span className="sr-only">LinkedIn</span>
              </a>
              <a
                href="https://github.com/kooshapari"
                target="_blank"
                rel="noopener noreferrer"
                className="text-white/50 hover:text-teal transition-colors"
              >
                <Github className="h-5 w-5" />
                <span className="sr-only">GitHub</span>
              </a>
              <a href="mailto:contact@kooshapari.com" className="text-white/50 hover:text-teal transition-colors">
                <Mail className="h-5 w-5" />
                <span className="sr-only">Email</span>
              </a>
            </div>
            <div className="text-xs text-white/50">Site last updated: April 2025</div>
          </div>
        </footer>
      </div>
    </PageTransition>
  )
}
