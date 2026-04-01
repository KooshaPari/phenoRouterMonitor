import Link from "next/link"
import { ArrowLeft, Calendar } from "lucide-react"
import { getAllPosts } from "@/lib/posts"

export const metadata = {
  title: "Blog | Koosha Paridehpour",
  description: "Thoughts on software engineering, design, and building Phenotype.",
}

export default function BlogPage() {
  const posts = getAllPosts()

  return (
    <div className="min-h-screen bg-darkgray text-white">
      <div className="container mx-auto px-4 py-16 max-w-4xl">
        <div className="mb-12">
          <Link
            href="/#about"
            className="inline-flex items-center gap-2 text-white/60 hover:text-teal transition-colors mb-8"
          >
            <ArrowLeft className="h-4 w-4" />
            <span className="text-sm">Back to home</span>
          </Link>
          <h1 className="text-4xl md:text-5xl font-bold mb-4">
            <span className="text-teal">Blog</span>
          </h1>
          <p className="text-white/70 text-lg max-w-2xl">
            Thoughts on software engineering, design, and building a one-person software organization.
          </p>
        </div>

        <div className="grid gap-6">
          {posts.map((post, i) => (
            <article
              key={post.slug}
              className="group border border-white/10 rounded-lg p-6 bg-white/5 backdrop-blur-sm hover:bg-white/10 hover:border-teal/30 transition-all duration-300"
            >
              <Link href={`/blog/${post.slug}`}>
                <div className="flex items-center gap-2 text-white/50 text-sm mb-3">
                  <Calendar className="h-4 w-4" />
                  <time dateTime={post.date}>
                    {new Date(post.date).toLocaleDateString("en-US", {
                      year: "numeric",
                      month: "long",
                      day: "numeric",
                    })}
                  </time>
                </div>
                <h2 className="text-xl font-semibold mb-2 group-hover:text-teal transition-colors">
                  {post.title}
                </h2>
                <p className="text-white/70 mb-4">{post.excerpt}</p>
                <span className="inline-flex items-center gap-1 text-teal text-sm font-medium group-hover:gap-2 transition-all">
                  Read more
                  <ArrowLeft className="h-3 w-3 rotate-180" />
                </span>
              </Link>
            </article>
          ))}
        </div>

        {posts.length === 0 && (
          <div className="text-center py-16">
            <p className="text-white/50 text-lg">No posts yet. Check back soon.</p>
          </div>
        )}
      </div>
    </div>
  )
}
