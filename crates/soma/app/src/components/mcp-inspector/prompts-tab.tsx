import { TabsContent } from '@/components/ui/tabs'
import { ChevronRight } from 'lucide-react'
import { ListPane } from './list-pane'
import { JsonView } from './json-view'
import type { Prompt } from '@modelcontextprotocol/sdk/types.js'

interface PromptsTabProps {
  prompts: Prompt[]
  listPrompts: () => void
  getPrompt: (name: string, args?: Record<string, string>) => void
  selectedPrompt: Prompt | null
  setSelectedPrompt: (prompt: Prompt | null) => void
  promptContent: string
}

export function PromptsTab({
  prompts,
  listPrompts,
  getPrompt,
  selectedPrompt,
  setSelectedPrompt,
  promptContent
}: PromptsTabProps) {
  return (
    <TabsContent value="prompts">
      <div className="grid grid-cols-2 gap-4">
        <ListPane
          items={prompts}
          listItems={listPrompts}
          clearItems={() => setSelectedPrompt(null)}
          setSelectedItem={(prompt) => {
            setSelectedPrompt(prompt)
            getPrompt(prompt.name)
          }}
          renderItem={(prompt) => (
            <div className="flex items-center w-full">
              <span className="flex-1 truncate font-medium">{prompt.name}</span>
              <ChevronRight className="w-4 h-4 flex-shrink-0 text-gray-400" />
            </div>
          )}
          title="Prompts"
          buttonText="List Prompts"
        />

        <div className="border rounded-lg shadow">
          <div className="p-4 border-b">
            <h3 className="font-semibold truncate">
              {selectedPrompt ? selectedPrompt.name : 'Select a prompt'}
            </h3>
          </div>
          <div className="p-4">
            {selectedPrompt ? (
              <>
                {selectedPrompt.description && (
                  <p className="text-sm text-muted-foreground mb-4">
                    {selectedPrompt.description}
                  </p>
                )}
                <JsonView data={promptContent} />
              </>
            ) : (
              <p className="text-sm text-muted-foreground">
                Select a prompt from the list to view its details
              </p>
            )}
          </div>
        </div>
      </div>
    </TabsContent>
  )
}
