import { FileText } from "lucide-react";
import ReactMarkdown from "react-markdown";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

interface ReadmeProps {
  content: string | null;
  packageName: string;
  version: string;
}

export function Readme({ content, packageName, version }: ReadmeProps) {
  if (!content || content.trim() === "") {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            README
          </CardTitle>
          <CardDescription>
            Documentation for {packageName} v{version}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex h-32 items-center justify-center">
            <p className="text-muted-foreground text-sm">No README available for this version</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FileText className="h-5 w-5" />
          README
        </CardTitle>
        <CardDescription>
          Documentation for {packageName} v{version}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="prose prose-sm dark:prose-invert max-w-none">
          <ReactMarkdown
            components={{
              // Customize heading styles to match the design system
              h1: ({ children }) => <h1 className="mb-4 font-bold text-2xl">{children}</h1>,
              h2: ({ children }) => <h2 className="mt-6 mb-3 font-semibold text-xl">{children}</h2>,
              h3: ({ children }) => <h3 className="mt-4 mb-2 font-semibold text-lg">{children}</h3>,
              h4: ({ children }) => <h4 className="mt-3 mb-2 font-semibold text-base">{children}</h4>,
              h5: ({ children }) => <h5 className="mt-2 mb-1 font-semibold text-sm">{children}</h5>,
              h6: ({ children }) => <h6 className="mt-2 mb-1 font-semibold text-xs">{children}</h6>,
              // Style paragraphs
              p: ({ children }) => <p className="mb-4 text-sm leading-relaxed">{children}</p>,
              // Style code blocks
              code: ({ children, className }) => {
                const isInline = !className;
                if (isInline) {
                  return <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{children}</code>;
                }
                return (
                  <code className="block overflow-x-auto rounded-md bg-muted p-4 font-mono text-xs">{children}</code>
                );
              },
              // Style pre blocks
              pre: ({ children }) => <pre className="mb-4 overflow-x-auto rounded-md bg-muted p-4">{children}</pre>,
              // Style lists
              ul: ({ children }) => <ul className="mb-4 ml-6 list-disc space-y-1">{children}</ul>,
              ol: ({ children }) => <ol className="mb-4 ml-6 list-decimal space-y-1">{children}</ol>,
              li: ({ children }) => <li className="text-sm">{children}</li>,
              // Style links
              a: ({ children, href }) => (
                <a
                  href={href}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary underline hover:no-underline"
                >
                  {children}
                </a>
              ),
              // Style blockquotes
              blockquote: ({ children }) => (
                <blockquote className="mb-4 border-muted-foreground/20 border-l-4 pl-4 text-muted-foreground italic">
                  {children}
                </blockquote>
              ),
              // Style images (including badges)
              img: ({ src, alt }) => (
                <img
                  src={src}
                  alt={alt || ""}
                  className="mr-1 inline-block max-h-6 max-w-full align-middle"
                  style={{ display: "inline-block" }}
                  loading="lazy"
                  onError={(e) => {
                    // Hide broken images gracefully
                    const target = e.target as HTMLImageElement;
                    target.style.display = "none";
                  }}
                />
              ),
              // Style tables
              table: ({ children }) => (
                <div className="mb-4 overflow-x-auto">
                  <table className="w-full border-collapse border border-border">{children}</table>
                </div>
              ),
              th: ({ children }) => (
                <th className="border border-border bg-muted px-3 py-2 text-left font-semibold text-sm">{children}</th>
              ),
              td: ({ children }) => <td className="border border-border px-3 py-2 text-sm">{children}</td>,
            }}
          >
            {content}
          </ReactMarkdown>
        </div>
      </CardContent>
    </Card>
  );
}
