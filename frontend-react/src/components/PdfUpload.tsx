import { useMutation } from '@tanstack/react-query'
import { processPdf } from '../lib/api'
import { useState } from 'react'
import { Loader2, CheckCircle, AlertCircle, Download } from 'lucide-react'

interface PdfUploadProps {
  operation: string
}

export function PdfUpload({ operation }: PdfUploadProps) {
  const [dragActive, setDragActive] = useState(false)
  const [fileName, setFileName] = useState<string>('')

  const { mutate, isPending, data, error, reset } = useMutation({
    mutationFn: (file: File) => processPdf(file, operation),
  })

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true)
    } else if (e.type === 'dragleave') {
      setDragActive(false)
    }
  }

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setDragActive(false)
    const files = e.dataTransfer.files
    if (files?.[0]) {
      handleFile(files[0])
    }
  }

  const handleFile = (file: File) => {
    if (!file.type.includes('pdf') && !file.type.includes('document') && !file.type.includes('sheet')) {
      alert('Please upload a PDF or Office document')
      return
    }
    setFileName(file.name)
    mutate(file)
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      handleFile(file)
    }
  }

  return (
    <div className="space-y-4">
      {/* Upload Area */}
      <div
        onDragEnter={handleDrag}
        onDragLeave={handleDrag}
        onDragOver={handleDrag}
        onDrop={handleDrop}
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          dragActive ? 'border-blue-500 bg-blue-50' : 'border-gray-300 hover:border-gray-400'
        }`}
      >
        <input
          type="file"
          id="file-input"
          onChange={handleInputChange}
          className="hidden"
          accept=".pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.jpg,.jpeg,.png"
        />
        <label htmlFor="file-input" className="cursor-pointer">
          <div className="text-4xl mb-2">📁</div>
          <p className="text-lg font-semibold text-gray-700">Drag & drop your file here</p>
          <p className="text-sm text-gray-500 mt-1">or click to browse</p>
        </label>
      </div>

      {/* File Name Display */}
      {fileName && (
        <div className="flex items-center gap-2 text-sm text-gray-600">
          <span className="truncate">File: <strong>{fileName}</strong></span>
        </div>
      )}

      {/* Loading State */}
      {isPending && (
        <div className="flex items-center justify-center gap-2 bg-blue-50 border border-blue-200 rounded-lg p-4">
          <Loader2 className="w-5 h-5 animate-spin text-blue-600" />
          <span className="text-blue-700">Processing your file...</span>
        </div>
      )}

      {/* Success State */}
      {data?.success && !isPending && (
        <div className="space-y-3">
          <div className="flex items-center gap-2 bg-green-50 border border-green-200 rounded-lg p-4">
            <CheckCircle className="w-5 h-5 text-green-600" />
            <div>
              <p className="font-semibold text-green-900">{data.message}</p>
              <p className="text-sm text-green-700">
                File size: {formatFileSize(data.file_size || 0)}
              </p>
            </div>
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => reset()}
              className="flex-1 px-4 py-2 bg-gray-200 text-gray-800 rounded-lg hover:bg-gray-300 transition"
            >
              Process Another File
            </button>
            <button className="flex-1 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition flex items-center justify-center gap-2">
              <Download className="w-4 h-4" />
              Download
            </button>
          </div>
        </div>
      )}

      {/* Error State */}
      {error && !isPending && (
        <div className="flex items-center gap-2 bg-red-50 border border-red-200 rounded-lg p-4">
          <AlertCircle className="w-5 h-5 text-red-600" />
          <div>
            <p className="font-semibold text-red-900">Error</p>
            <p className="text-sm text-red-700">
              {error instanceof Error ? error.message : 'Failed to process file'}
            </p>
          </div>
        </div>
      )}
    </div>
  )
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
}
