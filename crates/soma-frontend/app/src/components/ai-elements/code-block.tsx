"use client";

import { CheckIcon, CopyIcon } from "lucide-react";
import type { ComponentProps, HTMLAttributes, ReactNode } from "react";
import { createContext, useContext, useEffect, useState } from "react";
import { codeToHtml } from "shiki";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type CodeBlockContextType = {
	code: string;
};

const CodeBlockContext = createContext<CodeBlockContextType>({
	code: "",
});

export type CodeBlockProps = HTMLAttributes<HTMLDivElement> & {
	code: string;
	language: string;
	showLineNumbers?: boolean;
	children?: ReactNode;
};

export const CodeBlock = ({
	code,
	language,
	showLineNumbers = false,
	className,
	children,
	...props
}: CodeBlockProps) => {
	const [lightHtml, setLightHtml] = useState<string>("");
	const [darkHtml, setDarkHtml] = useState<string>("");

	useEffect(() => {
		// Generate syntax-highlighted HTML for light theme
		codeToHtml(code, {
			lang: language,
			theme: "github-light",
		})
			.then(setLightHtml)
			.catch(() => {
				// Fallback to plain code if language not supported
				setLightHtml(`<pre><code>${code}</code></pre>`);
			});

		// Generate syntax-highlighted HTML for dark theme
		codeToHtml(code, {
			lang: language,
			theme: "github-dark",
		})
			.then(setDarkHtml)
			.catch(() => {
				// Fallback to plain code if language not supported
				setDarkHtml(`<pre><code>${code}</code></pre>`);
			});
	}, [code, language]);

	return (
		<CodeBlockContext.Provider value={{ code }}>
			<div
				className={cn(
					"relative w-full overflow-hidden rounded-md border bg-background text-foreground",
					className,
				)}
				{...props}
			>
				<div className="relative">
					{/* Light theme */}
					<div
						className="overflow-hidden dark:hidden [&>pre]:m-0 [&>pre]:p-4 [&>pre]:text-sm [&>pre]:font-mono [&>pre]:bg-background"
						dangerouslySetInnerHTML={{ __html: lightHtml }}
					/>
					{/* Dark theme */}
					<div
						className="hidden overflow-hidden dark:block [&>pre]:m-0 [&>pre]:p-4 [&>pre]:text-sm [&>pre]:font-mono [&>pre]:bg-background"
						dangerouslySetInnerHTML={{ __html: darkHtml }}
					/>
					{children && (
						<div className="absolute top-2 right-2 flex items-center gap-2">
							{children}
						</div>
					)}
				</div>
			</div>
		</CodeBlockContext.Provider>
	);
};

export type CodeBlockCopyButtonProps = ComponentProps<typeof Button> & {
	onCopy?: () => void;
	onError?: (error: Error) => void;
	timeout?: number;
};

export const CodeBlockCopyButton = ({
	onCopy,
	onError,
	timeout = 2000,
	children,
	className,
	...props
}: CodeBlockCopyButtonProps) => {
	const [isCopied, setIsCopied] = useState(false);
	const { code } = useContext(CodeBlockContext);

	const copyToClipboard = async () => {
		if (typeof window === "undefined" || !navigator.clipboard.writeText) {
			onError?.(new Error("Clipboard API not available"));
			return;
		}

		try {
			await navigator.clipboard.writeText(code);
			setIsCopied(true);
			onCopy?.();
			setTimeout(() => setIsCopied(false), timeout);
		} catch (error) {
			onError?.(error as Error);
		}
	};

	const Icon = isCopied ? CheckIcon : CopyIcon;

	return (
		<Button
			className={cn("shrink-0", className)}
			onClick={copyToClipboard}
			size="icon"
			variant="ghost"
			{...props}
		>
			{children ?? <Icon size={14} />}
		</Button>
	);
};
