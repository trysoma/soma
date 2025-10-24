import { TabsContent } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { FileText, ChevronRight, RefreshCw } from 'lucide-react'
import { ListPane } from './list-pane'
import { JsonView } from './json-view'
import type { Resource } from '@modelcontextprotocol/sdk/types.js'

interface ResourcesTabProps {
  resources: Resource[]
  listResources: () => void
  readResource: (uri: string) => void
  selectedResource: Resource | null
  setSelectedResource: (resource: Resource | null) => void
  resourceContent: string
}

export function ResourcesTab({
  resources,
  listResources,
  readResource,
  selectedResource,
  setSelectedResource,
  resourceContent
}: ResourcesTabProps) {
  return (
    <TabsContent value="resources">
      <div className="grid grid-cols-2 gap-4">
        <ListPane
          items={resources}
          listItems={listResources}
          clearItems={() => setSelectedResource(null)}
          setSelectedItem={(resource) => {
            setSelectedResource(resource)
            readResource(resource.uri)
          }}
          renderItem={(resource) => (
            <div className="flex items-center w-full">
              <FileText className="w-4 h-4 mr-2 flex-shrink-0 text-gray-500" />
              <span className="flex-1 truncate" title={resource.uri}>
                {resource.name}
              </span>
              <ChevronRight className="w-4 h-4 flex-shrink-0 text-gray-400" />
            </div>
          )}
          title="Resources"
          buttonText="List Resources"
        />

        <div className="border rounded-lg shadow">
          <div className="p-4 border-b flex justify-between items-center">
            <h3 className="font-semibold truncate">
              {selectedResource ? selectedResource.name : 'Select a resource'}
            </h3>
            {selectedResource && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => readResource(selectedResource.uri)}
              >
                <RefreshCw className="w-4 h-4 mr-2" />
                Refresh
              </Button>
            )}
          </div>
          <div className="p-4">
            {selectedResource ? (
              <JsonView data={resourceContent} />
            ) : (
              <p className="text-sm text-muted-foreground">
                Select a resource from the list to view its contents
              </p>
            )}
          </div>
        </div>
      </div>
    </TabsContent>
  )
}
