import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn } from "@/lib/utils";

export const MarkdownDocumentation = ({ content }: { content: string }) => {
	if (!content) {
		return (
			<div className="text-sm text-muted-foreground">
				No documentation available
			</div>
		);
	}

	return (
		<div className="markdown-content">
			<ReactMarkdown
				remarkPlugins={[remarkGfm]}
				components={{
					h1: ({ node, ...props }) => (
						<h1 className="text-2xl font-bold mb-4 mt-6" {...props} />
					),
					h2: ({ node, ...props }) => (
						<h2 className="text-xl font-bold mb-3 mt-5" {...props} />
					),
					h3: ({ node, ...props }) => (
						<h3 className="text-lg font-semibold mb-2 mt-4" {...props} />
					),
					h4: ({ node, ...props }) => (
						<h4 className="text-base font-semibold mb-2 mt-3" {...props} />
					),
					h5: ({ node, ...props }) => (
						<h5 className="text-sm font-semibold mb-2 mt-3" {...props} />
					),
					h6: ({ node, ...props }) => (
						<h6 className="text-xs font-semibold mb-2 mt-3" {...props} />
					),
					p: ({ node, ...props }) => <p className="mb-4" {...props} />,
					ul: ({ node, ...props }) => (
						<ul className="list-disc pl-6 mb-4 space-y-2" {...props} />
					),
					ol: ({ node, ...props }) => (
						<ol className="list-decimal pl-6 mb-4 space-y-2" {...props} />
					),
					li: ({ node, ...props }) => <li className="mb-1" {...props} />,
					a: ({ node, ...props }) => (
						<a
							className="text-blue-600 hover:text-blue-800 underline"
							{...props}
						/>
					),
					code: ({ node, className, children, ...props }) => {
						const isInline = !className;
						return isInline ? (
							<code
								className="bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-sm font-mono"
								{...props}
							>
								{children}
							</code>
						) : (
							<code
								className={cn(
									"block bg-gray-100 dark:bg-gray-800 p-4 rounded text-sm font-mono overflow-x-auto",
									className,
								)}
								{...props}
							>
								{children}
							</code>
						);
					},
					pre: ({ node, ...props }) => (
						<pre className="mb-4 overflow-x-auto" {...props} />
					),
					blockquote: ({ node, ...props }) => (
						<blockquote
							className="border-l-4 border-gray-300 pl-4 italic my-4"
							{...props}
						/>
					),
					table: ({ node, ...props }) => (
						<table
							className="min-w-full border-collapse border border-gray-300 my-4"
							{...props}
						/>
					),
					thead: ({ node, ...props }) => (
						<thead className="bg-gray-100 dark:bg-gray-800" {...props} />
					),
					tbody: ({ node, ...props }) => <tbody {...props} />,
					tr: ({ node, ...props }) => (
						<tr className="border-b border-gray-300" {...props} />
					),
					th: ({ node, ...props }) => (
						<th
							className="border border-gray-300 px-4 py-2 text-left font-semibold"
							{...props}
						/>
					),
					td: ({ node, ...props }) => (
						<td className="border border-gray-300 px-4 py-2" {...props} />
					),
					hr: ({ node, ...props }) => (
						<hr className="my-8 border-t border-gray-300" {...props} />
					),
					strong: ({ node, ...props }) => (
						<strong className="font-bold" {...props} />
					),
					em: ({ node, ...props }) => <em className="italic" {...props} />,
				}}
			>
				{content}
			</ReactMarkdown>
		</div>
	);
};
