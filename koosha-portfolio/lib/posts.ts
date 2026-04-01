import fs from "fs"
import path from "path"
import matter from "gray-matter"

const postsDirectory = path.join(process.cwd(), "content/blog")

export interface PostMetadata {
  slug: string
  title: string
  date: string
  excerpt: string
}

export interface PostData extends PostMetadata {
  content: string
}

export function getAllPosts(): PostMetadata[] {
  const fileNames = fs.readdirSync(postsDirectory)
  const posts = fileNames
    .filter((fileName) => fileName.endsWith(".mdx"))
    .map((fileName) => {
      const slug = fileName.replace(/\.mdx$/, "")
      const fullPath = path.join(postsDirectory, fileName)
      const fileContents = fs.readFileSync(fullPath, "utf8")
      const { data } = matter(fileContents)

      return {
        slug,
        title: data.title as string,
        date: data.date as string,
        excerpt: data.excerpt as string,
      }
    })

  return posts.sort((a, b) => (a.date < b.date ? 1 : -1))
}

export function getPostBySlug(slug: string): PostData {
  const fullPath = path.join(postsDirectory, `${slug}.mdx`)
  const fileContents = fs.readFileSync(fullPath, "utf8")
  const { data, content } = matter(fileContents)

  return {
    slug,
    title: data.title as string,
    date: data.date as string,
    excerpt: data.excerpt as string,
    content,
  }
}
