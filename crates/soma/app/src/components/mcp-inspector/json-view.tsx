import { useState, useMemo } from 'react'
import { Button } from '@/components/ui/button'
import { Copy, CheckCheck } from 'lucide-react'

interface JsonViewProps {
  data: unknown
  className?: string
  isError?: boolean
}

export function JsonView({ data, className = '', isError = false }: JsonViewProps) {
  const [copied, setCopied] = useState(false)

  const normalizedData = useMemo(() => {
    return typeof data === 'string'
      ? (() => {
          try {
            return JSON.parse(data)
          } catch {
            return data
          }
        })()
      : data
  }, [data])

  const handleCopy = () => {
    const text = typeof normalizedData === 'string'
      ? normalizedData
      : JSON.stringify(normalizedData, null, 2)

    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className={`relative border rounded-lg p-4 ${className}`}>
      <Button
        size="icon"
        variant="ghost"
        className="absolute top-2 right-2"
        onClick={handleCopy}
      >
        {copied ? (
          <CheckCheck className="w-4 h-4 text-green-600" />
        ) : (
          <Copy className="w-4 h-4" />
        )}
      </Button>
      <pre className={`font-mono text-sm overflow-auto ${isError ? 'text-red-600' : ''}`}>
        {typeof normalizedData === 'string'
          ? normalizedData
          : JSON.stringify(normalizedData, null, 2)}
      </pre>
    </div>
  )
}
