import { Button } from '@/components/ui/button'

interface ListPaneProps<T> {
  items: T[]
  listItems: () => void
  clearItems: () => void
  setSelectedItem: (item: T) => void
  renderItem: (item: T) => React.ReactNode
  title: string
  buttonText: string
}

export function ListPane<T extends { name?: string; description?: string }>({
  items,
  listItems,
  clearItems,
  setSelectedItem,
  renderItem,
  title,
  buttonText
}: ListPaneProps<T>) {
  return (
    <div>
      <div className="mb-4">
        <h3 className="font-semibold mb-4">{title}</h3>
        <Button variant="outline" className="w-full mb-2" onClick={listItems}>
          {buttonText}
        </Button>
        <Button
          variant="outline"
          className="w-full mb-4"
          onClick={clearItems}
          disabled={items.length === 0}
        >
          Clear
        </Button>
      </div>
      <div className="space-y-2">
        {items.map((item, index) => (
          <div
            key={index}
            className="flex items-center py-2 px-4 rounded hover:bg-secondary cursor-pointer border"
            onClick={() => setSelectedItem(item)}
          >
            {renderItem(item)}
          </div>
        ))}
      </div>
    </div>
  )
}
