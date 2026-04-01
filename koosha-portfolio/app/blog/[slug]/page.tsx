import Link from "next/link"
import { ArrowLeft, Calendar } from "lucide-react"
import { MDXRemote } from "next-mdx-remote/rsc"
import { notFound } from "next/navigation"
import { getPostBySlug, getAllPosts, type PostData } from "@/lib/posts"

export function generateStaticParams() {
  const posts = getAllPosts()
  return posts.map((post) => ({ slug: post.slug }))
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params
  const post = getPostBySlug(slug)
  if (!post) return { title: "Post Not Found" }

  return {
    title: `${post.title} | Koosha Paridehpour`,
    description: post.excerpt,
  }
}

export default async function BlogPostPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params

  let post: PostData
  try {
    post = getPostBySlug(slug)
  } catch {
    notFound()
  }

  return (
    <div className="min-h-screen bg-darkgray text-white">
      <div className="container mx-auto px-4 py-16 max-w-3xl">
        <Link
          href="/blog"
          className="inline-flex items-center gap-2 text-white/60 hover:text-teal transition-colors mb-12"
        >
          <ArrowLeft className="h-4 w-4" />
          <span className="text-sm">Back to blog</span>
        </Link>

        <article>
          <header className="mb-10">
            <div className="flex items-center gap-2 text-white/50 text-sm mb-4">
              <Calendar className="h-4 w-4" />
              <time dateTime={post.date}>
                {new Date(post.date).toLocaleDateString("en-US", {
                  year: "numeric",
                  month: "long",
                  day: "numeric",
                })}
              </time>
            </div>
            <h1 className="text-3xl md:text-4xl font-bold text-teal mb-4">{post.title}</h1>
            <p className="text-white/70 text-lg">{post.excerpt}</p>
          </header>

          <div className="prose prose-invert prose-lg max-w-none prose-headings:text-teal prose-a:text-teal prose-a:no-underline hover:prose-a:underline prose-strong:text-white prose-code:text-teal prose-code:bg-white/5 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-pre:bg-white/5 prose-pre:border prose-pre:border-white/10">
            <MDXRemote source={post.content} />
          </div>
        </article>

        <div className="mt-16 pt-8 border-t border-white/10">
          <Link
            href="/blog"
            className="inline-flex items-center gap-2 text-teal hover:text-teal/80 transition-colors"
          >
            <ArrowLeft className="h-4 w-4" />
            Back to all posts
          </Link>
        </div>
      </div>
    </div>
  )
}
