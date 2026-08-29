import { createLazyFileRoute } from '@tanstack/react-router'
import { PdfUpload } from '../components/PdfUpload'
import { OperationSelector } from '../components/OperationSelector'
import { useState } from 'react'

export const Route = createLazyFileRoute('/')({ 
  component: Home,
})

function Home() {
  const [selectedOperation, setSelectedOperation] = useState<string>('compress')

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-8">
      <div className="max-w-2xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-gray-900 mb-2">
            📄 PDFin Workspace
          </h1>
          <p className="text-lg text-gray-600">
            High-performance PDF & Office Document Processing
          </p>
        </div>

        {/* Main Card */}
        <div className="bg-white rounded-lg shadow-xl p-8">
          {/* Operation Selector */}
          <OperationSelector 
            selected={selectedOperation} 
            onSelect={setSelectedOperation} 
          />

          {/* Upload Area */}
          <div className="mt-8">
            <PdfUpload operation={selectedOperation} />
          </div>
        </div>

        {/* Feature Grid */}
        <div className="grid grid-cols-2 gap-4 mt-8 text-sm text-gray-600">
          <div className="bg-white p-4 rounded-lg shadow">✓ PDF Compression</div>
          <div className="bg-white p-4 rounded-lg shadow">✓ PDF Merge</div>
          <div className="bg-white p-4 rounded-lg shadow">✓ PDF Split</div>
          <div className="bg-white p-4 rounded-lg shadow">✓ Format Conversion</div>
          <div className="bg-white p-4 rounded-lg shadow">✓ Watermarking</div>
          <div className="bg-white p-4 rounded-lg shadow">✓ Page Numbering</div>
        </div>
      </div>
    </div>
  )
}
