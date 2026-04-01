"use client"

import { useRef, useState, Suspense, useEffect } from "react"
import { Canvas, useFrame, useThree } from "@react-three/fiber"
import { PresentationControls, Environment, ContactShadows } from "@react-three/drei"
import type * as THREE from "three"
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js"

function KeyboardModel() {
  return (
    <div className="w-full h-full">
      <Canvas camera={{ position: [0, 0, 4], fov: 45 }}>
        <ambientLight intensity={0.5} />
        <spotLight position={[10, 10, 10]} angle={0.15} penumbra={1} intensity={1} castShadow />
        <PresentationControls
          global
          snap
          rotation={[0, 0.3, 0]}
          polar={[-Math.PI / 3, Math.PI / 3]}
          azimuth={[-Math.PI / 1.4, Math.PI / 2]}
        >
          <Suspense fallback={<LoadingFallback />}>
            <CustomGLBModel url="/keyboard.glb" />
          </Suspense>
        </PresentationControls>
        <ContactShadows position={[0, -1.4, 0]} opacity={0.75} scale={10} blur={2.5} far={4} />
        <Environment preset="city" />
      </Canvas>
    </div>
  )
}

function LoadingFallback() {
  return (
    <mesh>
      <boxGeometry args={[2, 0.5, 1]} />
      <meshStandardMaterial color="#7ebab5" wireframe />
    </mesh>
  )
}

function CustomGLBModel({ url }: { url: string }) {
  const keyboard = useRef<THREE.Group>(null)
  const { viewport } = useThree()
  const [model, setModel] = useState<THREE.Group | null>(null)
  const [loadError, setLoadError] = useState(false)

  useEffect(() => {
    const loader = new GLTFLoader()

    loader.load(
      url,
      (gltf) => {
        setModel(gltf.scene)
      },
      undefined,
      (error) => {
        console.error("Error loading GLB model:", error)
        setLoadError(true)
      },
    )
  }, [url])

  useFrame((state) => {
    if (keyboard.current) {
      keyboard.current.rotation.y = Math.sin(state.clock.getElapsedTime() / 4) * 0.3
      keyboard.current.rotation.x = Math.sin(state.clock.getElapsedTime() / 4) * 0.1
    }
  })

  if (loadError) {
    return <SimpleKeyboardModel />
  }

  if (!model) {
    return <LoadingFallback />
  }

  const scale = viewport.width > 768 ? 0.02 : 0.015

  return (
    <group ref={keyboard} scale={scale} position={[0, -0.5, 0]} rotation={[0.2, 0, 0]}>
      <primitive object={model} />
    </group>
  )
}

const KEY_COLORS = ["#7ebab5", "#6aa8a3", "#95ccc8", "#569691"]

function keyId(row: string, col: number) {
  return `${row}-col-${col}`
}

function SimpleKeyboardModel() {
  const keyboard = useRef<THREE.Group>(null)
  const { viewport } = useThree()

  useFrame((state) => {
    if (keyboard.current) {
      keyboard.current.rotation.y = Math.sin(state.clock.getElapsedTime() / 4) * 0.3
      keyboard.current.rotation.x = Math.sin(state.clock.getElapsedTime() / 4) * 0.1
    }
  })

  const scale = viewport.width > 768 ? 0.7 : 0.5

  return (
    <group ref={keyboard} scale={scale} position={[0, -0.5, 0]}>
      <mesh castShadow receiveShadow position={[0, 0, 0]}>
        <boxGeometry args={[3, 0.2, 1.5]} />
        <meshStandardMaterial color="#1f2022" metalness={0.7} roughness={0.2} />
      </mesh>

      {Array.from({ length: 12 }, (_, i) => (
        <mesh key={keyId("r1", i)} castShadow position={[-1.65 + i * 0.3, 0.15, -0.6]}>
          <boxGeometry args={[0.25, 0.1, 0.25]} />
          <meshStandardMaterial color={KEY_COLORS[i % KEY_COLORS.length]} />
        </mesh>
      ))}

      {Array.from({ length: 11 }, (_, i) => (
        <mesh key={keyId("r2", i)} castShadow position={[-1.5 + i * 0.3, 0.15, -0.3]}>
          <boxGeometry args={[0.25, 0.1, 0.25]} />
          <meshStandardMaterial color={KEY_COLORS[i % KEY_COLORS.length]} />
        </mesh>
      ))}

      {Array.from({ length: 10 }, (_, i) => (
        <mesh key={keyId("r3", i)} castShadow position={[-1.35 + i * 0.3, 0.15, 0]}>
          <boxGeometry args={[0.25, 0.1, 0.25]} />
          <meshStandardMaterial color={KEY_COLORS[i % KEY_COLORS.length]} />
        </mesh>
      ))}

      {Array.from({ length: 9 }, (_, i) => (
        <mesh key={keyId("r4", i)} castShadow position={[-1.2 + i * 0.3, 0.15, 0.3]}>
          <boxGeometry args={[0.25, 0.1, 0.25]} />
          <meshStandardMaterial color={KEY_COLORS[i % KEY_COLORS.length]} />
        </mesh>
      ))}

      <mesh castShadow position={[0, 0.15, 0.6]}>
        <boxGeometry args={[2, 0.1, 0.25]} />
        <meshStandardMaterial color="#7ebab5" />
      </mesh>
    </group>
  )
}

export { KeyboardModel }
