import type { DataPart, TextPart } from "@a2a-js/sdk";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism";

interface TextDataPartMarkdownProps {
  part: TextPart | DataPart;
}

export const TextDataPartMarkdown: React.FC<TextDataPartMarkdownProps> = ({ part }) => {
  if (part.kind === "text") {
    return (
      <ReactMarkdown
        components={{
          code({ children, className, ...rest }) {
            const match = /language-(\w+)/.exec(className || '')
            return match ? (
              <SyntaxHighlighter
                PreTag="div"
                language={match[1]}
                style={vscDarkPlus}
              >
                {String(children).replace(/\n$/, '')}
              </SyntaxHighlighter>
            ) : (
              <code {...rest} className={className}>
                {children}
              </code>
            )
          },
          h1: ({ ...props }) => <h1 className="text-3xl font-bold mb-4" {...props} />,
          h2: ({ ...props }) => <h2 className="text-2xl font-bold mb-3" {...props} />,
          h3: ({ ...props }) => <h3 className="text-xl font-bold mb-2" {...props} />,
          h4: ({ ...props }) => <h4 className="text-lg font-semibold mb-2" {...props} />,
          h5: ({ ...props }) => <h5 className="text-base font-semibold mb-1" {...props} />,
          h6: ({ ...props }) => <h6 className="text-sm font-bold mb-1" {...props} />,
        }}
      >
        {part.text}
      </ReactMarkdown>
    );
  } else {
    return (
      <ReactMarkdown
        components={{
          code({ children }) {
            return (
              <SyntaxHighlighter
                PreTag="div"
                language="json"
                style={vscDarkPlus}
              >
                {String(children).replace(/\n$/, '')}
              </SyntaxHighlighter>
            )
          }
        }}
      >
        {`\`\`\`json\n${JSON.stringify(part.data, null, 4)}\n\`\`\``}
      </ReactMarkdown>
    );
  }
};
