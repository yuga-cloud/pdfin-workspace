import { useState } from 'react'
import { ChevronDown } from 'lucide-react'

const OPERATIONS = [
  { id: 'compress', label: 'Compress PDF', icon: '🗜️' },
  { id: 'merge', label: 'Merge PDFs', icon: '📎' },
  { id: 'split', label: 'Split PDF', icon: '✂️' },
  { id: 'rotate', label: 'Rotate Pages', icon: '🔄' },
  { id: 'watermark', label: 'Add Watermark', icon: '💧' },
  { id: 'page-numbers', label: 'Page Numbers', icon: '#️⃣' },
  { id: 'to-word', label: 'PDF to Word', icon: '📝' },
  { id: 'to-excel', label: 'PDF to Excel', icon: '📊' },
  { id: 'to-powerpoint', label: 'PDF to PowerPoint', icon: '🎯' },
  { id: 'word-to-pdf', label: 'Word to PDF', icon: '📄' },
  { id: 'excel-to-pdf', label: 'Excel to PDF', icon: '📈' },
  { id: 'powerpoint-to-pdf', label: 'PowerPoint to PDF', icon: '🎨' },
  { id: 'jpg-to-pdf', label: 'JPG to PDF', icon: '🖼️' },
  { id: 'to-jpg', label: 'PDF to JPG', icon: '📸' },
]

interface OperationSelectorProps {
  selected: string
  onSelect: (operation: string) => void
}

export function OperationSelector({ selected, onSelect }: OperationSelectorProps) {
  const [isOpen, setIsOpen] = useState(false)
  const selectedOp = OPERATIONS.find(op => op.id === selected)

  return (
    <div className="relative">
      <label className="block text-sm font-medium text-gray-700 mb-2">
        Select Operation
      </label>
      
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between px-4 py-3 bg-white border border-gray-300 rounded-lg hover:border-gray-400 transition text-left"
      >
        <span className="flex items-center gap-2">
          <span>{selectedOp?.icon}</span>
          <span className="text-gray-900">{selectedOp?.label}</span>
        </span>
        <ChevronDown className={`w-4 h-4 text-gray-600 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-white border border-gray-300 rounded-lg shadow-lg z-10 max-h-64 overflow-y-auto">
          {OPERATIONS.map(op => (
            <button
              key={op.id}
              onClick={() => {
                onSelect(op.id)
                setIsOpen(false)
              }}
              className={`w-full flex items-center gap-2 px-4 py-3 text-left hover:bg-blue-50 transition ${
                selected === op.id ? 'bg-blue-100 text-blue-900 font-semibold' : 'text-gray-700'
              }`}
            >
              <span>{op.icon}</span>
              <span>{op.label}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
